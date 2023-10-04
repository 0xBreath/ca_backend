use actix_cors::Cors;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder, Result};
use actix_web::http::header;
use dotenv::dotenv;
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use std::collections::HashMap;
use database::{Article};
use std::fs::File;
use std::io::Read;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    init_logger();

    let port = std::env::var("PORT").unwrap_or_else(|_| "3333".to_string());
    let bind_address = format!("0.0.0.0:{}", port);


    info!("Starting Server...");
    HttpServer::new(|| {
        let cors = Cors::default()
          .allowed_origin("http://localhost:3000")
          .allowed_methods(vec!["GET", "POST"])
          .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
          .allowed_header(header::CONTENT_TYPE)
          .max_age(3600);

        App::new()
          .wrap(cors)
          .service(articles)
          .route("/", web::get().to(test))
    })
      .bind(bind_address)?
      .run()
      .await
}

fn init_logger() {
    TermLogger::init(
        LevelFilter::Info,
        SimpleLogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
      .expect("Failed to initialize logger");
}

async fn test() -> impl Responder {
    HttpResponse::Ok().body("Server is running...")
}

#[get("/articles")]
async fn articles() -> Result<HttpResponse, Error> {
    let articles_cache = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/articles.bin";
    debug!("articles_cache: {}", &articles_cache);

    let mut file = File::open(&articles_cache)
      .expect("Failed to open articles cache");
    // Read the contents into a Vec<u8>
    let mut articles_buf = Vec::new();
    file.read_to_end(&mut articles_buf).
      expect("Failed to read articles cache");

    let mut db_articles = bincode::deserialize::<HashMap<u64, Vec<u8>>>(&articles_buf).expect("Failed to read articles cache");
    let mut articles = Vec::new();
    // for each db_article in the hashmap, deserialize into Article and collect to vector
    for (_, db_article) in db_articles.drain() {
        let article = bincode::deserialize::<Article>(&db_article).expect("Failed to deserialize article");
        articles.push(article);
    }
    info!("GET articles: {:?}", &articles.len());

    Ok(HttpResponse::Ok().json(articles))
}