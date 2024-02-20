// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares from having to specify the `#[test]` attribute.
//
// One can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)

use std::net::TcpListener;
use actix_web::web::get;
use sqlx::{Connection, PgConnection, PgPool};
use zero2prod::configuration::get_config;

pub struct TestApp {
  pub address: String,
  pub db_pool: PgPool
}
// Launch the application in the background
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address =     format!("http://127.0.0.1:{}", port);

    let configuration = get_config().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
      .await
      .expect("Failed to connect to Postgres.");
    let server = zero2prod::startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
    TestApp {
      address,
      db_pool: connection_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &app.address))
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
  let app = spawn_app().await;
  let client = reqwest::Client::new();

  // Act
  let body = "name=test%20decoy&email=i_am_a_test%40testing.com";
  let response = client
    .post(&format!("{}/subscriptions", &app.address))
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(body)
    .send()
    .await
    .expect("Failed o post name and email");

  // Assert
  assert_eq!(200, response.status().as_u16());

  let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
  .fetch_one(&app.db_pool)
  .await
  .expect("Failed to fetch saved ubscription.");

  assert_eq!(saved.email, "i_am_a_test@testing.com");
  assert_eq!(saved.name, "test decoy");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
  // Arrange
  let app = spawn_app().await;
  let client = reqwest::Client::new();
  let test_cases = vec![
    ("name=le%20guin", "missing the email"),
    ("email=ursula_le_guin%40gmail.com", "missing the name"),
    ("", "missing both name and email")
  ];

  for (invalid_body, error_message) in test_cases {
    // Act
    let response = client
    .post(&format!("{}/subscriptions", &app.address))
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