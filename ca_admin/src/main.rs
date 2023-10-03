mod hash;

use hash::MessageHasher;
use ca_database::{PostgresClient, PostgresHandler};
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use std::str::FromStr;
use clap::{Parser, ArgEnum};
use crate::hash::MessageHasherTrait;
use dotenv::dotenv;

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
    dotenv().ok();
    init_logger();

    let args = Args::parse();
    let file_type = args.t;
    let path = args.f;
    info!("File type: {:?}", file_type);
    info!("path: {:?}", path);

    // init Postgres client
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    debug!("Get articles endpoint: {}", db_url);
    let client = PostgresClient::new_from_url(db_url)
      .await
      .expect("Failed to init PostgresClient");
    let wrapper = PostgresHandler::new(client);

    let mut hasher = MessageHasher::new();

    match file_type {
        FileType::Article => {
            // TODO: remove any empty lines before H1 (first line)
            let markdown = std::fs::read_to_string(path).expect("Failed to read article markdown file");
            debug!("file: {}", markdown);
            let hash = hasher.hash_article(&markdown);
            let key = bincode::serialize(&hash).expect("Failed to serialize key");

            let ser_article =
              bincode::serialize(&markdown).expect("Failed to serialize article");

            let result = wrapper.upsert_article(&key, &ser_article).await.expect("Failed to upsert article");
            info!("result: {:?}", result);
        },
        _ => {}
    }

    Ok(())
}












// use std::convert::TryInto;
// use image::{Rgb, RgbImage};
// use image::codecs::png::{PngEncoder};
//
// fn test() {
//     let mut img = RgbImage::new(512, 512);
//     for x in 0u32..512 {
//         for y in 0u32..512 {
//             let r = ((x + y) % 4).try_into().unwrap();
//             let g = 255 - r;
//             let b = 127 - r;
//             img.put_pixel(x, y, Rgb([r, g, b]));
//         }
//     }
//
//     let mut cursor = Vec::new();
//     let encoder = PngEncoder::new(&mut cursor);
//     encoder
//       .encode(&img, 512, 512, image::ColorType::Rgb8)
//       .unwrap();
//
//     let bytes = cursor;
//     println!("{}", bytes.len());
// }
