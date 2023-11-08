mod square;
mod errors;
mod oauth;

use square::*;
use oauth::*;

#[macro_use]
extern crate lazy_static;

use actix_cors::Cors;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder, Result, post};
use actix_web::http::header;
use dotenv::dotenv;
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode, WriteLogger, ConfigBuilder, CombinedLogger};
use std::collections::HashMap;
use database::{Article, Calibration, Testimonial};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use actix_web_httpauth::middleware::HttpAuthentication;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use futures::StreamExt;
use google_cloud_storage::http::objects::list::ListObjectsRequest;
use google_cloud_storage::client::{ClientConfig, Client};

const MAX_SIZE: usize = 262_144; // max payload size is 256k
const GCLOUD_BUCKET: &str = "consciousness-archive";
const GCLOUD_STORAGE_PREFIX: &str = "https://storage.googleapis.com/consciousness-archive/";
const LOCAL_STORAGE_PREFIX: &str = "/Users/riester/LIFE/C-Archive/";
const LOCAL_IMAGE_PREFIX: &str = "/";

pub enum Deployment {
    Dev,
    Prod
}

impl PartialEq for Deployment {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Deployment::Dev, Deployment::Dev) => true,
            (Deployment::Prod, Deployment::Prod) => true,
            _ => false
        }
    }
}

impl FromStr for Deployment {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        return match s {
            "dev" => Ok(Self::Dev),
            "prod" => Ok(Self::Prod),
            _ => {
                error!("Invalid deployment environment in env");
                Err(())
            }
        }
    }
}

lazy_static! {
    static ref SQUARE_CLIENT: Mutex<SquareClient> = Mutex::new(SquareClient::new());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    init_logger(&PathBuf::from("server.log".to_string()))?;

    info!("Starting Server...");

    let port = std::env::var("PORT").unwrap_or_else(|_| "3333".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    let deployment = Deployment::from_str(
        &std::env::var("DEPLOYMENT").expect("Failed to read deployment mode from env")
    ).expect("Failed to parse Environment from str");

    match deployment {
        Deployment::Prod => {
            HttpServer::new(|| {
                let cors = Cors::default()
                  .send_wildcard()
                  .allowed_origin("http://localhost:3000")
                  .allowed_origin("https://consciousnessarchive.com")
                  .allowed_methods(vec!["GET", "POST"])
                  .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
                  .max_age(3600);

                let auth = HttpAuthentication::bearer(validator);

                App::new()
                  .wrap(auth)
                  .wrap(cors)
                  .service(test)
                  .service(articles)
                  .service(calibrations)
                  .service(testimonials)
                  .service(upsert_catalog)
                  .service(subscribe)
                  .service(subscriptions)
                  .service(customers)
                  .service(invoices)
                  .service(email_list)
                  .service(user_profile)
                  .service(testimonial_images)
                  .service(learn_images)
                  .service(catalogs)
            })
              .bind(bind_address)?
              .run()
              .await
        },
        Deployment::Dev => {
            HttpServer::new(|| {
                let cors = Cors::default()
                  .send_wildcard()
                  .allowed_origin("http://localhost:3000")
                  .allowed_origin("https://consciousnessarchive.com")
                  .allowed_methods(vec!["GET", "POST"])
                  .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
                  .max_age(3600);

                App::new()
                  .wrap(cors)
                  .service(test)
                  .service(articles)
                  .service(calibrations)
                  .service(testimonials)
                  .service(upsert_catalog)
                  .service(subscribe)
                  .service(subscriptions)
                  .service(customers)
                  .service(invoices)
                  .service(email_list)
                  .service(user_profile)
                  .service(testimonial_images)
                  .service(learn_images)
                  .service(catalogs)
            })
              .bind(bind_address)?
              .run()
              .await
        }
    }
}

pub fn init_logger(log_file: &PathBuf) -> std::io::Result<()> {
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "DEBUG".to_string());
    let level_filter = LevelFilter::from_str(log_level.as_str()).unwrap();
    CombinedLogger::init(vec![
        TermLogger::new(
            level_filter,
            SimpleLogConfig::default(),
            TerminalMode::Mixed,
            ColorChoice::Always,
        ),
        WriteLogger::new(
            level_filter,
            ConfigBuilder::new().set_time_format_rfc3339().build(),
            File::create(log_file)?,
        ),
    ]).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize logger"))
}

