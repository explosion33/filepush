use crate::routes::start_api;
mod routes;

mod passwords;
mod users;

fn main() {
    start_api();
}