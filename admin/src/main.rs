use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use database::{Article, Calibration, Testimonial};
use log::*;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use std::str::FromStr;
use clap::{Parser, ArgEnum};
use dotenv::dotenv;
use anyhow::{anyhow, Error};
use serde::Deserialize;


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
    Articles,
    Calibrations,
    Testimonials,
}

impl FromStr for FileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "articles" => Ok(FileType::Articles),
            "calibrations" => Ok(FileType::Calibrations),
            "testimonials" => Ok(FileType::Testimonials),
            _ => Err(format!("{} is not a valid file type", s)),
        }
    }
}

#[derive(Deserialize, Debug)]
struct ArticleRaw {
    title: String,
    tags: Vec<String>,
    file_name: String,
    image_url: String,
}


#[derive(Parser, Debug)]
struct Args {
    /// File type (article, course, image, video, audio, calibrations)
    #[clap(short)]
    t: FileType,

    /// Path to file
    #[clap(short)]
    f: String,
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    init_logger();

    let args = Args::parse();
    let file_type = args.t;
    let path = args.f;

    match file_type {
        FileType::Articles => {
            // existing articles cache
            let articles_cache = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/articles.bin";
            let mut file = File::open(&articles_cache)
              .expect("Failed to open articles cache");
            let mut articles_buf = Vec::new();
            file.read_to_end(&mut articles_buf).
              expect("Failed to read articles cache");


            let mut new_file = File::open(path).expect("Failed to open new articles file");
            let mut new_buf = String::new();
            new_file.read_to_string(&mut new_buf).expect("Failed to read new articles file");
            let new_articles_raw = serde_json::from_str::<Vec<ArticleRaw>>(&new_buf).expect("Failed to deserialize new articles");

            let mut new_articles = Vec::new();
            for article in new_articles_raw.into_iter() {
                let file_path = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/data/articles/" + &article.file_name;
                let markdown = std::fs::read_to_string(file_path)?.trim_start_matches('\n').to_string();
                new_articles.push(Article {
                    title: article.title,
                    tags: article.tags,
                    data: markdown,
                    image_url: article.image_url,
                });
            }

            match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&articles_buf) {
                Ok(mut db_articles) => {
                    for new_article in new_articles.into_iter() {
                        let bytes = new_article.ser()?;
                        db_articles.insert(bytes.key, bytes.value);
                    }

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
                        for new_article in new_articles.into_iter() {
                            let bytes = new_article.ser()?;
                            db_articles.insert(bytes.key, bytes.value);
                        }
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
        FileType::Testimonials => {
                // Read the contents of the calibrations cache into a Vec<u8>
                let cache_path = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/testimonials.bin";
                let mut cache_file = File::open(&cache_path)
                  .expect("Failed to open testimonials cache");
                let mut cache_buf = Vec::new();
                cache_file.read_to_end(&mut cache_buf).
                  expect("Failed to read testimonials cache");

                // Read the contents of the new testimonials file into a Vec<u8>
                let mut new_file = File::open(path).expect("Failed to open new testimonials file");
                let mut new_buf = String::new();
                new_file.read_to_string(&mut new_buf).expect("Failed to read new testimonials file");
                let new_testimonials = serde_json::from_str::<Vec<Testimonial>>(&new_buf).expect("Failed to deserialize new testimonials");

                match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf) {
                    Ok(mut db_testimonials) => {
                        // append new calibration to cache
                        for new_testimonial in new_testimonials.into_iter() {
                            let bytes = new_testimonial.ser()?;
                            db_testimonials.insert(bytes.key, bytes.value);
                        }

                        let ser_testimonials = bincode::serialize(&db_testimonials)?;
                        std::fs::write(cache_path, ser_testimonials).expect("Failed to write to testimonials cache");
                        info!("Wrote testimonials to existing cache");
                    },
                    Err(e) => {
                        // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                        if e.to_string().contains("unexpected end of file") {
                            let mut db_testimonials = HashMap::new();
                            for new_testimonial in new_testimonials.into_iter() {
                                let bytes = new_testimonial.ser()?;
                                db_testimonials.insert(bytes.key, bytes.value);
                            }
                            let ser_testimonials = bincode::serialize(&db_testimonials)?;
                            std::fs::write(cache_path, ser_testimonials)?;
                            info!("Wrote testimonials to new cache");
                        } else {
                            return Err(anyhow!("Failed to deserialize testimonials cache: {}", e));
                        }
                    }
            }
        }
    }

    Ok(())
}