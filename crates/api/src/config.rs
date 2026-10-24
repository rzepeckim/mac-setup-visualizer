use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            port: env::var("PORT")
                .ok()
                .map(|p| p.parse().expect("PORT must be a valid u16"))
                .unwrap_or(3000),
        }
    }
}
