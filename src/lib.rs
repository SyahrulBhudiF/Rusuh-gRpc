pub mod config {
    pub mod db;
    pub mod env;
    pub mod redis;
}

pub mod core {
    pub mod server;
}

pub mod domain {
    pub mod entity {
        pub mod user;
    }
    pub mod redis_repository;
    pub mod repository;
}

pub mod handler {
    pub mod auth_handler;
}

pub mod infrastructure {
    pub mod redis_repository;
    pub mod user_repository;
}

pub mod interceptor {
    pub mod auth_interceptor;
}

pub mod pb {
    pub mod auth;
}

pub mod service {
    pub mod auth_service;
}

pub mod util {
    pub mod email;
    pub mod jwt;
    pub mod totp;
}

pub use config::env::cfg;
pub use util::email::email;
pub use util::totp::otp;
