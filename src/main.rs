#![feature(proc_macro_hygiene, decl_macro, duration_consts_2)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use okra::auth::{login, logout, AuthKey};
use okra::boxchecker::{ActionId, ActivityId, BoxChecker, BoxSearcher};
use okra::sqlite_boxchecker::SqliteBoxes;
use rocket::http::Method;
use rocket::serde::json::Json;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::convert::TryInto;

static USER_BOX_PREFIX: &str = "data/user_";

fn get_boxer<'a>(auth: &'a AuthKey) -> SqliteBoxes<'a> {
    SqliteBoxes::new(format!("{}{}.sqlite", USER_BOX_PREFIX, auth.0).as_str())
}

#[get("/action/get/<max_results>/<last_id>")]
fn get_actions(
    max_results: usize,
    last_id: usize,
    auth: AuthKey,
) -> Option<Json<Vec<(ActionId, String)>>> {
    // XXX limit or ossify/remove max_results
    let boxer = get_boxer(&auth);
    let mut dest = vec![(0, "".to_string()); max_results];
    let num_results = boxer.search_action_names("%", last_id.try_into().unwrap(), &mut dest);
    dest.truncate(num_results);
    Some(Json(dest))
}

#[get("/action/get_name/<action_id>")]
fn get_action_name(action_id: ActionId, auth: AuthKey) -> Option<String> {
    let boxer = get_boxer(&auth);
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
    auth: AuthKey,
) -> Option<Json<Vec<(ActivityId, ActionId)>>> {
    let boxer = get_boxer(&auth);
    let mut dest = vec![(0, 0); max_results];
    let num_results = boxer.search_activity_by_time(start, end, &mut dest);
    dest.truncate(num_results);
    Some(Json(dest))
}

#[get("/activity/log/<action_id>")]
fn log_activity(action_id: ActionId, auth: AuthKey) -> Option<String> {
    let mut boxer = get_boxer(&auth);
    let id = boxer.log_activity(action_id);
    if id != 0 {
        Some(id.to_string()) // Responder<i64> not implemented
    } else {
        None
    }
}

#[get("/activity/notate/<activity_id>/<notes>")]
fn notate_activity(activity_id: ActivityId, notes: &str, auth: AuthKey) -> Option<String> {
    let mut boxer = get_boxer(&auth);
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
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept", "Content-type"]),
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
        .mount("/", routes![logout])
        .mount("/", routes![notate_activity])
}
