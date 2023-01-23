#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::io::Cursor;
use std::num::ParseIntError;
use lazy_static::lazy_static;
use rocket::{Build, Request, Response, response, Rocket};
use rocket::form::FromForm;
use rocket::http::ContentType;
use rocket::request::FromParam;
use rocket::response::Responder;


#[derive(FromForm)]
struct Filters {
    age: u8,
    active: bool,
}

#[derive(Debug)]
struct User {
    uuid: String,
    name: String,
    age: u8,
    grade: u8,
    active: bool,
}

impl<'r> Responder<'r, 'r> for &'r User {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        let base_response = default_response();
        let user = format!("Found user: {:?}", self);
        Response::build()
            .sized_body(user.len(), Cursor::new(user))
            .raw_header("X-USER-ID", self.uuid.to_string())
            .merge(base_response)
            .ok()
    }
}

struct NewUser<'a>(Vec<&'a User>);

impl<'r> Responder<'r, 'r> for NewUser<'r> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        let base_response = default_response();
        let result = self.0
            .iter()
            .map(|u| format!("{:?}", u))
            .collect::<Vec<String>>()
            .join(",");

        Response::build()
            .sized_body(result.len(), Cursor::new(result))
            .raw_header("X-CUSTOM-ID", "USERS")
            .join(base_response)
            .ok()
    }
}

fn default_response<'r>() -> Response<'r> {
    Response::build()
        .header(ContentType::Plain)
        .raw_header("X-CUSTOM-HEADER", "CUSTOM")
        .finalize()
}

lazy_static! {
    static ref USERS: HashMap<&'static str, User> = {
        let mut map = HashMap::new();
        map.insert(
            "74d96050-8d8b-45e5-ac48-40c35208841e",
            User{
                uuid: String::from("74d96050-8d8b-45e5-ac48-40c35208841e"),
                name: "Alex".to_string(),
                age: 36,
                grade: 10,
                active: true
            }
        );
        map
    };
}

struct NameGrade<'r> {
    name: &'r str,
    grade: u8,
}

impl<'r> FromParam<'r> for NameGrade<'r> {
    type Error = &'static str;
    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        const ERROR_MESSAGE: Result<NameGrade, &'static str> = Err("Error parsing user parameter");

        let name_grade: Vec<&'r str> = param.split('_').collect();

        match name_grade.len() {
            2 => match name_grade[1].parse::<u8>()
            {
                Ok(n) => Ok(
                    Self {
                        name: name_grade[0],
                        grade: n,
                    }
                ),
                Err(_) => ERROR_MESSAGE
            }
            _ => ERROR_MESSAGE
        }
    }
}

#[route(GET, uri = "/user/<uuid>", rank = 1, format = "text/html")]
fn user(uuid: &str) -> Option<&User> {
    USERS.get(uuid)
}

#[get("/users/<name_grade>?<filters..>")]
fn users(name_grade: NameGrade, filters: Option<Filters>) -> Option<NewUser> {
    let users: Vec<&User> = USERS
        .values()
        .filter(|u| u.name.contains(name_grade.name) && u.grade == name_grade.grade)
        .filter(|u| {
            if let Some(filter) = &filters {
                println!("{:?}", filter.active);
                println!("{:?}", filter.age);
                u.age == filter.age && u.active == filter.active
            } else {
                print!("No filters");
                true
            }
        })
        .collect();

    if users.len() > 0 {
        Some(NewUser(users))
    } else {
        None
    }
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![user,users])
}