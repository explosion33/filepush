use std::path::{Path, PathBuf};

use rocket::{
    self,
    Config,
    fs::NamedFile,
    response::stream::{Event, EventStream},
    tokio::time::{self, Duration},
};

use rocket_dyn_templates::Template;

#[rocket::get("/event")]
fn index() -> Template {

    Template::render("event", rocket_dyn_templates::context!{})
}

#[rocket::get("/login")]
fn login() -> Template {

    Template::render("login", rocket_dyn_templates::context!{})
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

#[rocket::get("/static/<file>")]
async fn get_file(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("public/").join(file)).await.ok()
}


pub fn start_api() {
    rocket::tokio::runtime::Builder::new_multi_thread()
        .worker_threads(Config::from(Config::figment()).workers)
        // NOTE: graceful shutdown depends on the "rocket-worker" prefix.
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .expect("create tokio runtime")
        .block_on(async move {
            let _ = rocket::build()
            .mount("/", rocket::routes![index, login, get_file, stream])
            .attach(Template::fairing())
            //.manage()
            .launch()
            .await;
        });
}