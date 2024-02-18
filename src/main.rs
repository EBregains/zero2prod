use std::net::TcpListener;
use zero2prod::{configuration::get_config, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Panic if we can't read configuration
  let configuration = get_config().expect("Failed to read configuration.");
  // Get port from config
  let address = format!("127.0.0.1:{}", configuration.application_port);
  // Bind address from config
  let listener = TcpListener::bind(address).expect("Failed to bind address.");
    run(listener)?.await
}