#[get("/")]
async fn test() -> impl Responder {
    HttpResponse::Ok().body("Consciousness Archive server is running...")
}

#[get("/learn_images")]
async fn learn_images() -> Result<HttpResponse, Error> {
    let deployment = Deployment::from_str(
        &std::env::var("DEPLOYMENT").expect("Failed to read deployment mode from env")
    ).expect("Failed to parse Environment from str");

    match deployment {
        Deployment::Prod => {
            let config = ClientConfig::default()
              .with_auth()
              .await
              .expect("Failed to get cloud storage client");

            let client = Client::new(config);

            let objects = client.list_objects(&ListObjectsRequest {
                bucket: GCLOUD_BUCKET.to_string(),
                prefix: Some("images/learn".to_string()),
                ..Default::default()
            }).await.expect("Failed to list Google bucket objects for Learn");

            let mut images = Vec::<String>::new();
            if let Some(objects) = objects.items {
                images = objects.into_iter().map(|object| {
                    format!("{}{}", GCLOUD_STORAGE_PREFIX, object.name)
                }).collect::<Vec<String>>();
            }

            Ok(HttpResponse::Ok().json(images))

        },
        Deployment::Dev => {
            let dir_path = LOCAL_STORAGE_PREFIX.to_string() + "images/learn/";
            let images_dir = std::fs::read_dir(&PathBuf::from(dir_path.clone()))
              .expect("Failed to read from local Learn images directory");

            let mut images = Vec::<String>::new();
            for file_result in images_dir.into_iter() {
                let file = file_result.unwrap();
                let os_file_name = file.file_name();
                let file_name = os_file_name.to_str().expect("Failed to read file OsString name to string");
                let full_path = format!("{}images/learn/{}", LOCAL_IMAGE_PREFIX, file_name);
                debug!("Full Learn image path: {}", &full_path);

                let suffix = file_name.split('.').last().unwrap();
                if suffix == "DS_Store" {
                    continue;
                }

                images.push(full_path);
            }

            Ok(HttpResponse::Ok().json(images))
        }
    }
}

#[get("/articles")]
async fn articles() -> Result<HttpResponse, Error> {
    let deployment = Deployment::from_str(
        &std::env::var("DEPLOYMENT").expect("Failed to read deployment mode from env")
    ).expect("Failed to parse Environment from str");

    let cache_path = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/articles.bin";

    let mut cache_file = File::open(&cache_path)
      .expect("Failed to open articles cache");
    // Read the contents into a Vec<u8>
    let mut cache_buf = Vec::new();
    cache_file.read_to_end(&mut cache_buf).
      expect("Failed to read articles cache");

    let mut db_articles = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf).expect("Failed to read articles cache");
    let mut articles = Vec::new();
    // for each DbArticle in the hashmap, deserialize into Article and collect to vector
    for (_, db_article) in db_articles.drain() {
        let mut article = bincode::deserialize::<Article>(&db_article).expect("Failed to deserialize article");

        let prefix = match deployment {
            Deployment::Prod => GCLOUD_STORAGE_PREFIX,
            Deployment::Dev => LOCAL_IMAGE_PREFIX
        };
        let full_path = format!("{}{}", prefix, article.image_url);
        debug!("article path: {}", full_path);

        article.image_url = full_path;
        articles.push(article);
    }
    debug!("GET articles: {:?}", &articles.len());

    Ok(HttpResponse::Ok().json(articles))
}

