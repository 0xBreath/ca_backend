use actix_cors::Cors;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder, Result};
use actix_web::http::header;
use dotenv::dotenv;
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use ca_database::client::PostgresClient;
use ca_database::handler::PostgresHandler;

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
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    debug!("Get articles endpoint: {}", db_url);

    // init Postgres client
    let client = PostgresClient::new_from_url(db_url)
      .await
      .expect("Failed to init PostgresHandler");
    let wrapper = PostgresHandler::new(client);

    let articles = wrapper.get_articles().await.expect("Failed to get articles from database");

    Ok(HttpResponse::Ok().json(articles))
}