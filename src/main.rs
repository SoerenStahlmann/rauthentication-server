mod auth;
mod models;
mod repositroy;
mod routes;

use std::sync::Arc;

use auth::{Authenticator, BasicAuth};
use repositroy::UserRepo;
use rocket_slogger::Slogger;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    let db_repo: Arc<dyn UserRepo> = Arc::new(repositroy::InMemoryUserRepo::new()); // In Memory User Repository
    let auth: Arc<dyn Authenticator> = Arc::new(BasicAuth); // Basic Auth Strategy

    let logger = Slogger::new_terminal_logger();

    rocket::build()
        .attach(logger)
        .manage(db_repo)
        .manage(auth)
        .mount(
            "/",
            routes![routes::signup, routes::login, routes::authenticated],
        )
}
