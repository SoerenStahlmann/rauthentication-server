# Rust Authentication Server

This repository contains the code for a Rust-based authentication server built using the [Rocket](https://rocket.rs/) web framework. It serves as the accompanying code for my Medium article where I walk through the process of building an authentication server, highlighting key Rust concepts like dependency injection, lifetimes, and async programming.

## Why This Repository Exists

I created this repository as a learning project to explore building an authentication server in Rust. The goal is to demonstrate how to implement various authentication strategies such as BasicAuth, API Token, and JWT in a clean and idiomatic way using dependency injection.

## What This Repository Contains

- **Authentication Strategies**: Multiple authentication methods such as BasicAuth, with the flexibility to add others like API Token and JWT.
- **User Repository**: A user repository pattern that can switch between a global in-memory store (for development) and a transactional database like PostgreSQL (for production).
- **Request Guards**: Request guards to handle secure routes and validate authenticated users.
- **Async Programming**: Uses async functions and the `async_trait` crate for working with asynchronous code in Rust.

## Article on Medium

To learn more about how this authentication server was built, check out the accompanying article on Medium:
[Building an Authentication Server in Rust with Rocket](#)

---

Feel free to clone the repository and explore the code. Feedback and contributions are welcome!

## Running the Project

To run the server locally:

1. Clone this repository
2. Install Rust and Cargo if not already installed
3. Run the project:
   ```bash
   cargo run
   ````

