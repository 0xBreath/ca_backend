use serde::{Deserialize, Serialize};
use tokio_postgres::{Row};
use anyhow::{Error};
use crate::{MessageHasher, MessageHasherTrait};


#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct Article {
  pub title: String,
  pub data: String,
  pub image_url: String,
}

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct DbArticle {
  pub key: Vec<u8>,
  pub title: Vec<u8>,
  pub data: Vec<u8>,
  pub image_url: Vec<u8>,
}

impl Article {
  pub fn from_row(row: &Row) -> Result<Article, Error> {
    let row_key: Vec<u8> = row.get("key");
    let row_title: Vec<u8> = row.get("title");
    let row_data: Vec<u8> = row.get("data");
    let row_image_url: Vec<u8> = row.get("image_url");

    let title = bincode::deserialize(&row_title).expect("Failed to deserialize article title");
    let data = bincode::deserialize(&row_data).expect("Failed to deserialize article data");
    let image_url = bincode::deserialize(&row_image_url).expect("Failed to deserialize article image_url");

    Ok(Article {
      title,
      data,
      image_url,
    })
  }

  pub fn to_postgres(&self) -> Result<DbArticle, Error> {
    let hash = MessageHasher::new().hash_article(&self.data);
    let key = bincode::serialize(&hash).expect("Failed to serialize article key");
    let title = bincode::serialize(&self.title).expect("Failed to serialize article title");
    let data = bincode::serialize(&self.data).expect("Failed to serialize article data");
    let image_url = bincode::serialize(&self.image_url).expect("Failed to serialize article image_url");

    Ok(DbArticle {
      key,
      title,
      data,
      image_url,
    })
  }
}