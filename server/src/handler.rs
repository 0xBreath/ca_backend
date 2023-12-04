use crate::square::{
    CanceledSubscriptionInfo, CheckoutInfo, SquareClient, SquareResponse, UserEmailRequest,
    UserProfile,
};
use actix_web::{web, Result};
use database::{Article, Calibration, Testimonial};
use futures::StreamExt;
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::list::ListObjectsRequest;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use tokio::sync::MutexGuard;

const MAX_SIZE: usize = 262_144; // max payload size is 256k
pub const GCLOUD_BUCKET: &str = "consciousness-archive";
pub const GCLOUD_STORAGE_PREFIX: &str = "https://storage.googleapis.com/consciousness-archive/";

#[derive(Serialize, Deserialize, Debug)]
pub struct LoadState {
    pub learn_images: Vec<String>,
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

    pub async fn handle_learn_images() -> Result<Vec<String>> {
        let config = ClientConfig::default()
            .with_auth()
            .await
            .expect("Failed to get Google cloud storage client");
        let client = Client::new(config);

        let objects = client
            .list_objects(&ListObjectsRequest {
                bucket: GCLOUD_BUCKET.to_string(),
                prefix: Some("images/learn".to_string()),
                ..Default::default()
            })
            .await
            .expect("Failed to list Google cloud bucket with learn section images");

        let mut images = Vec::<String>::new();
        if let Some(objects) = objects.items {
            images = objects
                .into_iter()
                .map(|object| format!("{}{}", GCLOUD_STORAGE_PREFIX, object.name))
                .collect::<Vec<String>>();
        }
        Ok(images)
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
            let mut article = bincode::deserialize::<Article>(&db_article)
                .expect("Failed to deserialize article");

            let full_path = format!("{}{}", GCLOUD_STORAGE_PREFIX, article.image_url);
            debug!("article path: {}", full_path);

            article.image_url = full_path;
            articles.push(article);
        }
        Ok(articles)
    }

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
            let mut calibration = bincode::deserialize::<Calibration>(&db_calibration)
                .expect("Failed to deserialize calibration");

            let full_path = format!("{}{}", GCLOUD_STORAGE_PREFIX, calibration.image_url);
            debug!("calibration path: {}", full_path);

            calibration.image_url = full_path;
            calibrations.push(calibration);
        }
        Ok(calibrations)
    }

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
            let mut testimonial = bincode::deserialize::<Testimonial>(&db_testimonial)
                .expect("Failed to deserialize testimonial");

            let full_path = format!("{}{}", GCLOUD_STORAGE_PREFIX, testimonial.image_url);
            testimonial.image_url = full_path;

            testimonials.push(testimonial);
        }
        Ok(testimonials)
    }

    pub async fn handle_testimonial_images() -> Result<Vec<String>> {
        let config = ClientConfig::default()
            .with_auth()
            .await
            .expect("Failed to get cloud storage client");

        let client = Client::new(config);

        let objects = client
            .list_objects(&ListObjectsRequest {
                bucket: GCLOUD_BUCKET.to_string(),
                prefix: Some("images/testimonials".to_string()),
                ..Default::default()
            })
            .await
            .expect("Failed to list Google bucket objects for Testimonials");

        let mut images = Vec::<String>::new();
        if let Some(objects) = objects.items {
            images = objects
                .into_iter()
                .map(|object| format!("{}{}", GCLOUD_STORAGE_PREFIX, object.name))
                .collect::<Vec<String>>();
        }
        Ok(images)
    }

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
        let res = self.client.subscribe_checkout(buyer_email).await?;

        if let SquareResponse::Success(subscribe) = res {
            debug!("Subscription checkout: {:?}", &subscribe);
            Ok(subscribe)
        } else {
            error!("Failed to subscribe: {:?}", &res);
            Err(actix_web::error::ErrorBadRequest("Failed to subscribe"))
        }
    }

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

    pub async fn load_state(&self, mut payload: web::Payload) -> Result<LoadState> {
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

        let learn_images = Self::handle_learn_images().await?;
        let articles = Self::handle_articles()?;
        let calibrations = Self::handle_calibrations()?;
        let testimonials = Self::handle_testimonials()?;
        let testimonial_images = Self::handle_testimonial_images().await?;
        let subscribe_checkout = match self.client.subscribe_checkout(user_email.clone()).await? {
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
        let user_profile = self.client.get_user_profile(user_email).await?;
        // cancel subscription is the only endpoint that isn't loaded up front

        Ok(LoadState {
            learn_images,
            articles,
            calibrations,
            testimonials,
            testimonial_images,
            subscribe_checkout,
            user_profile,
        })
    }
}
