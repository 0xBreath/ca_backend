use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use database::{Article};
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

    let articles_cache = std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/cache/articles.bin";
    info!("articles_cache: {}", &articles_cache);

    let args = Args::parse();
    let file_type = args.t;
    let path = args.f;
    let name = args.n;
    let image = args.i;

    if let FileType::Article = file_type {
        let markdown = std::fs::read_to_string(path)?;
        // remove any empty lines before H1 (first line)
        let markdown = markdown.trim_start_matches("\n");

        let article = Article {
            title: name,
            data: markdown.to_string(),
            image_url: image,
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


    }

    Ok(())
}