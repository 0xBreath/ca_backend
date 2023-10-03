use crate::settings::DatabaseSettings;
use anyhow::{anyhow, Error};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::*;
use tokio_postgres::*;
use crate::StatementBuilder;

struct PostgresClientWrapper {
    client: Client,

    articles_statement: Statement,
    article_upsert_statement: Statement,
    article_delete_statement: Statement,
    article_statement: Statement,

    courses_statement: Statement,
    course_upsert_statement: Statement,
    course_delete_statement: Statement,
    course_statement: Statement,
}

pub struct PostgresClient {
    client: PostgresClientWrapper,
}

impl PostgresClient {
    pub async fn new_from_url(connection_url: String) -> Result<Self, Error> {
        let client = match DatabaseSettings::new_from_url(connection_url) {
            Err(err) => return Err(anyhow!("Error loading configuration: {}", err)),
            Ok(config) => PostgresClient::new(&config).await?,
        };
        Ok(client)
    }

    pub async fn new(config: &DatabaseSettings) -> Result<Self, Error> {
        let pool = Self::connect_to_db(config).await?;
        let client = pool.dedicated_connection().await?;

        let articles_statement =
          StatementBuilder::articles_statement(&client, config).await?;
        let article_upsert_statement = StatementBuilder::article_upsert_statement(&client, config).await?;
        let article_delete_statement = StatementBuilder::article_delete_statement(&client, config).await?;
        let article_statement = StatementBuilder::article_statement(&client, config).await?;

        let courses_statement =
          StatementBuilder::courses_statement(&client, config).await?;
        let course_upsert_statement = StatementBuilder::course_upsert_statement(&client, config).await?;
        let course_delete_statement = StatementBuilder::course_delete_statement(&client, config).await?;
        let course_statement = StatementBuilder::course_statement(&client, config).await?;


        info!("Created SimplePostgresClient");
        Ok(Self {
            client: PostgresClientWrapper {
                client,
                articles_statement,
                article_upsert_statement,
                article_delete_statement,
                article_statement,
                courses_statement,
                course_upsert_statement,
                course_delete_statement,
                course_statement,
            },
        })
    }

    async fn connect_to_db(
        config: &DatabaseSettings,
    ) -> Result<Pool<PostgresConnectionManager<NoTls>>, Error> {
        let connection_string = if let Some(connection_string) = &config.connection_string {
            connection_string.clone()
        } else {
            if config.host.is_none() || config.username.is_none() {
                let error = anyhow::anyhow!("Missing host or username in database configuration");
                return Err(error);
            }
            if config.database_name.is_none() {
                format!(
                    "host={} user={} password={} port={}",
                    config.host.as_ref().unwrap(),
                    config.username.as_ref().unwrap(),
                    config.password.as_ref().unwrap(),
                    config.port.unwrap_or(5432)
                )
            } else {
                format!(
                    "host={} user={} password={} port={} dbname={}",
                    config.host.as_ref().unwrap(),
                    config.username.as_ref().unwrap(),
                    config.password.as_ref().unwrap(),
                    config.port.unwrap_or(5432),
                    config.database_name.as_ref().unwrap()
                )
            }
        };

        let config = connection_string.parse::<Config>()?;
        let manager = PostgresConnectionManager::new(config, NoTls);
        let pool = Pool::builder().build(manager).await?;

        Ok(pool)
    }

    // ================ Articles =================

    pub async fn articles(&self) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.articles_statement;
        let client = &client.client;
        let result = client.query(statement, &[]).await;
        result.map_err(|err| anyhow!("Failed to get articles: {}", err))
    }

    pub async fn article(&self) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.article_statement;
        let client = &client.client;
        let result = client.query(statement, &[]).await;
        result.map_err(|err| anyhow!("Failed to get article: {}", err))
    }

    pub async fn article_upsert(&self, key: &[u8], data: &[u8]) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.article_upsert_statement;
        let client = &client.client;
        let result = client.query(statement, &[&key, &data]).await;
        result.map_err(|err| anyhow!("Failed to upsert article: {}", err))
    }

    pub async fn article_delete(&self, key: String, data: &[u8]) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.article_delete_statement;
        let client = &client.client;
        let result = client.query(statement, &[&key, &data]).await;
        result.map_err(|err| anyhow!("Failed to delete article: {}", err))
    }

    // ================ Courses =================

    pub async fn courses(&self) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.courses_statement;
        let client = &client.client;
        let result = client.query(statement, &[]).await;
        result.map_err(|err| anyhow!("Failed to get articles: {}", err))
    }

    pub async fn course(&self) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.course_statement;
        let client = &client.client;
        let result = client.query(statement, &[]).await;
        result.map_err(|err| anyhow!("Failed to get course: {}", err))
    }

    pub async fn course_upsert(&self, key: String, data: &[u8]) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.course_upsert_statement;
        let client = &client.client;
        let result = client.query(statement, &[&key, &data]).await;
        result.map_err(|err| anyhow!("Failed to upsert course: {}", err))
    }

    pub async fn course_delete(&self, key: String, data: &[u8]) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.course_delete_statement;
        let client = &client.client;
        let result = client.query(statement, &[&key, &data]).await;
        result.map_err(|err| anyhow!("Failed to delete course: {}", err))
    }
}
