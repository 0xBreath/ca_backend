mod types;
use types::*;
use actix_cors::Cors;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer, Responder, Result};
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
use futures::StreamExt;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    init_logger(&PathBuf::from("server.log".to_string()))?;

    let port = std::env::var("PORT").unwrap_or_else(|_| "3333".to_string());
    let bind_address = format!("0.0.0.0:{}", port);


    info!("Starting Server...");
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
          .service(articles)
          .service(calibrations)
          .service(testimonials)
          .service(subscribe)
          .route("/", web::get().to(test))
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
    ]).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize logger"))
}


async fn test() -> impl Responder {
    HttpResponse::Ok().body("Server is running...")
}

#[get("/articles")]
async fn articles() -> Result<HttpResponse, Error> {
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
        let article = bincode::deserialize::<Article>(&db_article).expect("Failed to deserialize article");
        articles.push(article);
    }
    info!("GET articles: {:?}", &articles.len());

    Ok(HttpResponse::Ok().json(articles))
}

#[get("/calibrations")]
async fn calibrations() -> Result<HttpResponse, Error> {
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
        let calibration = bincode::deserialize::<Calibration>(&db_calibration).expect("Failed to deserialize calibration");
        calibrations.push(calibration);
    }
    info!("GET calibrations: {:?}", &calibrations.len());

    Ok(HttpResponse::Ok().json(calibrations))
}

#[get("/testimonials")]
async fn testimonials() -> Result<HttpResponse, Error> {
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
        let testimonial = bincode::deserialize::<Testimonial>(&db_testimonial).expect("Failed to deserialize testimonial");
        testimonials.push(testimonial);
    }
    info!("GET testimonials: {:?}", &testimonials.len());

    Ok(HttpResponse::Ok().json(testimonials))
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[post("/subscribe")]
async fn subscribe(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(actix_web::error::ErrorBadRequest("Subscribe POST request bytes overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let request = serde_json::from_slice::<CreateCustomerRequest>(&body)?;
    debug!("Create customer request: {:?}", &request);

    // GET customer from Square, PUT if exists, POST if not
    let base_url = std::env::var("SQUARE_API_URL").unwrap_or_else(|_| "https://connect.squareupsandbox.com/v2/".to_string());
    let token = std::env::var("SQUARE_ACCESS_TOKEN").unwrap_or_else(|_| "".to_string());
    let version = std::env::var("SQUARE_API_VERSION").unwrap_or_else(|_| "2023-09-25".to_string());

    let client = reqwest::Client::new();

    // POST customer search
    let search_customer_endpoint = base_url.clone() + "customers/search";
    let query = SearchCustomerQuery::new(request.email_address.clone()).to_value()?;
    let res = client.post(search_customer_endpoint)
      .header("Square-Version", version.clone())
      .bearer_auth(token.clone())
      .json(&query)
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST customer search to Square"))?;
    let res = res.json::<SearchCustomerResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse SearchCustomerResponse from Square"))?;
    info!("POST Square search customer: {:?}", &res);

    if res.customers.is_empty() {
        // create new customer -> POST
        let create_customer_endpoint = base_url.clone() + "customers";
        let res = client.post(create_customer_endpoint)
          .header("Square-Version", version.clone())
          .bearer_auth(token.clone())
          .header("Content-Type", "application/json")
          .json(&request)
          .send()
          .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST customer create to Square"))?;
        let res = res.json::<serde_json::Value>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST response from Square"))?;
        info!("POST Square create customer: {:?}", &res);
    } else {
        // update existing customer to subscribe -> PUT
        let update_customer_endpoint = base_url.clone() + "customers";
        let res = client.put(update_customer_endpoint)
          .header("Square-Version", version.clone())
          .bearer_auth(token.clone())
          .header("Content-Type", "application/json")
          .json(&request)
          .send()
          .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send PUT customer update to Square"))?;
        let res = res.json::<serde_json::Value>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse PUT response from Square"))?;
        info!("PUT Square update customer: {:?}", &res);
    }


    Ok(HttpResponse::Ok().json(res))
}