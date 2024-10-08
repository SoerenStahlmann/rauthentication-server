use rocket::{http::Status, response::status, serde::json::Json, State};
use std::sync::Arc;

use crate::{
    auth::{AuthResponse, Authenticator},
    models::User,
    repositroy::UserRepo,
};

#[post("/signup", data = "<user>")]
pub async fn signup(
    user: Json<User>,
    db_repo: &State<Arc<dyn UserRepo>>,
) -> Result<String, status::Custom<String>> {
    let mut new_user = user.into_inner(); // Extract the User from the Json wrapper
    new_user.hash_password(); // Hash the password before storing it

    // Insert the new user into the global UserMap
    if db_repo.user_exists(&new_user.email).await {
        // Return 409 Conflict if the user already exists
        return Err(status::Custom(
            Status::Conflict,
            format!("User with email {} already exists!", new_user.email),
        ));
    }
    db_repo.add_user(new_user.clone()).await.map_err(|e| {
        // Return 500 Internal Server Error if there was an error adding the user
        status::Custom(
            Status::InternalServerError,
            format!("Error creating user: {}", e),
        )
    })?;

    Ok(format!(
        "Signed up user: {}\nWith password: {}",
        new_user.email, new_user.password
    ))
}

#[post("/login", data = "<user>")]
pub async fn login(
    user: Json<User>,
    db_repo: &State<Arc<dyn UserRepo>>,
    auth: &State<Arc<dyn Authenticator>>,
) -> Result<AuthResponse, status::Custom<String>> {
    let user = user.into_inner();

    // Authenticate the user
    auth.authenticate(user, db_repo).await
}

#[get("/authenticated")]
pub async fn authenticated(_user: User) -> &'static str {
    "You are authenticated!"
}
