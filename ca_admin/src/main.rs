use ca_database::{Article, PostgresClient, PostgresHandler};
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use std::str::FromStr;
use clap::{Parser, ArgEnum};
use dotenv::dotenv;
use anyhow::Error;

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

    /// Name of file to display on client
    #[clap(short)]
    n: String,

    /// Image associated with file
    #[clap(short)]
    i: String,
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    init_logger();

    let args = Args::parse();
    let file_type = args.t;
    let path = args.f;
    let name = args.n;
    let image = args.i;

    // init Postgres client
    let db_url = std::env::var("DATABASE_URL")?;
    let client = PostgresClient::new_from_url(db_url)
      .await
      .expect("Failed to init PostgresClient");
    let wrapper = PostgresHandler::new(client);

    if let FileType::Article = file_type {
        let markdown = std::fs::read_to_string(path)?;
        // remove any empty lines before H1 (first line)
        let markdown = markdown.trim_start_matches("\n");

        let article = Article {
            title: name,
            data: markdown.to_string(),
            image_url: image,
        };

        let result = wrapper.upsert_article(article).await?;
        info!("result: {:?}", result);
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
