use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Default)]
pub struct MessageHasher(pub DefaultHasher);

pub trait MessageHasherTrait {
    fn new() -> Self;
    fn finish(&mut self) -> u64;
    fn hash_article(&mut self, article: &str) -> u64;
    fn hash_calibration(&mut self, title: &str, calibration: u32) -> u64;
    fn hash_testimonial(&mut self, image_url: &str, testimonial: &str) -> u64;
    fn hash_string(&mut self, string: &str) -> u64;
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

    fn hash_calibration(&mut self, title: &str, calibration: u32) -> u64 {
        self.0 = DefaultHasher::new();
        title.hash(&mut self.0);
        calibration.hash(&mut self.0);
        self.finish()
    }

    fn hash_testimonial(&mut self, image_url: &str, testimonial: &str) -> u64 {
        self.0 = DefaultHasher::new();
        image_url.hash(&mut self.0);
        testimonial.hash(&mut self.0);
        self.finish()
    }

    fn hash_string(&mut self, string: &str) -> u64 {
        self.0 = DefaultHasher::new();
        string.hash(&mut self.0);
        self.finish()
    }
}
