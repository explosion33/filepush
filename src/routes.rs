use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use rocket::{
    self,
    Config,
    State,
    fs::NamedFile,
    response::status,
    response::stream::{Event, EventStream},
    tokio::time::{self, Duration},
    serde::json::Json,
    serde::Deserialize,
    serde::Serialize,
};
use rocket_dyn_templates::Template;

use crate::users::{Users, User, NewUser};

pub type TUsers = Arc<Mutex<Users>>;

#[rocket::get("/event")]
fn index() -> Template {

    Template::render("event", rocket_dyn_templates::context!{})
}

#[rocket::get("/")]
fn login() -> Template {

    Template::render("login", rocket_dyn_templates::context!{})
}

#[rocket::get("/register")]
fn register() -> Template {

    Template::render("register", rocket_dyn_templates::context!{})
}

#[rocket::get("/user")]
fn user() -> String {

    "user page".to_string()
}


#[rocket::get("/events")]
fn stream() -> EventStream![] {
    EventStream! {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            yield Event::data("ping");
            interval.tick().await;
        }
    }
}

#[rocket::post("/register", data = "<data>")]
fn register_user(state: &State<TUsers>, data: Json<NewUser>) -> Result<status::Accepted<String>, status::BadRequest<String>> {
    let mut users = state.lock().unwrap();
    let new_user = data.into_inner();

    match users.add_new_user(&new_user) {
        Ok(_) => {
            return Ok(status::Accepted(Some(format!(""))));
        },
        Err(_) => {
            return Err(status::BadRequest(Some(format!("User already exists"))));
        }
    };
}

#[rocket::post("/verify", data = "<data>")]
fn verify_user(state: &State<TUsers>, data: Json<NewUser>) -> Result<status::Accepted<String>, status::BadRequest<String>> {
    let users = state.lock().unwrap();
    let new_user = data.into_inner();

    if users.verify_user(&new_user) {
        return Ok(status::Accepted(Some(format!(""))));
    }

    Err(status::BadRequest(Some(format!("Username or Password is invalid"))))
}

#[rocket::post("/delete_user", data = "<data>")]
fn delete_user(state: &State<TUsers>, data: Json<NewUser>) -> Result<status::Accepted<String>, status::BadRequest<String>> {
    let mut users = state.lock().unwrap();
    let new_user = data.into_inner();

    if !users.verify_user(&new_user) {
        return Err(status::BadRequest(Some(format!("Username or Password is invalid"))));
    }

    let user = match users.find_user(&new_user.username) {
        Some(n) => n,
        None => {return Err(status::BadRequest(Some(format!("User does not exist"))))},
    };

    users.remove_user(&user);

    return Ok(status::Accepted(Some(format!("200 OK"))));
}


#[rocket::get("/static/<file>")]
async fn get_file(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).await.ok()
}


fn update_file(users: TUsers, delay: Duration) {
    loop {
        thread::sleep(delay);
        match users.lock() {
            Ok(n) => {
                n.update_file();
            },
            Err(_) => {},
        }
    }
}

pub fn start_api() {
    let users: TUsers = Arc::new(Mutex::new(Users::new("users.txt".to_string())));

    let users_copy = Arc::clone(&users);

    thread::spawn(move || {
        update_file(users_copy, Duration::from_millis(2000));
    });


    rocket::tokio::runtime::Builder::new_multi_thread()
        .worker_threads(Config::from(Config::figment()).workers)
        // NOTE: graceful shutdown depends on the "rocket-worker" prefix.
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .expect("create tokio runtime")
        .block_on(async move {
            let _ = rocket::build()
            .mount("/", rocket::routes![index, login, register, user, verify_user, delete_user, register_user, get_file, stream])
            .attach(Template::fairing())
            .manage(users)
            .launch()
            .await;
        });
}