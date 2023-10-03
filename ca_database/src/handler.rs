use crate::client::PostgresClient;
use anyhow::{Error};
use tokio_postgres::{Row};
use crate::types::{Article, FromRow};
use log::*;

pub struct PostgresHandler {
  pub client: PostgresClient
}

impl PostgresHandler {
  pub fn new(client: PostgresClient) -> Self {
    Self {
      client
    }
  }

  pub async fn get_articles(&self) -> Result<Vec<Article>, Error> {
    let rows = self
      .client
      .articles()
      .await
      .expect("Failed to get articles");
    // convert each Row to TestData
    let articles = rows
      .iter()
      .map(|row: &Row| {
        Article::from_row(row).expect("Failed to convert Row to Article")
      })
      .collect::<Vec<Article>>();
    Ok(articles)
  }

  pub async fn upsert_article(&self, key: &[u8], data: &[u8]) -> Result<Vec<Article>, Error> {
    let rows = self
      .client
      .article_upsert(key, data)
      .await
      .expect("Failed to upsert article");
    // convert each Row to TestData
    let articles = rows
      .iter()
      .map(|row: &Row| {
        Article::from_row(row).expect("Failed to convert Row to Article")
      })
      .collect::<Vec<Article>>();
    Ok(articles)
  }
}