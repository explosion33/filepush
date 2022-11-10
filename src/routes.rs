use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs::create_dir_all;

use rocket::data::ToByteUnit;
use rocket::{
    self,
    Config,
    State,
    fs::{NamedFile, TempFile},
    form::{Form, FromForm},
    Request,
    request::{self, Outcome, FromRequest},
    response::status,
    response::stream::{Event, EventStream},
    tokio::time::{self, Duration},
    serde::json::Json,
    serde::Deserialize,
    serde::Serialize,
    http::Status,
    Data,
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

struct UserLogin {
    username: String,
    password: String,
}

struct FileHeader {
    filename: String,
    bytes: u64,
}

#[derive(Debug)]
enum HeaderError {
    Missing,
    BadFormat,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserLogin {
    type Error = HeaderError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let username = match req.headers().get_one("username") {
            Some(n) => n,
            None => {return Outcome::Failure((Status::Unauthorized, HeaderError::Missing))}
        };
        let password = match req.headers().get_one("password") {
            Some(n) => n,
            None => {return Outcome::Failure((Status::Unauthorized, HeaderError::Missing))}
        };
        

        Outcome::Success(UserLogin {
            username: username.to_string(),
            password: password.to_string(),
        })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for FileHeader {
    type Error = HeaderError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let filename = match req.headers().get_one("filename") {
            Some(n) => n,
            None => {return Outcome::Failure((Status::BadRequest, HeaderError::Missing))}
        };
        let bytes_str = match req.headers().get_one("bytes") {
            Some(n) => n,
            None => {return Outcome::Failure((Status::BadRequest, HeaderError::Missing))}
        };

        let bytes: u64 = match bytes_str.parse() {
            Ok(n) => n,
            Err(_) => {
                return Outcome::Failure((Status::BadRequest, HeaderError::BadFormat))
            }
        };
        

        Outcome::Success(FileHeader {
            filename: filename.to_string(),
            bytes,
        })
    }
}


fn safe_path(path: String) -> String {
    path
        .replace("..", "")
        .replace("/", "")
        .replace("~", "")
}

#[rocket::post("/file_upload", data = "<file>")]
async fn file_upload(state: &State<TUsers>, file: Data<'_>, user: UserLogin, upload: FileHeader) -> Result<status::Accepted<String>, status::BadRequest<String>> {

    println!("{}, {}", user.username, user.password);
    println!("{}, {}", upload.filename, upload.bytes);

    // mutex cannot exist in the same scope as await
    // awaiting does not garuntee the same thread
    // so we offload the verification to a closure
    let verify = || -> bool {
        let users = match state.lock() {
            Ok(n) => n,
            Err(_) => {return false;}
        };

        let user = NewUser {username: user.username.clone(), password: user.password};
        return users.verify_user(&user);
    };

    let base = "user_images/".to_string() + user.username.as_str();
    let path = base.clone() + "/" + safe_path(upload.filename).as_str();

    println!("{}", path);

    match verify() {
        true => {},
        false => {
            return Err(status::BadRequest(Some(format!("Invalid Username or Password"))));
        },
    }

    match create_dir_all(base) {
        Ok(_) => {},
        Err(_) => {
            return Err(status::BadRequest(Some(format!("IO Error"))));
        },
    };
    

    match file.open(upload.bytes.bytes()).into_file(path).await {
        Ok(_) => {},
        Err(_) => {
            return Err(status::BadRequest(Some(format!("IO Error"))));
        },
    };

    // WEBHOOK
    // EVENT SERVER
    // FILE TIMER

    Ok(status::Accepted(Some(format!("200 OK"))))

}

#[rocket::get("/file/<file>")]
async fn file_link(state: &State<TUsers>, user: UserLogin, file: PathBuf) -> Result<NamedFile, status::BadRequest<String>> {
    let verify = || -> bool {
        let users = match state.lock() {
            Ok(n) => n,
            Err(_) => {return false;}
        };

        let user = NewUser {username: user.username.clone(), password: user.password};
        return users.verify_user(&user);
    };

    match verify() {
        true => {},
        false => {
            return Err(status::BadRequest(Some(format!("Invalid Username or Password"))));
        },
    }


    let base = "user_files/".to_string() + user.username.as_str();


    match NamedFile::open(Path::new(&base).join(file)).await {
        Ok(n) => Ok(n),
        Err(_) =>  Err(status::BadRequest(Some(format!("File does not exist"))))
    }
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
            .mount("/", rocket::routes![index, login, register, user, verify_user, delete_user, register_user, file_upload, file_link, get_file, stream])
            .attach(Template::fairing())
            .manage(users)
            .launch()
            .await;
        });
}