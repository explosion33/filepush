use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::fs::{create_dir_all, read_dir, remove_file};

use rocket::{
    self,
    Config,
    State,
    fs::NamedFile,
    Request,
    request::{self, Outcome, FromRequest},
    response::status,
    response::stream::{Event, EventStream},
    tokio::time::{self, Duration},
    serde::json::Json,
    http::Status,
    Data,
    data::ToByteUnit,
};
use rocket_dyn_templates::Template;

use crate::users::{Users, NewUser};

pub type TUsers = Arc<Mutex<Users>>;

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
            None => {
                println!("missing file name");
                return Outcome::Failure((Status::BadRequest, HeaderError::Missing))}
        };
        let bytes_str = match req.headers().get_one("bytes") {
            Some(n) => n,
            None => {
                println!("missing file size");
                return Outcome::Failure((Status::BadRequest, HeaderError::Missing))}
        };

        let bytes: u64 = match bytes_str.parse() {
            Ok(n) => n,
            Err(n) => {
                println!("{} | recieved: {}", n, bytes_str);
                return Outcome::Failure((Status::BadRequest, HeaderError::BadFormat))
            }
        };
        

        Outcome::Success(FileHeader {
            filename: filename.to_string(),
            bytes,
        })
    }
}

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
fn user() -> Template {
    Template::render("view", rocket_dyn_templates::context!{})
}

#[rocket::get("/client")]
fn client() -> Template {
    Template::render("client", rocket_dyn_templates::context!{})
}


#[rocket::get("/view/<file>")]
fn view(file: String) -> Template {
    Template::render("img", rocket_dyn_templates::context!{file_name: file})
}


#[rocket::get("/events")]
fn stream(state: &State<TUsers>, user: UserLogin) -> Result<EventStream![Event + '_], status::Unauthorized<String>> {
    let mut users = match state.lock() {
        Ok(n) => n,
        Err(_) => {return Err(status::Unauthorized(Some(format!("Internal Server Error"))));}
    };

    let user = NewUser {username: user.username.clone(), password: user.password.clone()};
    if !users.verify_user(&user) {
        return Err(status::Unauthorized(Some(format!("Invalid Username or Password"))));
    }

    {
        let mut user = users.find_user(&user.username).unwrap();
        user.has_client = true;
        users.update_user(user);
    }

    Ok( EventStream! {
        let mut interval = time::interval(Duration::from_secs(1));

        let get_new_files = || -> Vec<String> {
            let mut users = state.lock().unwrap();
            let mut user = users.find_user(&user.username).unwrap();
            let files = user.new_files.clone();

            user.new_files = Vec::new();
            users.update_user(user);

            return files;
        };

        let (tx, rx) = mpsc::channel();

        let users = Arc::clone(state.inner());
        let username = user.username.clone();

        thread::spawn(move || {
            detect_client_still_connected(rx, users, username);
        });

        loop {
            let _ = tx.send(String::new());

            let new_files = get_new_files();

            if (new_files.len() != 0) {
                yield Event::json(&new_files);
            }

            interval.tick().await;
        }
    })
}

