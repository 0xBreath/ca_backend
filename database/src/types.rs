use crate::{MessageHasher, MessageHasherTrait};
use anyhow::Error;
use serde::{Deserialize, Serialize};

// ==================== Article ====================

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub tags: Vec<String>,
    pub data: String,
    pub image_url: String,
    pub index: u32,
    pub premium: bool,
}

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct DbArticle {
    pub key: u64,
    pub value: Vec<u8>,
}

impl Article {
    pub fn de(article: &[u8]) -> Result<Article, Error> {
        let article =
            bincode::deserialize::<Article>(article).expect("Failed to deserialize article");

        Ok(Article {
            title: article.title,
            tags: article.tags,
            data: article.data,
            image_url: article.image_url,
            index: article.index,
            premium: article.premium,
        })
    }

    pub fn ser(&self) -> Result<DbArticle, Error> {
        let key = MessageHasher::new().hash_article(&self.data);
        let value = bincode::serialize(&self).expect("Failed to serialize article");

        Ok(DbArticle { key, value })
    }
}

// ==================== Calibration ====================

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct Calibration {
    pub title: String,
    pub calibration: u32,
    pub tags: Vec<String>,
    pub image_url: String,
    pub description: String,
}

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct DbCalibration {
    pub key: u64,
    pub value: Vec<u8>,
}

impl Calibration {
    pub fn de(calibration: &[u8]) -> Result<Calibration, Error> {
        let calibration = bincode::deserialize::<Calibration>(calibration)
            .expect("Failed to deserialize calibration");

        Ok(Calibration {
            title: calibration.title,
            calibration: calibration.calibration,
            tags: calibration.tags,
            image_url: calibration.image_url,
            description: calibration.description,
        })
    }

    pub fn ser(&self) -> Result<DbCalibration, Error> {
        let key = MessageHasher::new().hash_calibration(&self.title, self.calibration);
        let value = bincode::serialize(&self).expect("Failed to serialize calibration");

        Ok(DbCalibration { key, value })
    }
}

// ==================== Testimonial ====================

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct Testimonial {
    pub image_url: String,
    pub testimonial: String,
}

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct DbTestimonial {
    pub key: u64,
    pub value: Vec<u8>,
}

impl Testimonial {
    pub fn de(testimonial: &[u8]) -> Result<Testimonial, Error> {
        let testimonial = bincode::deserialize::<Testimonial>(testimonial)
            .expect("Failed to deserialize testimonial");

        Ok(Testimonial {
            image_url: testimonial.image_url,
            testimonial: testimonial.testimonial,
        })
    }

    pub fn ser(&self) -> Result<DbTestimonial, Error> {
        let key = MessageHasher::new().hash_testimonial(&self.image_url, &self.testimonial);
        let value = bincode::serialize(&self).expect("Failed to serialize testimonial");

        Ok(DbTestimonial { key, value })
    }
}
