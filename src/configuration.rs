use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
  pub database: DatabaseSettings,
  pub application_port: u16
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
  pub username: String,
  pub password: Secret<String>,
  pub port: u16,
  pub host: String,
  pub database_name: String,
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
  // Init Configuration Reader
  let settings = config::Config::builder()
  // Add values from files configuration.yaml
    .add_source(config::File::new("configuration.yaml", config::FileFormat::Yaml))
      .build()?;
  // Try convertion from configuration values into Settings type
  settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
  pub fn connection_string(&self) -> Secret<String> {
    Secret::new(format!(
      "postgres://{}:{}@{}:{}/{}",
      self.username, self.password.expose_secret(), self.host, self.port, self.database_name
    ))
  }
  pub fn conection_string_without_db(&self) -> Secret<String> {
    Secret::new(format!(
      "postgres://{}:{}@{}:{}",
      self.username, self.password.expose_secret(), self.host, self.port
    ))
  }
}