#[rocket::post("/register", data = "<data>")]
fn register_user(state: &State<TUsers>, data: Json<NewUser>) -> Result<status::Accepted<String>, status::BadRequest<String>> {
    let mut users = state.lock().unwrap();
    let new_user = data.into_inner();

    match users.add_new_user(&new_user) {
        Ok(_) => {

            let base = "user_files/".to_string() + new_user.username.as_str();

            match create_dir_all(base) {
                Ok(_) => {},
                Err(_) => {
                    return Err(status::BadRequest(Some(format!("IO Error"))));
                },
            };

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

    let name: String = safe_path(upload.filename);
    let base = "user_files/".to_string() + user.username.as_str();
    let path = base.clone() + "/" + name.as_str();

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

    {
        match state.lock() {
            Ok(mut users) => {
                let mut user = users.find_user(&user.username).unwrap();
                user.new_files.push(name.clone());
                users.update_user(user);
            },
            Err(_) => {}
        };

    }

    // FILE TIMER

    Ok(status::Accepted(Some(format!("{}", name))))

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

#[rocket::get("/has_client")]
fn has_client(state: &State<TUsers>, user: UserLogin) -> Result<status::Accepted<String>, status::BadRequest<String>> {
    let users = match state.lock() {
        Ok(n) => n,
        Err(_) => {return Err(status::BadRequest(Some(format!("InternalServerError"))));}
    };

    let user = NewUser {username: user.username.clone(), password: user.password};
    if !users.verify_user(&user) {
        return Err(status::BadRequest(Some(format!("Invalid Username or Password"))));
    }

    let mut out: String = "false".to_string();
    if users.find_user(&user.username).unwrap().has_client {
        out = "true".to_string();
    }

    Ok(status::Accepted(Some(out)))

}



#[rocket::get("/<username>/<file>")]
async fn file_link_public(state: &State<TUsers>, username: String, file: String) -> Result<NamedFile, status::BadRequest<String>> {
    let verify = || -> bool {
        let users = match state.lock() {
            Ok(n) => n,
            Err(_) => {return false;}
        };

        match users.find_user(&username) {
            Some(n) => n.settings.public.contains(&file.to_string()),
            None => false,
        }
    };

    match verify() {
        true => {},
        false => {
            return Err(status::BadRequest(Some(format!("File does not exist"))));
        },
    }


    let base = "user_files/".to_string() + username.as_str();
    let path = base + "/" + file.as_str();


    println!("{}", path);

    match NamedFile::open(Path::new(&path)).await {
        Ok(n) => Ok(n),
        Err(n) =>  {
            println!("{:?}", n);
            Err(status::BadRequest(Some(format!("File does not exist"))))
        }
    }
}

#[rocket::get("/images")]
fn get_user_images(state: &State<TUsers>, user: UserLogin) -> Result<Json<Vec<(String, bool)>>, status::BadRequest<String>> {
    let users = match state.lock() {
        Ok(n) => n,
        Err(_) => {return Err(status::BadRequest(Some(format!("Internal Server Error"))));}
    };

    let user = NewUser {username: user.username.clone(), password: user.password};
    match users.verify_user(&user) {
        true => {},
        false => {return Err(status::BadRequest(Some(format!("Invalid Username or Passwor"))));}
    };


    let base = "user_files/".to_string() + user.username.as_str();

    let mut out: Vec<(String, bool)> = vec![];

    for file in read_dir(base).unwrap() {
        let name = match file {
            Ok(n) => {
                match n.file_name().to_str() {
                    Some(n) => n.to_string(),
                    None => {continue;},
                } 
            },
            Err(_) => {continue;},
        };

        let visible = users.find_user(&user.username).unwrap().settings.public.contains(&name);


        out.push((name, visible));
    }

    return Ok(Json(out));
}


#[rocket::post("/permissions/<file_name>/<visible>")]
fn set_file_permissions(state: &State<TUsers>, file_name: String, visible: bool, user: UserLogin) -> Result<status::Accepted<String>, status::BadRequest<String>> {

    println!("{}, {}", user.username, user.password);

    let mut users = match state.lock() {
        Ok(n) => n,
        Err(_) => {
            return Err(status::BadRequest(Some(format!("Internal Server Error"))));
        }
    };

    let user = NewUser {username: user.username.clone(), password: user.password};
    match users.verify_user(&user) {
        true => {},
        false => {
            return Err(status::BadRequest(Some(format!("Invalid Username or Password"))));
        }
    };

    match users.find_user(&user.username) {
        Some(mut n) => {
            let mut contains: bool = false;
            for i in 0..n.settings.public.len() {
                if file_name == n.settings.public[i] {
                    contains = true;
                    if !visible {
                        n.settings.public.remove(i);
                        break;
                    }
                }
            }

            if !contains && visible {
                n.settings.public.push(file_name);
            }

            users.update_user(n);

        }
        None => {
            return Err(status::BadRequest(Some(format!("Invalid Username or Password"))));
        }
    };

    Ok(status::Accepted(Some(format!("200 OK"))))

}

#[rocket::post("/delete/<file_name>")]
async fn delete_file(state: &State<TUsers>, file_name: String, user: UserLogin) -> Result<status::Accepted<String>, status::BadRequest<String>> {
    let verify = || -> bool {
        let users = match state.lock() {
            Ok(n) => n,
            Err(_) => {return false;}
        };

        let user = NewUser {username: user.username.clone(), password: user.password.clone()};
        return users.verify_user(&user);
    };

    match verify() {
        true => {},
        false => {
            return Err(status::BadRequest(Some(format!("Invalid Username or Password"))));
        },
    };

    let path = "user_files/".to_string() + user.username.as_str() + "/" + safe_path(file_name.clone()).as_str();

    match remove_file(path) {
        Ok(_) => {},
        Err(_) => {return Err(status::BadRequest(Some(format!("File Does not Exist"))));}
    };

    return set_file_permissions(state, file_name, false, user);

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

fn detect_client_still_connected(rx: Receiver<String>, users: TUsers, username: String) {
    loop {
        match rx.recv() {
            Ok(_) => {

            },
            Err(_) => {
                let mut users = users.lock().unwrap();
                let mut user = users.find_user(&username).unwrap();
                user.has_client = false;
                users.update_user(user);

                println!("client disconnected");
                break;
            }
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
            .mount("/", rocket::routes![index, login, register, user, view, verify_user, delete_user, register_user, file_upload, file_link, file_link_public, get_file, stream, set_file_permissions, get_user_images, delete_file, client, has_client])
            .attach(Template::fairing())
            .manage(users)
            .launch()
            .await;
        });
}