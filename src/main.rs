use std::net::TcpListener;
use sqlx::PgPool;
use zero2prod::{configuration::get_config, startup::run};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Setup Logger to imrpove Observability
  let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
  init_subscriber(subscriber);
  // Panic if we can't read configuration
  let configuration = get_config().expect("Failed to read configuration.");
  // Get port from config
  let connection_pool = PgPool::connect(&configuration.database.connection_string())
    .await
    .expect("Failed to connect to Postgres.");
  let address = format!("127.0.0.1:{}", configuration.application_port);
  // Bind address from config
  let listener = TcpListener::bind(address).expect("Failed to bind address.");
  run(listener, connection_pool)?.await
}
