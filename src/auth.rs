use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::Outcome;
use rocket::request::{FromRequest, Request};
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post, request};
use sqlite::{Connection, State};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static AUTH_COOKIE: &str = "auth";
static USERS_DB_NAME: &str = "data/users.sqlite";
static USERS_COL_NAME: &str = "username";
static USERS_TABLE_NAME: &str = "users";
static SECRET_COL_NAME: &str = "secret";
static SESSION_DURATION: Duration = Duration::new(7 * 24 * 60 * 69, 0);

#[derive(Debug, Deserialize)]
pub struct LoginInfo<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug)]
pub struct AuthError {
    pub msg: String,
}

macro_rules! unwrap_msg {
    ($sql_err:expr) => {
        $sql_err.message.unwrap_or("???".to_string())
    };
}

pub trait Auth {
    fn add_user(&mut self, login: &LoginInfo) -> Result<bool, AuthError>;
    fn auth_cookie(&self, cookie: &str) -> Result<String, AuthError>;
    fn auth_user(&self, login: &LoginInfo) -> Result<String, AuthError>;
}

pub struct SqliteAuth {
    conn: Connection,
}

impl SqliteAuth {
    pub fn new(path: &str) -> Result<Self, sqlite::Error> {
        let conn = sqlite::open(path).unwrap();
        let query = format!(
            "
                CREATE TABLE IF NOT EXISTS {} ({} TEXT UNIQUE, {} TEXT);
                CREATE INDEX IF NOT EXISTS idx_username ON {} ({});
            ",
            USERS_TABLE_NAME, USERS_COL_NAME, SECRET_COL_NAME, USERS_TABLE_NAME, USERS_COL_NAME
        );
        {
            // explicit lifetime to avoid conn.drop();
            let mut stat = conn.prepare(query)?;
            stat.next()?;
        }
        Ok(SqliteAuth { conn: conn })
    }
}

/// Check that the plaintext/extracted authorization cookie is valid and
/// has not yet expired.
/// The format of the token is the epoch milliseconds followed by the (unchecked)
/// username.
fn auth_cookie(cookie: &str) -> Result<String, AuthError> {
    let tokens: Vec<&str> = cookie.split(' ').collect();
    if tokens.len() < 2 {
        return Err(AuthError {
            msg: format!("invalid auth cookie: '{}'", cookie),
        });
    }

    match tokens[0].parse::<u128>() {
        Ok(expiry) => {
            let epochs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            if epochs < expiry {
                Ok("OK".to_string())
            } else {
                Err(AuthError {
                    msg: format!("expired auth cookie: '{}'", cookie),
                })
            }
        }
        Err(_) => Err(AuthError {
            msg: format!("invalid auth cookie: '{}'", cookie),
        }),
    }
}

impl Auth for SqliteAuth {
    /// Insert a new user into the users database.
    fn add_user(&mut self, login: &LoginInfo) -> Result<bool, AuthError> {
        let query = format!(
            "INSERT INTO {} ({}, {}) VALUES(?, ?);",
            USERS_TABLE_NAME, USERS_COL_NAME, SECRET_COL_NAME
        );
        let mut stat = self.conn.prepare(query).unwrap();
        stat.bind(1, login.username).unwrap();

        let hashed = hash(login.password, DEFAULT_COST).unwrap();
        stat.bind(2, hashed.as_str()).unwrap();
        match stat.next() {
            Ok(_) => Ok(true),
            Err(e) => Err(AuthError {
                msg: format!("cannot add user: {}", unwrap_msg!(e)),
            }),
        }
    }

    /// Check that the plaintext/extracted authorization cookie is valid and
    /// has not yet expired.
    /// The format of the token is the epoch milliseconds followed by the (unchecked)
    /// username.
    fn auth_cookie(&self, cookie: &str) -> Result<String, AuthError> {
        auth_cookie(cookie)
    }

    /// Query the database for username and password match, returning a
    /// plaintext authorization cookie to be privately recorded.
    /// TODO: edit configuration to return non 404 error response.
    fn auth_user(&self, login: &LoginInfo) -> Result<String, AuthError> {
        let query = format!(
            "SELECT {} FROM {} WHERE {} = ?",
            SECRET_COL_NAME, USERS_TABLE_NAME, USERS_COL_NAME
        );
        let mut stat = self.conn.prepare(query).unwrap();
        stat.bind(1, login.username).unwrap();
        match stat.next() {
            Ok(State::Row) => {
                let stored_secret = stat.read::<String>(0).unwrap();

                log::debug!(
                    "checking {} {} {}",
                    login.username,
                    login.password,
                    stored_secret
                );
                match verify(login.password, &stored_secret) {
                    Ok(verified) => {
                        if verified {
                            Ok(get_auth_str(login))
                        } else {
                            log::error!("invalid user/pass {} {}", login.username, login.password);
                            Err(AuthError {
                                msg: "invalid password".to_string(),
                            })
                        }
                    }
                    Err(e) => {
                        log::error!("bcrypt error: {}", e);
                        Err(AuthError { msg: e.to_string() })
                    }
                }
            }
            Ok(State::Done) => Err(AuthError {
                msg: format!("no user: {}", login.username),
            }),
            Err(e) => Err(AuthError {
                msg: format!(
                    "failed to lookup user {}: {}",
                    login.username,
                    unwrap_msg!(e)
                ),
            }),
        }
    }
}

/// Construct an authorization string valid for the default duration.
fn get_auth_str(login: &LoginInfo) -> String {
    let expiry = SystemTime::now()
        .checked_add(SESSION_DURATION)
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{} {}", expiry, login.username)
}

#[derive(Debug)]
pub struct AuthKey(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthKey {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.cookies().get_private(AUTH_COOKIE) {
            Some(cookie) => {
                log::debug!("cookie {}", cookie.value());
                let tokens: Vec<&str> = cookie.value().split(' ').collect();
                let username = tokens[1];
                let expiry_str = tokens[0];
                match expiry_str.parse::<u128>() {
                    Ok(expiry) => {
                        let epochs = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        if epochs < expiry {
                            Outcome::Success(AuthKey(username.to_string()))
                        } else {
                            Outcome::Failure((
                                Status::NotExtended,
                                AuthError {
                                    msg: "Expired".to_string(),
                                },
                            ))
                        }
                    }
                    Err(_) => Outcome::Failure((
                        Status::BadRequest,
                        AuthError {
                            msg: "InvalidAuth".to_string(),
                        },
                    )),
                }
            }
            None => Outcome::Failure((
                Status::Unauthorized,
                AuthError {
                    msg: "Unauthorized".to_string(),
                },
            )),
        }
    }
}

fn get_auth() -> SqliteAuth {
    SqliteAuth::new(USERS_DB_NAME).unwrap()
}

#[post("/users/login", format = "application/json", data = "<login_info>")]
pub fn login(login_info: Json<LoginInfo>, cookies: &CookieJar<'_>) -> Option<String> {
    match get_auth().auth_user(&login_info) {
        Ok(token) => {
            let cookie = Cookie::build(AUTH_COOKIE, token).finish();
            cookies.add_private(cookie);
            Some(format!("hello {}", login_info.username))
        }
        Err(_) => {
            None // XXX TODO redirect
        }
    }
}

#[get("/users/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Option<String> {
    cookies.remove_private(Cookie::named(AUTH_COOKIE));
    Some("OK".to_string())
}

#[cfg(test)]
#[path = "./auth_test.rs"]
mod auth_test;
