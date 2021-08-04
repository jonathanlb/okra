#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use okra::boxchecker::{ActionId, ActivityId, BoxChecker, BoxSearcher};
use okra::sqlite_boxchecker::SqliteBoxes;
use std::convert::TryInto;

#[get("/action/get/<max_results>/<last_id>")]
fn get_actions(max_results: usize, last_id: usize) -> String {
    // XXX limit or ossify/remove max_results
    // TODO: add DB and user configuration
    let boxer = SqliteBoxes::new("test.db");
    let mut dest = vec![(0, "".to_string()); max_results];
    boxer.search_action_names("%", last_id.try_into().unwrap(), &mut dest);
    json!(dest).to_string()
}

#[get("/action/get_name/<action_id>")]
fn get_action_name(action_id: ActionId) -> Option<String> {
    let boxer = SqliteBoxes::new("test.db");
    let name = boxer.get_action_name(action_id);
    if name == "" {
        None
    } else {
        Some(name)
    }
}

#[get("/activity/log/<action_id>")]
fn log_activity(action_id: ActionId) -> Option<String> {
    let mut boxer = SqliteBoxes::new("test.db");
    let id = boxer.log_activity(action_id);
    if id != 0 {
        Some(id.to_string()) // Responder<i64> not implemented
    } else {
        None
    }
}

#[get("/activity/notate/<activity_id>/<notes>")]
fn notate_activity(activity_id: ActivityId, notes: &str) -> Option<String> {
    let mut boxer = SqliteBoxes::new("test.db");
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
    rocket::build()
        .mount("/", routes![get_action_name])
        .mount("/", routes![get_actions])
        .mount("/", routes![log_activity])
        .mount("/", routes![notate_activity])
}
