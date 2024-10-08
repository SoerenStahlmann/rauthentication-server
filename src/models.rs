use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub email: String,
    pub password: String,
}

impl User {
    pub fn hash_password(&mut self) {
        self.password = bcrypt::hash(&self.password, bcrypt::DEFAULT_COST).unwrap();
    }

    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password).unwrap()
    }
}
