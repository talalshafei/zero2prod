use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use sqlx::PgPool;
use std::net::TcpListener;
use secrecy::ExposeSecret;


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    
    init_subscriber(
        get_subscriber(
            "zero2prod".into(),
            "info".into(),
            std::io::stdout
    ));

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    
    let connection = PgPool::connect_lazy(
        &configuration.database.connection_string().expose_secret()
    )
    .expect("Failed to connect to Postgres");

    let address = format!(
        "{}:{}", 
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}