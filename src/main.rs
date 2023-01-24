#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::io::Cursor;
use std::num::ParseIntError;
use std::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use rocket::{Build, Request, Response, response, Rocket, State};
use rocket::form::FromForm;
use rocket::http::{ContentType, Status};
use rocket::request::FromParam;
use rocket::response::Responder;
use rocket::response::content;
use rocket::response::status::NotFound;
use serde::Deserialize;
use sqlx::{FromRow, PgPool};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[derive(Deserialize)]
struct Config {
    database_url: String,
}

struct VisitorCounter {
    visitor: AtomicU64,
}

impl VisitorCounter {
    fn increment(&self) {
        self.visitor.fetch_add(1, Ordering::Relaxed);
        println!("The number of visitors: {}", self.visitor.load(Ordering::Relaxed));
    }
}

#[derive(FromForm)]
struct Filters {
    age: u8,
    active: bool,
}

#[derive(Debug, FromRow)]
#[sqlx(rename_all = "camelCase")]
struct User {
    uuid: Uuid,
    name: String,
    age: i16,
    grade: i16,
    #[sqlx(rename = "active")]
    present: bool,
    #[sqlx(default)]
    not_in_database: String,
}

impl<'r> Responder<'r, 'r> for User {
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

impl<'r> Responder<'r, 'r> for NewUser {
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
async fn user(
    counter: &State<VisitorCounter>,
    pool: &rocket::State<PgPool>,
    uuid: &str,
) -> Result<User, Status> {
    counter.increment();
    let parsed_uuid = Uuid::parse_str(uuid)
        .map_err(|_| Status::BadRequest)?;

    let user = sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE uuid = $1"#,
        parsed_uuid
    ).fetch_one(pool.inner())
        .await;
    todo!()
}

#[get("/users/<name_grade>?<filters..>")]
fn users<'a>(
    counter: &State<VisitorCounter>,
    name_grade: NameGrade,
    filters: Option<Filters>,
) -> Result<NewUser<'a>, Status> {
    todo!()
}

#[catch(404)]
fn default_404(req: &Request) -> content::RawHtml<String> {
    content::RawHtml(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <title>404 Not Found</title>
</head>
<body align="center">
    <div role="main" align="center">
        <h1>404: Not Found</h1>
        <p>The requested resource <span style="background:gray; padding: 0 5px; color: white;">{}</span> could not be found.</p>
        <hr />
    </div>
    <div role="contentinfo" align="center">
        <small>Rocket</small>
    </div>
</body>
</html>
    "#, req.uri()
    ))
}

#[launch]
async fn rocket() -> Rocket<Build> {
    let visitor_counter = VisitorCounter {
        visitor: AtomicU64::new(0)
    };

    let starship = rocket::build();

    let config: Config = starship
        .figment()
        .extract()
        .expect("Incorrect Rocket.toml configuration");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to the database");

    starship
        .manage(visitor_counter)
        .manage(pool)
        .mount("/", routes![user,users])
        .register("/", catchers![default_404])
}