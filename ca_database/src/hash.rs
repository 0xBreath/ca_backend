use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};


#[derive(Debug, Default)]
pub struct MessageHasher(pub DefaultHasher);

pub trait MessageHasherTrait {
  fn new() -> Self;
  fn finish(&mut self) -> u64;
  fn hash_article(&mut self, article: &str) -> u64;
}

impl MessageHasherTrait for MessageHasher {
  fn new() -> Self {
    Self(DefaultHasher::new())
  }

  /// Reset contents of hasher for reuse
  fn finish(&mut self) -> u64 {
    self.0.finish()
  }

  fn hash_article(&mut self, article_markdown: &str) -> u64 {
    self.0 = DefaultHasher::new();
    article_markdown.hash(&mut self.0);
    self.finish()
  }
}