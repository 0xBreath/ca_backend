use anyhow::{anyhow, Error};
use clap::{ArgEnum, Parser};
use database::{Article, Calibration, MessageHasher, MessageHasherTrait, Testimonial};
use dotenv::dotenv;
use log::*;
use serde::Deserialize;
use simplelog::{ColorChoice, Config as SimpleLogConfig, TermLogger, TerminalMode};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

pub const GCLOUD_STORAGE_PREFIX: &str = "https://storage.googleapis.com/consciousness-archive/";

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
    TestimonialImages,
    CategoryImages,
    ContentTypeImages,
}

impl FromStr for FileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "articles" => Ok(FileType::Articles),
            "calibrations" => Ok(FileType::Calibrations),
            "testimonials" => Ok(FileType::Testimonials),
            "testimonial_images" => Ok(FileType::TestimonialImages),
            "content_type_images" => Ok(FileType::ContentTypeImages),
            "category_images" => Ok(FileType::CategoryImages),
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
    index: u32,
    premium: bool,
}

#[derive(Parser, Debug)]
struct Args {
    /// File type (articles, calibrations, testimonials, etc)
    #[clap(short)]
    t: FileType,

    /// Path to file/folder
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
            let articles_cache = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/cache/articles.bin";
            let mut file = File::open(&articles_cache).expect("Failed to open articles cache");
            let mut articles_buf = Vec::new();
            file.read_to_end(&mut articles_buf)
                .expect("Failed to read articles cache");

            let mut new_file = File::open(path).expect("Failed to open new articles file");
            let mut new_buf = String::new();
            new_file
                .read_to_string(&mut new_buf)
                .expect("Failed to read new articles file");
            let new_articles_raw = serde_json::from_str::<Vec<ArticleRaw>>(&new_buf)
                .expect("Failed to deserialize new articles");

            let mut new_articles = Vec::new();
            for article in new_articles_raw.into_iter() {
                let file_path = std::env::current_dir()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    + "/data/articles/"
                    + &article.file_name;
                info!("Article file path: {}", &file_path);

                let markdown = std::fs::read_to_string(file_path)?
                    .trim_start_matches('\n')
                    .to_string();
                new_articles.push(Article {
                    title: article.title,
                    tags: article.tags,
                    data: markdown,
                    image_url: article.image_url,
                    index: article.index,
                    premium: article.premium,
                });
            }

