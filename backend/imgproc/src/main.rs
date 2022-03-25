use std::error::Error;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;

use common::aws::s3::{ByteStream, Media, Thumbnails};
use common::aws::S3Bucket;
use common::aws::SQSQueue;
use common::config::Config;

use serde_json::{self, Value};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

use image::io::Reader as ImageReader;

use reqwest::Client;

mod message;
use message::{to_messages, EventType, Message};

#[derive(Debug)]
enum ProcessingError {
    GenericError,
    DownloadFailed(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::load_dotenv().await?;

    let max_wait_time = config.aws_sqs_max_wait_time;

    let sqs = SQSQueue::new(&config).await?;
    let media_bucket = Arc::new(S3Bucket::<Media>::new(&config).await);
    let thumbs_bucket = Arc::new(S3Bucket::<Thumbnails>::new(&config).await);

    loop {
        let result = sqs.receive(max_wait_time).await?;

        if let Some(messages) = result.messages() {
            log::info!("Received {} messages", messages.len());
            log::debug!("{:?}", messages);
            for message in messages {
                if let Some(body) = message.body() {
                    let value = serde_json::from_str::<Value>(body)?;

                    if let Some(parsed_messages) = to_messages(&value) {
                        log::info!("Found {} events", parsed_messages.len());

                        if parsed_messages.is_empty() {
                            continue;
                        }

                        let res =
                            handle_messages(&media_bucket, &thumbs_bucket, &parsed_messages).await;

                        if res.is_ok() {
                            if let Some(handle) = message.receipt_handle() {
                                sqs.delete_message(handle).await?;
                                log::info!("Completed handling message");
                            } else {
                                log::warn!("Did not find handle to clear message from queue");
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn handle_messages(
    media_bucket: &Arc<S3Bucket<Media>>,
    thumbs_bucket: &Arc<S3Bucket<Thumbnails>>,
    messages: &[Arc<Message>],
) -> Result<(), Box<dyn Error>> {
    let mut results = Vec::new();

    for message in messages {
        let message = message.clone();
        let media_bucket = media_bucket.clone();
        let thumbs_bucket = thumbs_bucket.clone();
        let res = tokio::spawn(async move {
            handle_message(media_bucket, thumbs_bucket, message)
                .await
                .map_err(|err| {
                    log::error!("{}", err);
                    ProcessingError::GenericError
                })
        })
        .await?;
        results.push(res);
    }

    for res in results {
        res?;
    }

    Ok(())
}

async fn handle_message(
    media_bucket: Arc<S3Bucket<Media>>,
    thumbs_bucket: Arc<S3Bucket<Thumbnails>>,
    message: Arc<Message>,
) -> Result<(), Box<dyn Error>> {
    match message.event_type {
        EventType::ObjectCreated => {
            handle_object_created(&message.key, media_bucket, thumbs_bucket).await?;
        }
        EventType::ObjectRemoved => {
            handle_object_deleted(&message.key, thumbs_bucket).await?;
        }
    }

    Ok(())
}

async fn handle_object_created(
    key: &String,
    media_bucket: Arc<S3Bucket<Media>>,
    thumbs_bucket: Arc<S3Bucket<Thumbnails>>,
) -> Result<(), Box<dyn Error>> {
    log::info!("Creating thumbnail {}", key);
    let url = media_bucket.get_object_presigned_url(key).await?;

    // file path
    let temp_path = std::env::temp_dir().join(key);
    log::debug!("temp file at {}", temp_path.to_str().unwrap());

    // Download data to file
    download_image(&url, &temp_path).await?;

    // Process image
    process_image(&temp_path).await?;

    // Upload to thumbnails bucket
    log::debug!("uploading image to thumbnails");
    thumbs_bucket
        .put_object(key, "image/jpeg", ByteStream::from_path(&temp_path).await?)
        .await?;

    // Clean up
    log::debug!("cleaning up");
    fs::remove_file(temp_path).await?;

    Ok(())
}

async fn handle_object_deleted(
    key: &String,
    thumbs_bucket: Arc<S3Bucket<Thumbnails>>,
) -> Result<(), Box<dyn Error>> {
    log::info!("Deleting thumbnail {}", key);
    thumbs_bucket.delete_object(key).await?;
    Ok(())
}

async fn download_image(url: &String, temp_path: &Path) -> Result<(), Box<dyn Error>> {
    log::debug!("saving media to file");
    let client = Client::new();

    let res = client.get(url).send().await?;

    if res.status().is_success() {
        let mut stream = res.bytes_stream();

        let mut file = File::create(&temp_path).await?;

        while let Some(buf) = stream.try_next().await? {
            file.write(&buf[..]).await?;
        }

        Ok(())
    } else {
        Err(ProcessingError::DownloadFailed(format!(
            "Failed to download to {} from {}",
            temp_path.to_string_lossy(),
            url
        )))
    }?;

    Ok(())
}

async fn process_image(temp_path: &Path) -> Result<(), Box<dyn Error>> {
    let img = ImageReader::open(&temp_path)?
        .with_guessed_format()?
        .decode()?;

    let resize_img = img.thumbnail(300, 240);

    let key = temp_path.file_name().ok_or(ProcessingError::GenericError)?;

    let mut new_img_path = temp_path.to_path_buf();
    new_img_path.set_file_name(format!("{}_new", key.to_string_lossy()));

    resize_img.save_with_format(&new_img_path, image::ImageFormat::Jpeg)?;

    fs::rename(&new_img_path, &temp_path).await?;

    Ok(())
}

impl Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::GenericError => write!(f, "Failed to process messages"),
            ProcessingError::DownloadFailed(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ProcessingError {}
