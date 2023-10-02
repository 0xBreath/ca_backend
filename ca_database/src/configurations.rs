use std::path::PathBuf;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: Option<String>,
    pub password: Option<String>,
    pub port: Option<u16>,
    pub host: Option<String>,
    pub connection_string: Option<String>,
    pub database_name: Option<String>,
    pub threads: Option<usize>,
    pub batch_size: Option<usize>,
    pub panic_on_db_error: Option<bool>,
}

pub fn get_configuration() -> Result<DatabaseSettings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Could not get current directory");
    let geyser_pg_path = base_path.join("..");
    let conf_directory = geyser_pg_path.join("configuration");

    let settings = config::Config::builder()
        .add_source(config::File::from(conf_directory.join("base.yaml")))
        .build()?;

    settings.try_deserialize::<DatabaseSettings>()
}

impl DatabaseSettings {
    pub fn new_with_config_path(
        config_path: PathBuf,
    ) -> Result<DatabaseSettings, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::from(config_path))
            .build()?;

        settings.try_deserialize::<DatabaseSettings>()
    }

    pub fn new_from_url(connection_url: String) -> Result<DatabaseSettings, config::ConfigError> {
        let settings = config::Config::builder()
            .set_default("connection_string", connection_url)?
            .set_default("database_name", "ca-default")?
            .build()?;
        settings.try_deserialize::<DatabaseSettings>()
    }
}
