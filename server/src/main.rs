mod errors;
mod handler;
mod oauth;
mod square;

use handler::*;
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
                    .service(learn_images)
                    .service(user_profile)
                    .service(subscribe)
                    .service(testimonials)
                    .service(testimonial_images)
                    .service(load_state),
            )
            .service(
                web::scope("/admin")
                    .wrap(admin_auth)
                    .service(catalogs)
                    .service(customers)
                    .service(email_list)
                    .service(invoices)
                    .service(orders)
                    .service(subscriptions)
                    .service(upsert_subscription_catalog),
            )
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

// ================================== API ================================== //

#[post("/load_state")]
async fn load_state(payload: web::Payload) -> Result<HttpResponse, Error> {
    debug!("Loading state...");
    let client = SQUARE_CLIENT.lock().await;
    let handler = ServerHandler::new(client);
    let res = handler.load_state(payload).await?;
    Ok(HttpResponse::Ok().json(res))
}

#[get("/content_type_images")]
async fn learn_images() -> Result<HttpResponse, Error> {
    let images = ServerHandler::handle_content_type_images().await?;
    Ok(HttpResponse::Ok().json(images))
}

#[get("/articles")]
async fn articles() -> Result<HttpResponse, Error> {
    let articles = ServerHandler::handle_articles()?;
    Ok(HttpResponse::Ok().json(articles))
}

#[get("/calibrations")]
async fn calibrations() -> Result<HttpResponse, Error> {
    let calibrations = ServerHandler::handle_calibrations()?;
    Ok(HttpResponse::Ok().json(calibrations))
}

#[get("/testimonials")]
async fn testimonials() -> Result<HttpResponse, Error> {
    let testimonials = ServerHandler::handle_testimonials()?;
    Ok(HttpResponse::Ok().json(testimonials))
}

#[get("/testimonial_images")]
async fn testimonial_images() -> Result<HttpResponse, Error> {
    let images = ServerHandler::handle_testimonial_images().await?;
    Ok(HttpResponse::Ok().json(images))
}

#[post("/subscribe")]
async fn subscribe(payload: web::Payload) -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let handler = ServerHandler::new(client);
    let res = handler.handle_subscribe(payload).await?;
    Ok(HttpResponse::Ok().json(res))
}

#[post("/user_profile")]
async fn user_profile(payload: web::Payload) -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let handler = ServerHandler::new(client);
    let info = handler.handle_user_profile(payload).await?;
    Ok(HttpResponse::Ok().json(info))
}

#[post("/cancel_subscription")]
async fn cancel_subscription(payload: web::Payload) -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let handler = ServerHandler::new(client);
    let info = handler.handle_cancel_subscription(payload).await?;
    Ok(HttpResponse::Ok().json(info))
}

// ================================== ADMIN ================================== //

#[get("/upsert_subscription_catalog")]
async fn upsert_subscription_catalog() -> Result<HttpResponse, Error> {
    let client = SQUARE_CLIENT.lock().await;
    let catalog = client.upsert_subscription_catalog().await?;
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
