#[derive(serde::Deserialize)]
pub struct Settings {
  pub database: DatabaseSettings,
  pub application_port: u16
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
  pub username: String,
  pub password: String,
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
  pub fn connection_string(&self) -> String {
    format!(
      "postgres://{}:{}@{}:{}/{}",
      self.username, self.password, self.host, self.port, self.database_name
    )
  }
}