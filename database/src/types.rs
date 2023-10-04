use serde::{Deserialize, Serialize};
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
  pub key: u64,
  pub value: Vec<u8>,
}

impl Article {
  pub fn de(article: &[u8]) -> Result<Article, Error> {
    let article = bincode::deserialize::<Article>(article).expect("Failed to deserialize article");

    Ok(Article {
      title: article.title,
      data: article.data,
      image_url: article.image_url,
    })
  }

  pub fn ser(&self) -> Result<DbArticle, Error> {
    let key = MessageHasher::new().hash_article(&self.data);
    let value = bincode::serialize(&self).expect("Failed to serialize article");

    Ok(DbArticle {
      key,
      value
    })
  }
}