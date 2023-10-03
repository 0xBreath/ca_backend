use crate::client::PostgresClient;
use anyhow::{Error};
use log::info;
use tokio_postgres::{Row};
use crate::types::{Article};

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
      .await?;
    info!("get articles rows: {}", rows.len());
    let articles = rows_to_articles(rows)?;
    Ok(articles)
  }

  pub async fn upsert_article(&self, article: Article) -> Result<Vec<Article>, Error> {
    let db_article = article.to_postgres()?;

    let rows = self
      .client
      .article_upsert(db_article)
      .await?;
    let articles = rows_to_articles(rows)?;
    Ok(articles)
  }
}

fn rows_to_articles(rows: Vec<Row>) -> Result<Vec<Article>, Error> {
  // convert each Row to TestData
  let articles = rows
    .iter()
    .map(|row: &Row| {
      Article::from_row(row).expect("Failed to convert Row to Article")
    })
    .collect::<Vec<Article>>();
  Ok(articles)
}