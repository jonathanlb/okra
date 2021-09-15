#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use okra::boxchecker::{ActionId, ActivityId, BoxChecker, BoxSearcher};
use okra::sqlite_boxchecker::SqliteBoxes;
use rocket::http::{Cookie, CookieJar, Method};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::convert::TryInto;

// TODO get db name for particular user
fn get_boxer(file_name: &str) -> SqliteBoxes {
    SqliteBoxes::new(file_name)
}

#[get("/action/get/<max_results>/<last_id>")]
fn get_actions(max_results: usize, last_id: usize) -> Option<Json<Vec<(ActionId, String)>>> {
    // XXX limit or ossify/remove max_results
    // TODO: add DB and user configuration
    let boxer = get_boxer("test.db");
    let mut dest = vec![(0, "".to_string()); max_results];
    let num_results = boxer.search_action_names("%", last_id.try_into().unwrap(), &mut dest);
    dest.truncate(num_results);
    Some(Json(dest))
}

#[get("/action/get_name/<action_id>")]
fn get_action_name(action_id: ActionId) -> Option<String> {
    let boxer = get_boxer("test.db");
    let name = boxer.get_action_name(action_id);
    if name == "" {
        None
    } else {
        Some(name)
    }
}

#[get("/activity/get/<start>/<end>/<max_results>")]
fn get_activities(
    start: usize,
    end: usize,
    max_results: usize,
) -> Option<Json<Vec<(ActivityId, ActionId)>>> {
    let boxer = get_boxer("test.db");
    let mut dest = vec![(0, 0); max_results];
    let num_results = boxer.search_activity_by_time(start, end, &mut dest);
    dest.truncate(num_results);
    Some(Json(dest))
}

#[get("/activity/log/<action_id>")]
fn log_activity(action_id: ActionId) -> Option<String> {
    let mut boxer = get_boxer("test.db");
    let id = boxer.log_activity(action_id);
    if id != 0 {
        Some(id.to_string()) // Responder<i64> not implemented
    } else {
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

#[post("/users/login", format = "application/json", data = "<login_info>")]
fn login(login_info: Json<LoginInfo>, cookies: &CookieJar<'_>) -> Option<String> {
    let cookie = Cookie::new("auth", "secret");
    cookies.add_private(cookie);
    Some(format!("hello {}", login_info.username))
}

#[get("/activity/notate/<activity_id>/<notes>")]
fn notate_activity(activity_id: ActivityId, notes: &str) -> Option<String> {
    let mut boxer = get_boxer("test.db");
    let id = boxer.annotate_activity(activity_id, notes);
    if id != 0 {
        Some(id.to_string()) // Responder<i64> not implemented
    } else {
        None
    }
}

#[launch]
fn rocket() -> _ {
    println!("starting!");
    env_logger::init();

    let allowed_origins = AllowedOrigins::all();
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::build()
        .attach(cors)
        .mount("/", routes![get_action_name])
        .mount("/", routes![get_actions])
        .mount("/", routes![get_activities])
        .mount("/", routes![log_activity])
        .mount("/", routes![login])
        .mount("/", routes![notate_activity])
}
