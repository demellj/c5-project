use std::error::Error;
use std::marker::PhantomData;
use std::time::Duration;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::output::{GetObjectOutput, PutObjectOutput};
use aws_sdk_s3::presigning::config::PresigningConfig;
use aws_sdk_s3::{Client, Region};

pub use aws_sdk_s3::types::ByteStream;

use crate::config;

pub struct Thumbnails;
pub struct Media;

pub struct S3Bucket<T> {
    bucket: String,
    client: Client,
    data: PhantomData<T>,
}

const EXPIRES_IN: Duration = Duration::from_secs(5 * 60);

impl S3Bucket<Media> {
    pub async fn new(config: &config::Config) -> Self {
        let region_provider =
            RegionProviderChain::first_try(Some(Region::new(config.aws_region.clone())))
                .or_default_provider()
                .or_else("us-east-1");

        let shared_config = aws_config::from_env().region(region_provider).load().await;

        let client = Client::new(&shared_config);

        S3Bucket {
            bucket: config.aws_media_bucket.clone(),
            client,
            data: PhantomData,
        }
    }
}

impl S3Bucket<Thumbnails> {
    pub async fn new(config: &config::Config) -> Self {
        let region_provider =
            RegionProviderChain::first_try(Some(Region::new(config.aws_region.clone())))
                .or_default_provider()
                .or_else("us-east-1");

        let shared_config = aws_config::from_env().region(region_provider).load().await;

        let client = Client::new(&shared_config);

        S3Bucket {
            bucket: config.aws_thumbnails_bucket.clone(),
            client,
            data: PhantomData,
        }
    }
}

impl<T> S3Bucket<T> {
    pub async fn put_object(
        &self,
        object: &String,
        content_type: impl Into<String>,
        data: ByteStream,
    ) -> Result<PutObjectOutput, Box<dyn Error>> {
        let resp = self
            .client
            .put_object()
            .content_type(content_type)
            .bucket(&self.bucket)
            .key(object)
            .body(data)
            .send()
            .await?;

        Ok(resp)
    }

    pub async fn get_object(&self, object: &String) -> Result<GetObjectOutput, Box<dyn Error>> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object)
            .send()
            .await?;

        Ok(resp)
    }

    pub async fn delete_object(&self, object: &String) -> Result<(), Box<dyn Error>> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(object)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_object_presigned_url(
        &self,
        object: &String,
    ) -> Result<String, Box<dyn Error>> {
        let presigned_req = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object)
            .presigned(PresigningConfig::expires_in(EXPIRES_IN)?)
            .await?;

        Ok(presigned_req.uri().to_string())
    }

    pub async fn put_object_presigned_url(
        &self,
        object: &String,
    ) -> Result<String, Box<dyn Error>> {
        let presigned_req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(object)
            .presigned(PresigningConfig::expires_in(EXPIRES_IN)?)
            .await?;

        Ok(presigned_req.uri().to_string())
    }
}
