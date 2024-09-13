use std::net::TcpListener;
use sqlx::PgPool;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use env_logger::Env;


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    
    let connection = PgPool::connect(
        &configuration.database.connection_string()
    ).await.expect("Failed to connect to Postgres");

    // We have removed the hard-coded `8000` - it's now coming from our settings!
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}