            let mut db_articles = match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&articles_buf)
            {
                Ok(db_articles) => db_articles,
                Err(e) => {
                    // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                    if e.to_string().contains("unexpected end of file") {
                        HashMap::new()
                    } else {
                        return Err(anyhow!("Failed to deserialize articles cache: {}", e));
                    }
                }
            };

            for new_article in new_articles.into_iter() {
                let bytes = new_article.ser()?;
                db_articles.insert(bytes.key, bytes.value);
            }
            let ser_articles = bincode::serialize(&db_articles)?;
            std::fs::write(articles_cache, ser_articles)?;
            info!("Wrote articles to existing cache");
        }
        FileType::Calibrations => {
            // Read the contents of the calibrations cache into a Vec<u8>
            let cache_path = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/cache/calibrations.bin";
            let mut cache_file =
                File::open(&cache_path).expect("Failed to open calibrations cache");
            let mut cache_buf = Vec::new();
            cache_file
                .read_to_end(&mut cache_buf)
                .expect("Failed to read calibrations cache");

            // Read the contents of the new calibrations file into a Vec<u8>
            let mut new_file = File::open(path).expect("Failed to open new calibrations file");
            let mut new_buf = String::new();
            new_file
                .read_to_string(&mut new_buf)
                .expect("Failed to read new calibrations file");
            let new_calibrations = serde_json::from_str::<Vec<Calibration>>(&new_buf)
                .expect("Failed to deserialize new calibrations");

            let mut db_calibrations =
                match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf) {
                    Ok(db_calibrations) => db_calibrations,
                    Err(e) => {
                        // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                        if e.to_string().contains("unexpected end of file") {
                            HashMap::new()
                        } else {
                            return Err(anyhow!("Failed to deserialize calibrations cache: {}", e));
                        }
                    }
                };
            // append new calibration to cache
            for new_calibration in new_calibrations.into_iter() {
                let bytes = new_calibration.ser()?;
                db_calibrations.insert(bytes.key, bytes.value);
            }

            let ser_calibrations = bincode::serialize(&db_calibrations)?;
            std::fs::write(cache_path, ser_calibrations)?;
            info!("Wrote calibrations to existing cache");
        }
        FileType::Testimonials => {
            // Read the contents of the testimonials cache into a Vec<u8>
            let cache_path = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/cache/testimonials.bin";
            let mut cache_file =
                File::open(&cache_path).expect("Failed to open testimonials cache");
            let mut cache_buf = Vec::new();
            cache_file
                .read_to_end(&mut cache_buf)
                .expect("Failed to read testimonials cache");

            // Read the contents of the new testimonials file into a Vec<u8>
            let mut new_file = File::open(path).expect("Failed to open new testimonials file");
            let mut new_buf = String::new();
            new_file
                .read_to_string(&mut new_buf)
                .expect("Failed to read new testimonials file");
            let new_testimonials = serde_json::from_str::<Vec<Testimonial>>(&new_buf)
                .expect("Failed to deserialize new testimonials");

            let mut db_testimonials =
                match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf) {
                    Ok(db_testimonials) => db_testimonials,
                    Err(e) => {
                        // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                        if e.to_string().contains("unexpected end of file") {
                            HashMap::new()
                        } else {
                            return Err(anyhow!("Failed to deserialize testimonials cache: {}", e));
                        }
                    }
                };
            for new_testimonial in new_testimonials.into_iter() {
                let bytes = new_testimonial.ser()?;
                db_testimonials.insert(bytes.key, bytes.value);
            }
            let ser_testimonials = bincode::serialize(&db_testimonials)?;
            std::fs::write(cache_path, ser_testimonials)?;
            info!("Wrote testimonials to new cache");
        }
        FileType::TestimonialImages => {
            // read all files from directory
            let dir = std::fs::read_dir(PathBuf::from(&path))
                .expect("Failed to read testimonial images directory");

            // read directory after renaming and create JSON for
            // data/testimonial_images/testimonial_images.json
            let mut testimonial_images = Vec::<String>::new();

            for file in dir {
                let file = file.expect("Failed to read testimonial image DirEntry");
                let file_name_os = file.file_name();

                let file_name = file_name_os.to_str().unwrap().to_string();

                if file_name == ".DS_Store" {
                    continue;
                };

                // check if name contains a space, if so concat with -
                let file_name = if file_name.contains(' ') {
                    file_name.replace(' ', "-")
                } else {
                    file_name
                };

                debug!("File name: {:?}", file_name);
                let gcloud_url = format!(
                    "{}images/testimonial_images/{}",
                    GCLOUD_STORAGE_PREFIX, file_name
                );
                info!("Testimonial image: {}", gcloud_url);
                testimonial_images.push(gcloud_url)
            }

            // write testimonial_images.json to data/testimonial_images/testimonial_images.json
            let testimonial_images_json = serde_json::to_string(&testimonial_images)?;

            let database = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/data/testimonial_images";

            let testimonial_images_path = format!("{}/testimonial_images.json", database);
            match std::fs::write(testimonial_images_path, testimonial_images_json) {
                Ok(_) => {
                    info!("Successfully wrote testimonial_images.json");
                }
                Err(e) => {
                    error!("Failed to write testimonial_images.json: {}", e);
                }
            }

            // Read the contents of the testimonials cache into a Vec<u8>
            let cache_path = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/cache/testimonial_images.bin";
            let mut cache_file =
                File::open(&cache_path).expect("Failed to open testimonials cache");
            let mut cache_buf = Vec::new();
            cache_file
                .read_to_end(&mut cache_buf)
                .expect("Failed to read testimonials cache");

            let mut db_testimonial_images =
                match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf) {
                    Ok(db_testimonial_images) => db_testimonial_images,
                    Err(e) => {
                        // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                        if e.to_string().contains("unexpected end of file") {
                            HashMap::new()
                        } else {
                            return Err(anyhow!("Failed to deserialize testimonials cache: {}", e));
                        }
                    }
                };
            // append new calibration to cache
            for image in testimonial_images.into_iter() {
                let key = MessageHasher::new().hash_string(&image);
                let value =
                    bincode::serialize(&image).expect("Failed to serialize testimonial image");
                db_testimonial_images.insert(key, value);
            }

            let ser_testimonial_images = bincode::serialize(&db_testimonial_images)?;
            std::fs::write(cache_path, ser_testimonial_images)?;
            info!("Wrote testimonial images to existing cache");
        }
        FileType::ContentTypeImages => {
            // read all files from directory
            let dir = std::fs::read_dir(PathBuf::from(&path))
                .expect("Failed to read content type images directory");

            // read directory after renaming and create JSON for
            // data/testimonial_images/testimonial_images.json
            let mut content_type_images = Vec::<String>::new();

            for file in dir {
                let file = file.expect("Failed to read content type image DirEntry");
                let file_name_os = file.file_name();

                let file_name = file_name_os.to_str().unwrap().to_string();

                if file_name == ".DS_Store" {
                    continue;
                };

                // check if name contains a space, if so concat with -
                let file_name = if file_name.contains(' ') {
                    file_name.replace(' ', "-")
                } else {
                    file_name
                };

                debug!("File name: {:?}", file_name);
                let gcloud_url = format!(
                    "{}images/content_type_images/{}",
                    GCLOUD_STORAGE_PREFIX, file_name
                );
                info!("Content type image: {}", gcloud_url);
                content_type_images.push(gcloud_url)
            }

            let content_type_images_json = serde_json::to_string(&content_type_images)?;

            let database = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/data/content_type_images";

            let content_type_images_path = format!("{}/content_type_images.json", database);
            match std::fs::write(content_type_images_path, content_type_images_json) {
                Ok(_) => {
                    info!("Successfully wrote content_type_images.json");
                }
                Err(e) => {
                    error!("Failed to write content_type_images.json: {}", e);
                }
            }

            // Read the contents of the testimonials cache into a Vec<u8>
            let cache_path = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/cache/content_type_images.bin";
            let mut cache_file =
                File::open(&cache_path).expect("Failed to open content type images cache");
            let mut cache_buf = Vec::new();
            cache_file
                .read_to_end(&mut cache_buf)
                .expect("Failed to read content type images cache");

            let mut db_content_type_images =
                match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf) {
                    Ok(db_content_type_images) => db_content_type_images,
                    Err(e) => {
                        // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                        if e.to_string().contains("unexpected end of file") {
                            HashMap::new()
                        } else {
                            return Err(anyhow!(
                                "Failed to deserialize content type images cache: \
                            {}",
                                e
                            ));
                        }
                    }
                };
            // append new calibration to cache
            for image in content_type_images.into_iter() {
                let key = MessageHasher::new().hash_string(&image);
                let value =
                    bincode::serialize(&image).expect("Failed to serialize testimonial image");
                db_content_type_images.insert(key, value);
            }

            let ser_content_type_images = bincode::serialize(&db_content_type_images)?;
            std::fs::write(cache_path, ser_content_type_images)?;
            info!("Wrote content type images to existing cache");
        }
        FileType::CategoryImages => {
            // read all files from directory
            let dir = std::fs::read_dir(PathBuf::from(&path))
                .expect("Failed to read category images directory");

            // read directory after renaming and create JSON for
            // data/testimonial_images/testimonial_images.json
            let mut category_images = Vec::<String>::new();

            for file in dir {
                let file = file.expect("Failed to read category image DirEntry");
                let file_name_os = file.file_name();

                let file_name = file_name_os.to_str().unwrap().to_string();

                if file_name == ".DS_Store" {
                    continue;
                };

                // check if name contains a space, if so concat with -
                let file_name = if file_name.contains(' ') {
                    file_name.replace(' ', "-")
                } else {
                    file_name
                };

                debug!("File name: {:?}", file_name);
                let gcloud_url = format!(
                    "{}images/category_images/{}",
                    GCLOUD_STORAGE_PREFIX, file_name
                );
                info!("Category image: {}", gcloud_url);
                category_images.push(gcloud_url)
            }

            let category_images_json = serde_json::to_string(&category_images)?;

            let database = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/data/category_images";

            let category_images_path = format!("{}/category_images.json", database);
            match std::fs::write(category_images_path, category_images_json) {
                Ok(_) => {
                    info!("Successfully wrote content_type_images.json");
                }
                Err(e) => {
                    error!("Failed to write content_type_images.json: {}", e);
                }
            }

            // Read the contents of the category images cache into a Vec<u8>
            let cache_path = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + "/cache/category_images.bin";
            let mut cache_file =
                File::open(&cache_path).expect("Failed to open category images cache");
            let mut cache_buf = Vec::new();
            cache_file
                .read_to_end(&mut cache_buf)
                .expect("Failed to read category images cache");

            let mut db_category_images =
                match bincode::deserialize::<HashMap<u64, Vec<u8>>>(&cache_buf) {
                    Ok(db_category_images) => db_category_images,
                    Err(e) => {
                        // if error is Io(Kind(UnexpectedEof)), then write to file as new hashmap
                        if e.to_string().contains("unexpected end of file") {
                            HashMap::new()
                        } else {
                            return Err(anyhow!(
                                "Failed to deserialize category images cache: \
                            {}",
                                e
                            ));
                        }
                    }
                };
            // append new category image to cache
            for image in category_images.into_iter() {
                let key = MessageHasher::new().hash_string(&image);
                let value = bincode::serialize(&image).expect("Failed to serialize category image");
                db_category_images.insert(key, value);
            }

            let ser_category_images = bincode::serialize(&db_category_images)?;
            std::fs::write(cache_path, ser_category_images)?;
            info!("Wrote category images to existing cache");
        }
    }

    Ok(())
}
