use api::{app, config::Config};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();

    let pool = PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("migrations failed");

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port))
        .await
        .expect("failed to bind port");
    tracing::info!("listening on {}", config.port);

    axum::serve(listener, app(pool))
        .await
        .expect("server error");
}
