use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use database::{Article, Calibration};
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use std::str::FromStr;
use clap::{Parser, ArgEnum};
use dotenv::dotenv;
use anyhow::{anyhow, Error};


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
    Calibrations,
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
            "calibrations" => Ok(FileType::Calibrations),
            _ => Err(format!("{} is not a valid file type", s)),
        }
    }
}


#[derive(Parser, Debug)]
struct Args {
    /// File type (article, course, image, video, audio, calibrations)
    #[clap(short)]
    t: FileType,

    /// Path to file
    #[clap(short)]
    f: String,

    /// Name of file to display on client
    #[clap(short)]
    n: Option<String>,

    /// Image associated with file
    #[clap(short)]
    i: Option<String>,
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

    match file_type {
        FileType::Article => {
            let articles_cache = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/articles.bin";
            info!("articles_cache: {}", &articles_cache);

            let markdown = std::fs::read_to_string(path)?;
            // remove any empty lines before H1 (first line)
            let markdown = markdown.trim_start_matches("\n");

            let article = Article {
                title: name.expect("Article name not provided"),
                data: markdown.to_string(),
                image_url: image.expect("Article image not provided"),
            };

            let mut file = File::open(&articles_cache)
              .expect("Failed to open articles cache");
            // Read the contents into a Vec<u8>
            let mut articles_buf = Vec::new();
            file.read_to_end(&mut articles_buf).
              expect("Failed to read articles cache");

            let new_article = article.ser()?;
            match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&articles_buf) {
                Ok(mut db_articles) => {
                    // append new article to articles cache
                    match db_articles.insert(new_article.key, new_article.value) {
                        Some(_) => info!("Updated article to articles cache"),
                        None => info!("Added new article to articles cache"),
                    };

                    let ser_articles = bincode::serialize(&db_articles)?;
                    match std::fs::write(articles_cache, ser_articles) {
                        Ok(_) => info!("Successfully wrote to articles cache"),
                        Err(e) => {
                            error!("Failed to write to articles cache: {}", e);
                            return Err(anyhow!("Failed to write to articles cache: {}", e))
                        },
                    };
                },
                Err(e) => {
                    // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                    if e.to_string().contains("unexpected end of file") {
                        let mut db_articles = HashMap::new();
                        db_articles.insert(new_article.key, new_article.value);
                        let ser_articles = bincode::serialize(&db_articles)?;
                        std::fs::write(articles_cache, ser_articles)?;
                    } else {
                        return Err(anyhow!("Failed to deserialize articles cache: {}", e));
                    }
                }
            }
        },
        FileType::Calibrations => {
            // Read the contents of the calibrations cache into a Vec<u8>
            let cache_path = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/calibrations.bin";
            let mut cache_file = File::open(&cache_path)
              .expect("Failed to open calibrations cache");
            let mut cache_buf = Vec::new();
            cache_file.read_to_end(&mut cache_buf).
              expect("Failed to read calibrations cache");

            // Read the contents of the new calibrations file into a Vec<u8>
            let mut new_file = File::open(path).expect("Failed to open new calibrations file");
            let mut new_buf = String::new();
            new_file.read_to_string(&mut new_buf).expect("Failed to read new calibrations file");
            let new_calibrations = serde_json::from_str::<Vec<Calibration>>(&new_buf).expect("Failed to deserialize new calibrations");

            match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf) {
                Ok(mut db_calibrations) => {
                    // append new calibration to cache
                    for new_calibration in new_calibrations.into_iter() {
                        let bytes = new_calibration.ser()?;
                        db_calibrations.insert(bytes.key, bytes.value);
                    }

                    let ser_calibrations = bincode::serialize(&db_calibrations)?;
                    std::fs::write(cache_path, ser_calibrations).expect("Failed to write to calibrations cache");
                    info!("Wrote calibrations to existing cache");
                },
                Err(e) => {
                    // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                    if e.to_string().contains("unexpected end of file") {
                        let mut db_calibrations = HashMap::new();
                        for new_calibration in new_calibrations.into_iter() {
                            let bytes = new_calibration.ser()?;
                            db_calibrations.insert(bytes.key, bytes.value);
                        }
                        let ser_calibrations = bincode::serialize(&db_calibrations)?;
                        std::fs::write(cache_path, ser_calibrations)?;
                        info!("Wrote calibrations to new cache");
                    } else {
                        return Err(anyhow!("Failed to deserialize calibrations cache: {}", e));
                    }
                }
            }
        },
        _ => {

        }
    }

    Ok(())
}