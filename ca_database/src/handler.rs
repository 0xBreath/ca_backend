use crate::client::{
    TestData, PostgresClient, PostgresClientBuilder,
    SimplePostgresClient,
};
use crate::configurations::{get_configuration, DatabaseSettings};
use anyhow::{anyhow, Error};
use tokio_postgres::Row;

pub struct PostgresHandler {
    pub client: SimplePostgresClient,
}

impl PostgresHandler {
    pub async fn new() -> Result<Self, Error> {
        let client = match get_configuration() {
            Err(err) => return Err(anyhow!("Error loading configuration: {}", err)),
            Ok(config) => PostgresClientBuilder::build_simple_postgres_client(&config).await?,
        };
        Ok(Self { client })
    }

    pub async fn new_from_url(connection_url: String) -> Result<Self, Error> {
        let client = match DatabaseSettings::new_from_url(connection_url) {
            Err(err) => return Err(anyhow!("Error loading configuration: {}", err)),
            Ok(config) => PostgresClientBuilder::build_simple_postgres_client(&config).await?,
        };
        Ok(Self { client })
    }

    pub async fn get_articles(&self) -> Result<Vec<TestData>, Error> {
        let rows = self
          .client
          .articles()
          .await
          .expect("Failed to get articles");
        // convert each Row to TestData
        let articles = rows
          .iter()
          .map(|row: &Row| {
              TestData::from_row(row).expect("Failed to convert Row to TestData")
          })
          .collect::<Vec<TestData>>();
        Ok(articles)
    }
}
