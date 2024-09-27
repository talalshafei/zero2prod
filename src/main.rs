use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::startup::Application;

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
    

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}