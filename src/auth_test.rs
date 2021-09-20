use super::*;

#[test]
fn denies_missing_user() {
    let auth = SqliteAuth::new(":memory:").unwrap();
    let login = LoginInfo {
        username: "bob",
        password: "secret",
    };
    assert!(auth.auth_user(&login).is_err());
}

#[test]
fn denies_bad_password() {
    let mut auth = SqliteAuth::new(":memory:").unwrap();
    let mut login = LoginInfo {
        username: "bob",
        password: "secret",
    };
    auth.add_user(&login).unwrap();
    login.password = "guess";
    assert!(auth.auth_user(&login).is_err());
}

#[test]
fn grants_user() {
    let mut auth = SqliteAuth::new(":memory:").unwrap();
    let login = LoginInfo {
        username: "bob",
        password: "secret",
    };
    auth.add_user(&login).unwrap();
    let cookie = auth.auth_user(&login).unwrap();
    assert!(cookie.len() > 0);
}

#[test]
fn auths_good_cookie() {
    let mut auth = SqliteAuth::new(":memory:").unwrap();
    let login = LoginInfo {
        username: "bob",
        password: "secret",
    };
    auth.add_user(&login).unwrap();
    let cookie = auth.auth_user(&login).unwrap();
    let new_cookie = auth.auth_cookie(&cookie).unwrap();
    assert!(new_cookie.len() > 0);
}

#[test]
fn denies_bad_cookie() {
    let auth = SqliteAuth::new(":memory:").unwrap();
    let cookie = "abc";
    assert!(auth.auth_cookie(&cookie).is_err());
}

#[test]
fn rejects_adding_duplicate_user() {
    let mut auth = SqliteAuth::new(":memory:").unwrap();
    let login = LoginInfo {
        username: "bob",
        password: "secret",
    };
    assert!(auth.add_user(&login).is_ok());
    assert!(auth.add_user(&login).is_err());
}
