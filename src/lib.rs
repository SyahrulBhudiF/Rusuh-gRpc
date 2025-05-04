pub mod config {
    pub mod db;
    pub mod redis;
}

pub mod core {
    pub mod server;
}

pub mod domain {
    pub mod entity {
        pub mod user;
    }
    pub mod jwt;
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

pub mod pb {
    pub mod auth;
}

pub mod service {
    pub mod auth_service;
}
