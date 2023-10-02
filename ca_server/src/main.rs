// #[macro_use]
// extern crate lazy_static;

use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder, Result};
use dotenv::dotenv;
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use ca_database::handler::PostgresHandler;

// TODO: init client with tokio block on because lazy static can't do async
// lazy_static! {
//     static ref CLIENT: String = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    init_logger();

    let port = std::env::var("PORT").unwrap_or_else(|_| "3333".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    info!("Starting Server...");
    HttpServer::new(|| {
        App::new()
          .service(get_articles)
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
async fn get_articles() -> Result<HttpResponse, Error> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("Init db_url in get_articles: {}", db_url);

    // init Postgres client
    let client = PostgresHandler::new_from_url(db_url)
      .await
      .expect("Failed to init PostgresHandler");
    info!("Init client in get_articles");

    let articles = client.get_articles().await.expect("Failed to get articles from database");

    Ok(HttpResponse::Ok().json(articles))
}