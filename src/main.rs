#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};

#[derive(FromForm)]
struct Filters {
    age: u8,
    active: bool,
}

#[route(GET, uri = "/user/<uuid>", rank = 1, format = "text/plain")]
fn user(uuid: &str) {}

#[route(GET, uri = "/users/<grade>?<filters..>", rank = 1, format = "text/plain")]
fn users(grade: u8, filters: Filters) {}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![user,users])
}