use ca_database::handler::PostgresHandler;
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use std::str::FromStr;
use clap::{Parser, ArgEnum};

fn init_logger() {
    TermLogger::init(
        LevelFilter::Info,
        SimpleLogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
      .expect("Failed to initialize logger");
}

#[derive(ArgEnum, Debug, Clone)]
enum FileType {
    Article,
    Course,
    Image,
    Video,
    Audio,
}

impl FromStr for FileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "article" => Ok(FileType::Article),
            "course" => Ok(FileType::Course),
            "image" => Ok(FileType::Image),
            "video" => Ok(FileType::Video),
            "audio" => Ok(FileType::Audio),
            _ => Err(format!("{} is not a valid file type", s)),
        }
    }
}


#[derive(Parser, Debug)]
struct Args {
    /// File type (article, course, image, video, audio)
    #[clap(short)]
    t: FileType,

    /// Path to file
    #[clap(short)]
    f: String,
}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_logger();

    let args = Args::parse();
    let file_type = args.t;
    let path = args.f;
    info!("File type: {:?}", file_type);
    info!("path: {:?}", path);

    // let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // debug!("Get articles endpoint: {}", db_url);
    //
    // // init Postgres client
    // let client = PostgresHandler::new_from_url(db_url)
    //   .await
    //   .expect("Failed to init PostgresHandler");
    //
    // // read cli arg for file path to article
    // let args: Vec<String> = std::env::args().collect();


    Ok(())
}
