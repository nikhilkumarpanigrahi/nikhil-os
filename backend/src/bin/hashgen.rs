//! Generates an argon2id hash for the `ADMIN_PASSWORD_HASH` env var.
//!
//! Usage:
//!   cargo run --bin hashgen                     # prompts for the password
//!   cargo run --bin hashgen "my-password"       # or pass it as an argument
//!
//! The printed value goes into backend/.env as ADMIN_PASSWORD_HASH.

use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHasher};

fn main() {
    let password = std::env::args().nth(1).unwrap_or_else(|| {
        eprint!("Enter admin password: ");
        std::io::Write::flush(&mut std::io::stdout()).expect("flush stdout");
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .expect("read password");
        line.trim().to_string()
    });

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("argon2 hashing should not fail")
        .to_string();
    println!("{hash}");
}
