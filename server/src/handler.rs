use crate::square::{
    CanceledSubscriptionInfo, CheckoutInfo, SquareClient, SquareResponse, UserEmailRequest,
    UserProfile,
};
use actix_web::{web, Result};
use database::{Article, Calibration, Testimonial};
use futures::StreamExt;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use tokio::sync::MutexGuard;

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Serialize, Deserialize, Debug)]
pub struct LoadState {
    pub content_type_images: Vec<String>,
    pub category_images: Vec<String>,
    pub articles: Vec<Article>,
    pub calibrations: Vec<Calibration>,
    pub testimonials: Vec<Testimonial>,
    pub testimonial_images: Vec<String>,
    pub subscribe_checkout: CheckoutInfo,
    pub user_profile: UserProfile,
}

pub struct ServerHandler<'a> {
    pub client: MutexGuard<'a, SquareClient>,
}

impl<'a> ServerHandler<'a> {
    pub fn new(client: MutexGuard<'a, SquareClient>) -> Self {
        ServerHandler { client }
    }

    /// Open all to all users
    pub fn handle_content_type_images() -> Result<Vec<String>> {
        let cache_path = std::env::current_dir()
          .unwrap()
          .to_str()
          .unwrap()
          .to_string()
          + "/cache/content_type_images.bin";

        let mut cache_file = File::open(&cache_path).expect("Failed to open content type images \
        cache");
        // Read the contents into a Vec<u8>
        let mut cache_buf = Vec::new();
        cache_file
          .read_to_end(&mut cache_buf)
          .expect("Failed to read content type images cache");

        let mut db_images = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf)
          .expect("Failed to read content type images cache");
        let mut images = Vec::new();

