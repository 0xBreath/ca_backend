mod errors;
mod oauth;
mod square;

use oauth::*;
use square::*;

// #[macro_use]
// extern crate lazy_static;

use actix_cors::Cors;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer, Result};
use actix_web_httpauth::middleware::HttpAuthentication;
use database::{Article, Calibration, Testimonial};
use dotenv::dotenv;
use futures::StreamExt;
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::list::ListObjectsRequest;
use lazy_static::lazy_static;
use log::*;
use simplelog::{
    ColorChoice, CombinedLogger, Config as SimpleLogConfig, ConfigBuilder, TermLogger,
    TerminalMode, WriteLogger,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::sync::Mutex;

const MAX_SIZE: usize = 262_144; // max payload size is 256k
const GCLOUD_BUCKET: &str = "consciousness-archive";
const GCLOUD_STORAGE_PREFIX: &str = "https://storage.googleapis.com/consciousness-archive/";

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

    tokio::spawn(async {
        let client = SQUARE_CLIENT.lock().await;
        let res = client
            .restart_orders_webhook()
            .await
            .expect("Failed to create orders webhook");
        info!("Orders webhook response: {:?}", &res);
    });

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("https://consciousnessarchive.com")
            .allowed_origin("https://drew.ngrok-free.app")
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        let auth = HttpAuthentication::bearer(validator);
        let admin_auth = HttpAuthentication::bearer(admin_validator);

        App::new()
            .wrap(cors)
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .service(articles)
                    .service(calibrations)
                    .service(cancel_subscription)
                    .service(coaching)
                    .service(learn_images)
                    .service(user_profile)
                    .service(user_sessions)
                    .service(subscribe)
                    .service(testimonials)
                    .service(testimonial_images),
            )
            .service(
                web::scope("/admin")
                    .wrap(admin_auth)
                    .service(catalogs)
                    .service(create_attributes)
                    .service(customers)
                    .service(email_list)
                    .service(invoices)
                    .service(list_webhook_events)
                    .service(orders)
                    .service(subscriptions)
                    .service(upsert_coaching_catalog)
                    .service(upsert_subscription_catalog),
            )
            .service(order_webhook_callback)
            .service(test)
    })
    .bind(bind_address)?
    .run()
    .await
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
    ])
    .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize logger"))
}

#[get("/")]
async fn test() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Welcome to Consciousness Archive! We hope you brought cookies."))
}

#[post("/")]
async fn order_webhook_callback(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(actix_web::error::ErrorBadRequest(
                "Orders webhook callback POST request bytes overflow",
            ));
        }
        body.extend_from_slice(&chunk);
    }

    // let data = serde_json::from_slice::<InvoiceWebhookResponse>(&body)?;
    let data = serde_json::from_slice::<serde_json::Value>(&body)?;

    info!("Order webhook callback: {:?}", &data);

    // let client = SQUARE_CLIENT.lock().await;
    // let info: UserSessions = client.get_user_sessions(buyer_email).await?;
    // debug!("Get user sessions info: {:?}", &info);

    Ok(HttpResponse::Ok().json(data))
}

// ================================== API ================================== //

#[get("/learn_images")]
async fn learn_images() -> Result<HttpResponse, Error> {
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
    Ok(HttpResponse::Ok().json(images))
}

#[get("/articles")]
async fn articles() -> Result<HttpResponse, Error> {
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
        let mut article =
            bincode::deserialize::<Article>(&db_article).expect("Failed to deserialize article");

        let full_path = format!("{}{}", GCLOUD_STORAGE_PREFIX, article.image_url);
        debug!("article path: {}", full_path);

        article.image_url = full_path;
        articles.push(article);
    }
    debug!("GET articles: {:?}", &articles.len());

    Ok(HttpResponse::Ok().json(articles))
}

#[get("/calibrations")]
async fn calibrations() -> Result<HttpResponse, Error> {
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
    debug!("GET calibrations: {:?}", &calibrations.len());

    Ok(HttpResponse::Ok().json(calibrations))
}

#[get("/testimonials")]
async fn testimonials() -> Result<HttpResponse, Error> {
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
    debug!("GET testimonials: {:?}", &testimonials.len());

    Ok(HttpResponse::Ok().json(testimonials))
}

#[get("/testimonial_images")]
async fn testimonial_images() -> Result<HttpResponse, Error> {
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

    Ok(HttpResponse::Ok().json(images))
}

#[post("/subscribe")]
async fn subscribe(mut payload: web::Payload) -> Result<HttpResponse, Error> {
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
    let client = SQUARE_CLIENT.lock().await;
    let res = client.subscribe_checkout(buyer_email).await?;

    if let SquareResponse::Success(subscribe) = res {
        debug!("Subscription checkout: {:?}", &subscribe);
        Ok(HttpResponse::Ok().json(subscribe))
    } else {
        error!("Failed to subscribe: {:?}", &res);
        Err(actix_web::error::ErrorBadRequest("Failed to subscribe"))
    }
}