#[get("/calibrations")]
async fn calibrations() -> Result<HttpResponse, Error> {
    let deployment = Deployment::from_str(
        &std::env::var("DEPLOYMENT").expect("Failed to read deployment mode from env")
    ).expect("Failed to parse Environment from str");

    let cache_path = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/calibrations.bin";

    let mut cache_file = File::open(&cache_path)
      .expect("Failed to open calibrations cache");
    // Read the contents into a Vec<u8>
    let mut cache_buf = Vec::new();
    cache_file.read_to_end(&mut cache_buf).
      expect("Failed to read calibrations cache");

    let mut db_calibrations = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf).expect("Failed to read calibrations cache");
    let mut calibrations = Vec::new();
    // for each DbCalibration in the hashmap, deserialize into Calibration and collect to vector
    for (_, db_calibration) in db_calibrations.drain() {
        let mut calibration = bincode::deserialize::<Calibration>(&db_calibration).expect("Failed to deserialize calibration");

        let prefix = match deployment {
            Deployment::Prod => GCLOUD_STORAGE_PREFIX,
            Deployment::Dev => LOCAL_IMAGE_PREFIX
        };
        let full_path = format!("{}{}", prefix, calibration.image_url);
        debug!("calibration path: {}", full_path);

        calibration.image_url = full_path;
        calibrations.push(calibration);
    }
    debug!("GET calibrations: {:?}", &calibrations.len());

    Ok(HttpResponse::Ok().json(calibrations))
}

#[get("/testimonials")]
async fn testimonials() -> Result<HttpResponse, Error> {
    let deployment = Deployment::from_str(
        &std::env::var("DEPLOYMENT").expect("Failed to read deployment mode from env")
    ).expect("Failed to parse Environment from str");

    let cache_path = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/testimonials.bin";

    let mut cache_file = File::open(&cache_path)
      .expect("Failed to open testimonials cache");
    // Read the contents into a Vec<u8>
    let mut cache_buf = Vec::new();
    cache_file.read_to_end(&mut cache_buf).
      expect("Failed to read testimonials cache");

    let mut db_testimonials = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf).expect("Failed to read testimonials cache");
    let mut testimonials = Vec::new();
    // for each DbCalibration in the hashmap, deserialize into Calibration and collect to vector
    for (_, db_testimonial) in db_testimonials.drain() {
        let mut testimonial = bincode::deserialize::<Testimonial>(&db_testimonial).expect("Failed to deserialize testimonial");

        let prefix = match deployment {
            Deployment::Prod => GCLOUD_STORAGE_PREFIX,
            Deployment::Dev => LOCAL_IMAGE_PREFIX
        };
        let full_path = format!("{}{}", prefix, testimonial.image_url);
        testimonial.image_url = full_path;

        testimonials.push(testimonial);
    }
    debug!("GET testimonials: {:?}", &testimonials.len());

    Ok(HttpResponse::Ok().json(testimonials))
}

#[get("/testimonial_images")]
async fn testimonial_images() -> Result<HttpResponse, Error> {
    let deployment = Deployment::from_str(
        &std::env::var("DEPLOYMENT").expect("Failed to read deployment mode from env")
    ).expect("Failed to parse Environment from str");

    match deployment {
        Deployment::Prod => {
            let config = ClientConfig::default()
              .with_auth()
              .await
              .expect("Failed to get cloud storage client");

            let client = Client::new(config);

            let objects = client.list_objects(&ListObjectsRequest {
                bucket: GCLOUD_BUCKET.to_string(),
                prefix: Some("images/testimonials".to_string()),
                ..Default::default()
            }).await.expect("Failed to list Google bucket objects for Testimonials");

            let mut images = Vec::<String>::new();
            if let Some(objects) = objects.items {
                images = objects.into_iter().map(|object| {
                    format!("{}{}", GCLOUD_STORAGE_PREFIX, object.name)
                }).collect::<Vec<String>>();
            }

            Ok(HttpResponse::Ok().json(images))

        },
        Deployment::Dev => {
            let dir_path = LOCAL_STORAGE_PREFIX.to_string() + "images/testimonials/";
            let images_dir = std::fs::read_dir(&PathBuf::from(dir_path.clone()))
              .expect("Failed to read from local testimonial images directory");

            let mut images = Vec::<String>::new();
            for file_result in images_dir.into_iter() {
                let file = file_result.unwrap();
                let os_file_name = file.file_name();
                let file_name = os_file_name.to_str().expect("Failed to read file OsString name to string");
                let full_path = format!("{}images/testimonials/{}", LOCAL_IMAGE_PREFIX, file_name);
                debug!("Full testimonial image path: {}", &full_path);

                let suffix = file_name.split('.').last().unwrap();
                if suffix == "DS_Store" {
                    continue;
                }

                images.push(full_path);
            }

            Ok(HttpResponse::Ok().json(images))
        }
    }
}

