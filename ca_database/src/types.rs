use serde::{Deserialize, Serialize};
use tokio_postgres::{Row};
use anyhow::{Error};

pub trait FromRow<T> {
  fn from_row(row: &Row) -> Result<T, Error>;
}

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct Article {
  pub key: u64,
  pub data: String,
}

impl FromRow<Article> for Article {
  fn from_row(row: &Row) -> Result<Article, Error> {
    let row_key: Vec<u8> = row.get("key");
    let row_data: Vec<u8> = row.get("data");

    let key = bincode::deserialize(&row_key).expect("Failed to deserialize article key");
    let data = bincode::deserialize(&row_data).expect("Failed to deserialize article data");

    Ok(Article {
      key,
      data,
    })
  }
}