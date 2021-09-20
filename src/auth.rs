use rocket::{post};
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use sqlite::{Connection, State};


static USERS_TABLE_NAME: &str = "users";
static USERS_COL_NAME: &str = "username";
static SECRET_COL_NAME: &str = "secret";

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginInfo<'a> {
    username: &'a str,
    password: &'a str,
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

trait Auth {
    fn add_user(&mut self, login: &LoginInfo) -> Result<bool, AuthError>;
    fn auth_cookie(&self, cookie: &str) -> Result<String, AuthError>;
    fn auth_user(&self, login: &LoginInfo) -> Result<String, AuthError>;
}

struct SqliteAuth {
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
            USERS_TABLE_NAME,
            USERS_COL_NAME,
            SECRET_COL_NAME,
            USERS_TABLE_NAME,
            USERS_COL_NAME
            );
        {   // explicit lifetime to avoid conn.drop();
            let mut stat = conn.prepare(query)?;
            stat.next()?;
        }
        Ok(SqliteAuth { conn: conn })
    }
}

impl Auth for SqliteAuth {
    fn add_user(&mut self, login: &LoginInfo) -> Result<bool, AuthError> {
        let query = format!(
            "INSERT INTO {} ({}, {}) VALUES(?, ?);",
            USERS_TABLE_NAME, USERS_COL_NAME, SECRET_COL_NAME);
        let mut stat = self.conn.prepare(query).unwrap();
        stat.bind(1, login.username).unwrap();
        stat.bind(2, login.password).unwrap(); // XXX hash TODO
        match stat.next() {
            Ok(_) => Ok(true),
            Err(e) => Err(AuthError {
                msg: format!("cannot add user: {}", unwrap_msg!(e)),
            }),
        }
    }

    fn auth_cookie(&self, cookie: &str) -> Result<String, AuthError> {
        Err(AuthError{
            msg: "unimplemented".to_string()
        })
    }

    fn auth_user(&self, login: &LoginInfo) -> Result<String, AuthError> {
        let query = format!(
            "SELECT {} FROM {} WHERE {} = ?",
            SECRET_COL_NAME, USERS_TABLE_NAME, USERS_COL_NAME);
        let mut stat = self.conn.prepare(query).unwrap();
        stat.bind(1, login.username).unwrap();
        match stat.next() {
            Ok(State::Row) => {
                let stored_secret = stat.read::<String>(0).unwrap();
                // XXX hash TODO
                if login.password.eq(&stored_secret) {
                    Ok("OK".to_string())
                } else {
                    Err(AuthError{
                        msg: "invalid password".to_string(),
                    })
                }
            },
            Ok(State::Done) => Err(AuthError{
                msg: format!("no user: {}", login.username),
            }),
            Err(e) => Err(AuthError {
                msg: format!("failed to lookup user {}: {}", login.username, unwrap_msg!(e)),
            }),
        }

        
    }
}

#[post("/users/login", format = "application/json", data = "<login_info>")]
pub fn login(login_info: Json<LoginInfo>, cookies: &CookieJar<'_>) -> Option<String> {
    let cookie = Cookie::new("auth", "secret");
    cookies.add_private(cookie);
    Some(format!("hello {}", login_info.username))
}

#[cfg(test)]
#[path = "./auth_test.rs"]
mod auth_test;
