// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares from having to specify the `#[test]` attribute.
//
// One can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)

use std::{net::TcpListener, sync::Arc};
use sqlx::{PgConnection, Connection};
use zero2prod::configuration::get_config;

// Launch the application in the background
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::startup::run(listener).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}


#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
  // Arrange
  let app_address = spawn_app();
  let configuration = get_config().expect("Failed to read configuration.");
  let db_connection_string = configuration.database.connection_string();
  // The `Connection` trait MUST be in scope to invoke
  // `PgConnection::connect` - it is not an inherent method of the struct!
  let mut connection = PgConnection::connect(&db_connection_string)
    .await
    .expect("Failed to connect with Postgres");
  let client = reqwest::Client::new();

  // Act
  let body = "name=test%20decoy&email=i_am_a_test%40testing.com";
  let response = client
    .post(&format!("{}/subscriptions", &app_address))
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(body)
    .send()
    .await
    .expect("Failed o post name and email");

  // Assert
  assert_eq!(200, response.status().as_u16());

  let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
  .fetch_one(&mut connection)
  .await
  .expect("Failed to fetch saved ubscription.");

  assert_eq!(saved.email, "i_am_a_test@testing.com");
  assert_eq!(saved.name, "test decoy");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
  // Arrange
  let app_address = spawn_app();
  let client = reqwest::Client::new();
  let test_cases = vec![
    ("name=le%20guin", "missing the email"),
    ("email=ursula_le_guin%40gmail.com", "missing the name"),
    ("", "missing both name and email")
  ];

  for (invalid_body, error_message) in test_cases {
    // Act
    let response = client
    .post(&format!("{}/subscriptions", &app_address))
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(invalid_body)
    .send()
    .await
    .expect("Failed to execute request.");

    // Assert
    assert_eq!(
      400,
      response.status().as_u16(),
      // Additional customised error message on test failure
      "The API did not fail with 400 Bad Request when the payload was {}.",
      error_message
    );
  }
}