use crate::configurations::DatabaseSettings;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::*;
use serde::{Deserialize, Serialize};
use tokio_postgres::*;

struct PostgresClientWrapper {
    client: Client,
    articles_statement: Statement,
}

pub struct SimplePostgresClient {
    client: PostgresClientWrapper,
}

impl Eq for TestData {}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TestData {
    pub key: i64,
    pub data: Vec<u8>,
}

impl TestData {
    pub fn new(key: i64, data: &[u8]) -> Self {
        Self {
            key,
            data: data.to_vec(),
        }
    }

    pub fn from_row(row: &Row) -> Result<Self, Error> {
        Ok(Self {
            key: row.get("key"),
            data: row.get("data"),
        })
    }
}

#[async_trait]
pub trait PostgresClient {
    async fn articles(&self) -> Result<Vec<Row>, Error>;
}

impl SimplePostgresClient {
    pub async fn new(config: &DatabaseSettings) -> Result<Self, Error> {
        let pool = Self::connect_to_db(config).await?;
        let client = pool.dedicated_connection().await?;
        let articles_statement =
          Self::build_articles_statement(&client, config).await?;

        info!("Created SimplePostgresClient");

        Ok(Self {
            client: PostgresClientWrapper {
                client,
                articles_statement,
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

    async fn build_articles_statement(
        client: &Client,
        config: &DatabaseSettings,
    ) -> Result<Statement, Error> {
        let stmt = include_str!("../prepared_statements/articles.sql");
        let stmt = client.prepare(stmt).await;

        match stmt {
            Ok(stmt) => Ok(stmt),
            Err(error) => {
                let error = anyhow::anyhow!(
                    "Failed to prepare articles statement: {} host: {:?}, user: {:?}, config{:?}",
                    error,
                    config.host,
                    config.username,
                    config
                );
                Err(error)
            }
        }
    }

    async fn articles(&self) -> Result<Vec<Row>, Error> {
        let client = &self.client;
        let statement = &client.articles_statement;
        let client = &client.client;
        let result = client.query(statement, &[]).await;
        result.map_err(|err| anyhow!("Failed to get articles: {}", err))
    }
}

#[async_trait]
impl PostgresClient for SimplePostgresClient {
    async fn articles(&self) -> Result<Vec<Row>, Error> {
        self.articles().await
    }
}

pub struct PostgresClientBuilder {}

impl PostgresClientBuilder {
    pub async fn build_simple_postgres_client(
        config: &DatabaseSettings,
    ) -> Result<SimplePostgresClient, Error> {
        SimplePostgresClient::new(config).await
    }
}
