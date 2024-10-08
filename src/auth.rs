use base64::prelude::*;
use log::info;
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::{self, status, Responder, Response};
use rocket::Request;
use std::io::Cursor;
use std::sync::Arc;

use crate::models::User;
use crate::repositroy::UserRepo;

// AuthResponse is a custom Responder that returns a response with a custom status, message, and Authorization header
pub struct AuthResponse {
    pub status: Status,
    pub message: String,
    pub auth_header: String,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for AuthResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        let response = Response::build()
            .status(self.status)
            .header(Header::new("Authorization", self.auth_header))
            .sized_body(None, Cursor::new(self.message))
            .finalize();

        Ok(response)
    }
}

// Request guard for extracting the User from the Authorization header
#[derive(Debug)]
pub enum AuthError {
    MissingAuthHeader,
    InvalidData,
    Unauthorized,
    MissingAuthStrategy,
    InternalServerError,
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    // Authenticate during login, returning a response with appropriate headers (like Authorization).
    async fn authenticate<'r>(
        &self,
        user: User,
        db_repo: &Arc<dyn UserRepo>,
    ) -> Result<AuthResponse, status::Custom<String>>;

    // Verify the authorization header in subsequent requests.
    async fn verify<'r>(&self, request: &'r Request<'_>) -> Result<User, AuthError>;
}

// BASIC AUTHENTICATION STRATEGY
pub struct BasicAuth;

#[rocket::async_trait]
impl Authenticator for BasicAuth {
    async fn authenticate<'r>(
        &self,
        user: User,
        db_repo: &Arc<dyn UserRepo>,
    ) -> Result<AuthResponse, status::Custom<String>> {
        // Retrieve the user from the Database
        let stored_user = match db_repo.get_user(&user.email).await {
            Ok(user) => user,
            Err(e) => {
                return Err(status::Custom(
                    Status::NotFound,
                    format!("Error logging in: {}", e),
                ))
            }
        };

        // Verify the password
        if !stored_user.verify_password(&user.password) {
            return Err(status::Custom(
                Status::Unauthorized,
                "Invalid password!".to_string(),
            ));
        }

        // Encode email and password in Base64
        let credentials = format!("{}:{}", user.email, user.password);
        let encoded_credentials = BASE64_STANDARD.encode(credentials);

        // Create and return an AuthResponse
        let response = AuthResponse {
            status: Status::Ok,
            message: format!("Welcome back, {}!", user.email),
            auth_header: format!("Basic {}", encoded_credentials),
        };

        return Ok(response);
    }

    async fn verify<'r>(&self, request: &'r Request<'_>) -> Result<User, AuthError> {
        // Extract the Authorization header from the request
        let auth_header = match request.headers().get_one("Authorization") {
            Some(header) => header,
            None => {
                info!("No Auth header found.");
                return Err(AuthError::MissingAuthHeader);
            }
        };

        // Split the Authorization header into its parts
        let auth_parts: Vec<&str> = auth_header.split_whitespace().collect();
        if auth_parts.len() != 2 {
            info!("Authorization header is not in the correct format.");
            return Err(AuthError::MissingAuthHeader);
        }

        // Decode the base64 encoded Authorization header
        let binding = BASE64_STANDARD
            .decode(auth_parts[1])
            .unwrap()
            .iter()
            .map(|&c| c as char)
            .collect::<String>();
        let user_pass: Vec<&str> = binding.split(':').collect();

        // Ensure the Authorization header contains a username and password
        if user_pass.len() != 2 {
            info!("Authorization header does not contain a username and password.");
            return Err(AuthError::InvalidData);
        }

        // Create a User struct from the Authorization header
        let user = User {
            email: user_pass[0].to_string(),
            password: user_pass[1].to_string(),
        };

        let db_repo = match request.rocket().state::<Arc<dyn UserRepo>>() {
            Some(repo) => repo,
            None => {
                info!("No UserRepo found in request state.");
                return Err(AuthError::InternalServerError);
            }
        };

        // Check if the user exists in the database
        let db_user = match db_repo.get_user(&user.email).await {
            Ok(user) => user,
            Err(_) => {
                info!("Could not find user in database.");
                return Err(AuthError::Unauthorized);
            }
        };

        // Verify the user's password
        if !db_user.verify_password(&user.password) {
            info!("Password does not match.");
            return Err(AuthError::Unauthorized);
        }

        Ok(user)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AuthError;
    async fn from_request(request: &'r Request<'_>) -> Outcome<User, AuthError> {
        // Get Authenticator stategy from request state through dependency injection
        let authenticator: &Arc<dyn Authenticator> =
            match request.rocket().state::<Arc<dyn Authenticator>>() {
                Some(auth) => auth,
                None => {
                    return Outcome::Error((
                        Status::InternalServerError,
                        AuthError::MissingAuthStrategy,
                    ));
                }
            };

        match authenticator.verify(request).await {
            Ok(user) => Outcome::Success(user),
            Err(AuthError::MissingAuthHeader) => {
                Outcome::Error((Status::Unauthorized, AuthError::MissingAuthHeader))
            }
            Err(AuthError::InvalidData) => {
                Outcome::Error((Status::Unauthorized, AuthError::InvalidData))
            }
            Err(AuthError::Unauthorized) => {
                Outcome::Error((Status::Unauthorized, AuthError::Unauthorized))
            }
            Err(AuthError::InternalServerError) => {
                Outcome::Error((Status::InternalServerError, AuthError::InternalServerError))
            }
            Err(AuthError::MissingAuthStrategy) => {
                Outcome::Error((Status::InternalServerError, AuthError::MissingAuthStrategy))
            }
        }
    }
}