#[post("/coaching")]
async fn coaching(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(actix_web::error::ErrorBadRequest(
                "Coaching POST request bytes overflow",
            ));
        }
        body.extend_from_slice(&chunk);
    }

    let request = serde_json::from_slice::<CoachingRequest>(&body)
        .unwrap_or_else(|e| panic!("Failed to parse coaching checkout request: {}", e));
    debug!("Coaching request: {:?}", &request);
    let client = SQUARE_CLIENT.lock().await;

    let checkout = client.coaching_checkout(request).await?;

    if let SquareResponse::Success(checkout) = checkout {
        debug!("Coaching checkout: {:?}", &checkout);
        Ok(HttpResponse::Ok().json(checkout))
    } else {
        error!(
            "Failed to get coaching package checkout info: {:?}",
            &checkout
        );
        Err(actix_web::error::ErrorBadRequest(
            "Failed to get coaching package checkout info",
        ))
    }
}

#[post("/user_profile")]
async fn user_profile(mut payload: web::Payload) -> Result<HttpResponse, Error> {
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
    let client = SQUARE_CLIENT.lock().await;
    let info = client.get_user_profile(buyer_email).await?;
    debug!("Get user subscription info: {:?}", &info);
    Ok(HttpResponse::Ok().json(info))
}

#[post("/user_sessions")]
async fn user_sessions(mut payload: web::Payload) -> Result<HttpResponse, Error> {
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
    let client = SQUARE_CLIENT.lock().await;

    let info: UserSessions = client.get_user_sessions(buyer_email).await?;
    debug!("Get user sessions info: {:?}", &info);

    Ok(HttpResponse::Ok().json(info))
}

#[post("/cancel_subscription")]
async fn cancel_subscription(mut payload: web::Payload) -> Result<HttpResponse, Error> {
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
    let client = SQUARE_CLIENT.lock().await;

    let info = client.cancel_subscription(buyer_email).await?;
    if let SquareResponse::Success(info) = info {
        Ok(HttpResponse::Ok().json(info))
    } else {
        error!("Failed to cancel subscription: {:?}", &info);
        Err(actix_web::error::ErrorBadRequest(
            "Failed to cancel subscription",
        ))
    }
}

// ================================== ADMIN ================================== //

#[get("/upsert_subscription_catalog")]
async fn upsert_subscription_catalog() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let catalog = client.upsert_subscription_catalog().await?;
    Ok(HttpResponse::Ok().json(catalog))
}

#[get("/upsert_coaching_catalog")]
async fn upsert_coaching_catalog() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let catalog = client.upsert_coaching_catalog().await?;
    Ok(HttpResponse::Ok().json(catalog))
}

#[get("/subscriptions")]
async fn subscriptions() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_subscriptions().await?;
    Ok(HttpResponse::Ok().json(list))
}

#[get("/email_list")]
async fn email_list() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.email_list().await?;
    Ok(HttpResponse::Ok().json(list))
}

#[get("/customers")]
async fn customers() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_customers().await?;
    Ok(HttpResponse::Ok().json(list))
}

#[get("/orders")]
async fn orders() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_orders().await?;
    match list {
        SquareResponse::Success(orders) => {
            info!("Orders: {:?}", &orders.orders.len());
            Ok(HttpResponse::Ok().json(orders))
        }
        SquareResponse::Error(e) => Err(actix_web::error::ErrorBadRequest(format!(
            "Failed to get Square orders: {:?}",
            e
        ))),
    }
}

#[get("/invoices")]
async fn invoices() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_invoices().await?;
    match list {
        SquareResponse::Success(invoices) => {
            info!("Invoices: {:?}", &invoices.invoices.len());
            Ok(HttpResponse::Ok().json(invoices))
        }
        SquareResponse::Error(e) => Err(actix_web::error::ErrorBadRequest(format!(
            "Failed to get Square invoices: {:?}",
            e
        ))),
    }
}

#[get("/catalogs")]
async fn catalogs() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let list = client.list_catalogs().await?;
    Ok(HttpResponse::Ok().json(list))
}

#[get("/create_attributes")]
async fn create_attributes() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let sessions = client
        .create_custom_attribute("sessions".to_string())
        .await?;
    let sessions_credited = client
        .create_custom_attribute("sessions_credited".to_string())
        .await?;
    let sessions_debited = client
        .create_custom_attribute("sessions_debited".to_string())
        .await?;
    let res = CustomAttributeResponses {
        sessions,
        sessions_credited,
        sessions_debited,
    };
    Ok(HttpResponse::Ok().json(res))
}

#[get("/webhook_events")]
async fn list_webhook_events() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let res = client.list_available_webhook_events().await?;
    match res {
        SquareResponse::Success(res) => Ok(HttpResponse::Ok().json(res)),
        SquareResponse::Error(e) => Err(actix_web::error::ErrorBadRequest(format!(
            "Failed to get webhook event list: {:?}",
            e
        ))),
    }
}