        for (_, db_image) in db_images.drain() {
            let image = bincode::deserialize::<String>(&db_image)
              .expect("Failed to deserialize content type image");
            images.push(image);
        }
        Ok(images)
    }

    /// Open all to all users
    pub fn handle_category_images() -> Result<Vec<String>> {
        let cache_path = std::env::current_dir()
          .unwrap()
          .to_str()
          .unwrap()
          .to_string()
          + "/cache/category_images.bin";

        let mut cache_file = File::open(&cache_path).expect("Failed to open category images \
        cache");
        // Read the contents into a Vec<u8>
        let mut cache_buf = Vec::new();
        cache_file
          .read_to_end(&mut cache_buf)
          .expect("Failed to read category images cache");

        let mut db_images = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf)
          .expect("Failed to read category images cache");
        let mut images = Vec::new();

        for (_, db_image) in db_images.drain() {
            let image = bincode::deserialize::<String>(&db_image)
              .expect("Failed to deserialize category image");
            images.push(image);
        }
        Ok(images)
    }

    pub fn handle_free_articles() -> Result<Vec<Article>> {
        let articles = Self::handle_articles()?;
        Ok(articles.into_iter().map(|mut article| {
            if article.premium {
                article.data = String::new();
            }
            article
        }).collect::<Vec<Article>>())
    }

    pub fn handle_articles() -> Result<Vec<Article>> {
        let cache_path = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/cache/articles.bin";

        let mut cache_file = File::open(&cache_path).expect("Failed to open articles cache");
        // Read the contents into a Vec<u8>
        let mut cache_buf = Vec::new();
        cache_file
            .read_to_end(&mut cache_buf)
            .expect("Failed to read articles cache");

        let mut db_articles = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf)
            .expect("Failed to read articles cache");
        let mut articles = Vec::new();
        // for each DbArticle in the hashmap, deserialize into Article and collect to vector
        for (_, db_article) in db_articles.drain() {
            let article = bincode::deserialize::<Article>(&db_article)
                .expect("Failed to deserialize article");
            articles.push(article);
        }
        Ok(articles)
    }

    /// Open all to all users
    pub fn handle_calibrations() -> Result<Vec<Calibration>> {
        let cache_path = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/cache/calibrations.bin";

        let mut cache_file = File::open(&cache_path).expect("Failed to open calibrations cache");
        // Read the contents into a Vec<u8>
        let mut cache_buf = Vec::new();
        cache_file
            .read_to_end(&mut cache_buf)
            .expect("Failed to read calibrations cache");

        let mut db_calibrations = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf)
            .expect("Failed to read calibrations cache");
        let mut calibrations = Vec::new();
        // for each DbCalibration in the hashmap, deserialize into Calibration and collect to vector
        for (_, db_calibration) in db_calibrations.drain() {
            let calibration = bincode::deserialize::<Calibration>(&db_calibration)
                .expect("Failed to deserialize calibration");
            calibrations.push(calibration);
        }
        Ok(calibrations)
    }

    /// Open all to all users
    pub fn handle_testimonials() -> Result<Vec<Testimonial>> {
        let cache_path = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/cache/testimonials.bin";

        let mut cache_file = File::open(&cache_path).expect("Failed to open testimonials cache");
        // Read the contents into a Vec<u8>
        let mut cache_buf = Vec::new();
        cache_file
            .read_to_end(&mut cache_buf)
            .expect("Failed to read testimonials cache");

        let mut db_testimonials = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf)
            .expect("Failed to read testimonials cache");
        let mut testimonials = Vec::new();
        // for each DbCalibration in the hashmap, deserialize into Calibration and collect to vector
        for (_, db_testimonial) in db_testimonials.drain() {
            let testimonial = bincode::deserialize::<Testimonial>(&db_testimonial)
                .expect("Failed to deserialize testimonial");
            testimonials.push(testimonial);
        }
        Ok(testimonials)
    }

    /// Open all to all users
    pub fn handle_testimonial_images() -> Result<Vec<String>> {
        let cache_path = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + "/cache/testimonial_images.bin";

        let mut cache_file = File::open(&cache_path)?;
        // Read the contents into a Vec<u8>
        let mut cache_buf = Vec::new();
        cache_file.read_to_end(&mut cache_buf)?;

        let mut db_images = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf)
            .expect("Failed to read testimonial images cache");
        let mut images = Vec::new();
        // for each DbArticle in the hashmap, deserialize into Article and collect to vector
        for (_, db_image) in db_images.drain() {
            let image = bincode::deserialize::<String>(&db_image)
                .expect("Failed to deserialize testimonial image");
            images.push(image);
        }
        Ok(images)
    }

    /// Restricted to authenticated request
    pub async fn handle_subscribe(&self, mut payload: web::Payload) -> Result<CheckoutInfo> {
        let mut body = web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > MAX_SIZE {
                return Err(actix_web::error::ErrorBadRequest(
                    "POST request bytes overflow",
                ));
            }
            body.extend_from_slice(&chunk);
        }

        let buyer_email = serde_json::from_slice::<UserEmailRequest>(&body)?;
        debug!("Checkout user email: {:?}", &buyer_email);
        let res = self.client.subscribe_checkout(Some(buyer_email)).await?;

        if let SquareResponse::Success(subscribe) = res {
            debug!("Subscription checkout: {:?}", &subscribe);
            Ok(subscribe)
        } else {
            error!("Failed to subscribe: {:?}", &res);
            Err(actix_web::error::ErrorBadRequest("Failed to subscribe"))
        }
    }

    /// Open to all users
    pub async fn handle_user_profile(&self, mut payload: web::Payload) -> Result<UserProfile> {
        let mut body = web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > MAX_SIZE {
                return Err(actix_web::error::ErrorBadRequest(
                    "POST request bytes overflow",
                ));
            }
            body.extend_from_slice(&chunk);
        }

        let buyer_email = serde_json::from_slice::<UserEmailRequest>(&body)?;
        debug!("User subscription request email: {:?}", &buyer_email);
        let info = self.client.get_user_profile(buyer_email).await?;
        debug!("Get user subscription info: {:?}", &info);
        Ok(info)
    }

    /// Restricted to authenticated request
    pub async fn handle_cancel_subscription(
        &self,
        mut payload: web::Payload,
    ) -> Result<CanceledSubscriptionInfo> {
        let mut body = web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > MAX_SIZE {
                return Err(actix_web::error::ErrorBadRequest(
                    "POST request bytes overflow",
                ));
            }
            body.extend_from_slice(&chunk);
        }

        let buyer_email = serde_json::from_slice::<UserEmailRequest>(&body)?;
        let info = self.client.cancel_subscription(buyer_email).await?;
        if let SquareResponse::Success(info) = info {
            Ok(info)
        } else {
            error!("Failed to cancel subscription: {:?}", &info);
            Err(actix_web::error::ErrorBadRequest(
                "Failed to cancel subscription",
            ))
        }
    }

    pub async fn load_free_state(&self) -> Result<LoadState> {
        let content_type_images = Self::handle_content_type_images()?;
        debug!("Fetched content type images");
        let category_images = Self::handle_category_images()?;
        debug!("Fetched category images");
        let articles = Self::handle_free_articles()?;
        debug!("Fetched articles");
        let calibrations = Self::handle_calibrations()?;
        debug!("Fetched calibrations");
        let testimonials = Self::handle_testimonials()?;
        debug!("Fetched testimonials");
        let testimonial_images = Self::handle_testimonial_images()?;
        debug!("Fetched testimonial images");
        let subscribe_checkout = match self.client.subscribe_checkout(None)
          .await? {
            SquareResponse::Success(subscribe) => subscribe,
            SquareResponse::Error(err) => {
                error!(
                    "Failed to fetch subscribe checkout in state dump: {:?}",
                    &err
                );
                return Err(actix_web::error::ErrorBadRequest(
                    "Failed to fetch subscribe checkout in
                    state dump",
                ));
            }
        };
        debug!("Fetched subscribe checkout");

        Ok(LoadState {
            content_type_images,
            category_images,
            articles,
            calibrations,
            testimonials,
            testimonial_images,
            subscribe_checkout,
            user_profile: Default::default(),
        })
    }

    /// Filtered responses if not subscribed
    pub async fn load_state(
        &self,
        mut payload: web::Payload,
    ) -> Result<LoadState> {
        let mut body = web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > MAX_SIZE {
                return Err(actix_web::error::ErrorBadRequest(
                    "POST request bytes overflow",
                ));
            }
            body.extend_from_slice(&chunk);
        }
        let user_email = serde_json::from_slice::<UserEmailRequest>(&body)?;
        let email = user_email.email.clone();

        let content_type_images = Self::handle_content_type_images()?;
        debug!("Fetched content type images");
        let category_images = Self::handle_category_images()?;
        debug!("Fetched category images");
        let articles = Self::handle_articles()?;
        debug!("Fetched articles");
        let calibrations = Self::handle_calibrations()?;
        debug!("Fetched calibrations");
        let testimonials = Self::handle_testimonials()?;
        debug!("Fetched testimonials");
        let testimonial_images = Self::handle_testimonial_images()?;
        debug!("Fetched testimonial images");
        let subscribe_checkout = match self.client.subscribe_checkout(Some(user_email.clone()))
          .await? {
            SquareResponse::Success(subscribe) => subscribe,
            SquareResponse::Error(err) => {
                error!(
                    "Failed to fetch subscribe checkout in state dump: {:?}",
                    &err
                );
                return Err(actix_web::error::ErrorBadRequest(
                    "Failed to fetch subscribe checkout in
                    state dump",
                ));
            }
        };
        debug!("Fetched subscribe checkout");
        let user_profile = self.client.get_user_profile(user_email).await?;
        debug!("Fetched user profile");
        // cancel subscription is the only endpoint that isn't loaded up front

        debug!("Loaded state for {}", email);
        Ok(LoadState {
            content_type_images,
            category_images,
            articles,
            calibrations,
            testimonials,
            testimonial_images,
            subscribe_checkout,
            user_profile,
        })
    }
}