// ================================== SQUARE API ================================== //

#[post("/subscribe")]
async fn subscribe(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    let deployment = Deployment::from_str(
        &std::env::var("DEPLOYMENT").expect("Failed to read deployment mode from env")
    ).expect("Failed to parse Environment from str");

    match deployment {
        Deployment::Prod => {
            let mut body = web::BytesMut::new();
            while let Some(chunk) = payload.next().await {
                let chunk = chunk?;
                if (body.len() + chunk.len()) > MAX_SIZE {
                    return Err(actix_web::error::ErrorBadRequest("Subscribe POST request bytes overflow"));
                }
                body.extend_from_slice(&chunk);
            }

            let buyer_email = serde_json::from_slice::<UserEmailRequest>(&body)?;
            debug!("Checkout user email: {:?}", &buyer_email);
            let client = SQUARE_CLIENT.lock().await;
            let subscribe = client.subscribe(buyer_email).await?;
            info!("Checkout: {:?}", &subscribe);
            Ok(HttpResponse::Ok().json(subscribe))
        },
        Deployment::Dev => {
            let url = "http://localhost:3000".to_string();
            let checkout_info = CheckoutInfo {
                url,
                amount: 1.0
            };

            Ok(HttpResponse::Ok().json(checkout_info))
        }
    }

}

#[post("/user")]
async fn user_profile(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    let deployment = Deployment::from_str(
        &std::env::var("DEPLOYMENT").expect("Failed to read deployment mode from env")
    ).expect("Failed to parse Environment from str");

    match deployment {
        Deployment::Prod => {
            let mut body = web::BytesMut::new();
            while let Some(chunk) = payload.next().await {
                let chunk = chunk?;
                if (body.len() + chunk.len()) > MAX_SIZE {
                    return Err(actix_web::error::ErrorBadRequest("Subscribe POST request bytes overflow"));
                }
                body.extend_from_slice(&chunk);
            }

            let buyer_email = serde_json::from_slice::<UserEmailRequest>(&body)?;
            debug!("User subscription request email: {:?}", &buyer_email);
            let client = SQUARE_CLIENT.lock().await;
            let info: Option<UserProfile> = client.get_user_profile(buyer_email).await?;
            debug!("Get user subscription info: {:?}", &info);
            Ok(HttpResponse::Ok().json(info))
        },
        Deployment::Dev => {
            let customer = CustomerInfo {
                email_address: "email@example.com".to_string(),
                family_name: "Doe".to_string(),
                given_name: "John".to_string(),
                cards: None
            };
            let subscription_info = SubscriptionInfo {
                title: "Premium Local Test".to_string(),
                cost: 1.0,
            };
            let user_subscription = None;
            let user_profile = UserProfile {
                customer,
                subscription_info,
                user_subscription
            };

            Ok(HttpResponse::Ok().json(user_profile))
        }
    }
}

// ================================== ADMIN API ================================== //

// todo: scope = admin
#[get("/upsert_catalog")]
async fn upsert_catalog() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let catalog = client.upsert_catalog().await?;
    Ok(HttpResponse::Ok().json(catalog))
}

// todo: scope = admin
#[get("/subscriptions")]
async fn subscriptions() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_subscriptions().await?;
    Ok(HttpResponse::Ok().json(list))
}

// todo: scope = admin
#[get("/email_list")]
async fn email_list() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.email_list().await?;
    Ok(HttpResponse::Ok().json(list))
}

// todo: scope = admin
#[get("/customers")]
async fn customers() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_customers().await?;
    Ok(HttpResponse::Ok().json(list))
}

// todo: scope = admin
#[get("/invoices")]
async fn invoices() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_invoices().await?;
    Ok(HttpResponse::Ok().json(list))
}

// todo: scope = admin
#[get("/catalogs")]
async fn catalogs() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_catalogs().await?;
    Ok(HttpResponse::Ok().json(list))
}