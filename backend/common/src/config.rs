use std::{collections::HashMap, fmt::Display, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

use dotenv;

use crate::jwt;
use crate::aws;

pub const AWS_PROFILE: &'static str = "AWS_PROFILE";
pub const AWS_REGION: &'static str = "AWS_REGION";
pub const AWS_MEDIA_BUCKET: &'static str = "AWS_MEDIA_BUCKET";
pub const AWS_THUMBNAILS_BUCKET: &'static str = "AWS_THUMBNAILS_BUCKET";
pub const AWS_THUMBNAILS_BASE_URL: &'static str = "AWS_THUMBNAILS_BASE_URL";
pub const AWS_SQS_QUEUE: &'static str = "AWS_SQS_QUEUE";
pub const AWS_SQS_MAX_WAIT_TIME_IN_SEC: &'static str = "AWS_SQS_MAX_WAIT_TIME_IN_SEC";
pub const POSTGRESS_USERNAME: &'static str = "POSTGRESS_USERNAME";
pub const POSTGRESS_PASSWORD: &'static str = "POSTGRESS_PASSWORD";
pub const POSTGRESS_DATABASE: &'static str = "POSTGRESS_DATABASE";
pub const POSTGRESS_HOST: &'static str = "POSTGRESS_HOST";
pub const JWT_SECRET: &'static str = "JWT_SECRET";
pub const JWT_TOKEN_TIMEOUT: &'static str = "JWT_TOKEN_TIMEOUT";

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub aws_sqs_queue: String,
    pub aws_sqs_max_wait_time: Duration,
    pub aws_media_bucket: String,
    pub aws_thumbnails_bucket: String,
    pub aws_thumbnails_base_url: String,
    #[serde(default = "gen_aws_default_profile")]
    pub aws_profile: String,
    pub aws_region: String,
    pub database_username: String,
    pub database_password: String,
    pub database_name: String,
    pub database_host: String,
    #[serde(default = "gen_default_database_dialect")]
    pub database_dialect: String,
    pub jwt_secret: String,
    pub jwt_token_timeout: Duration,
}

fn gen_aws_default_profile() -> String {
    "default".into()
}

fn gen_default_database_dialect() -> String {
    "postgres".into()
}

#[derive(Debug)]
pub struct VarNotFound<'a>(&'a str);

impl Display for VarNotFound<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config variable \"{}\" is required, but was not found",
            self.0
        )
    }
}
impl std::error::Error for VarNotFound<'_> {}

impl Config {
    pub async fn load_dotenv() -> Result<Config, Box<dyn std::error::Error>> {
        let vars = spawn_blocking(init_dotenv).await??;

        let default_timeout = format!("{}", jwt::DEFAULT_TIMEOUT_IN_SEC);
        let timeout = vars
            .get(JWT_TOKEN_TIMEOUT)
            .unwrap_or(&default_timeout)
            .parse::<u64>()
            .expect("Failed to parse JWT_TOKEN_TIMEOUT from env");
        let timeout = Duration::new(timeout, 0);

        let default_max_wait_time = format!("{}", aws::sqs::DEFAULT_MAX_WAIT_TIME_IN_SEC);
        let sqs_max_wait_time = vars
            .get(AWS_SQS_MAX_WAIT_TIME_IN_SEC)
            .unwrap_or(&default_max_wait_time)
            .parse::<u64>()
            .expect("Failed to parse AWS_SQS_MAX_WAIT_TIME_IN_SEC from env");
        let sqs_max_wait_time = Duration::from_secs(sqs_max_wait_time);

        Ok(Config {
            aws_sqs_queue: vars
                .get(AWS_SQS_QUEUE)
                .ok_or(VarNotFound(AWS_SQS_QUEUE))?
                .clone(),
            aws_sqs_max_wait_time: sqs_max_wait_time,
            aws_thumbnails_bucket: vars
                .get(AWS_THUMBNAILS_BUCKET)
                .ok_or(VarNotFound(AWS_THUMBNAILS_BUCKET))?
                .clone(),
            aws_thumbnails_base_url: vars
                .get(AWS_THUMBNAILS_BASE_URL)
                .ok_or(VarNotFound(AWS_THUMBNAILS_BASE_URL))?
                .clone(),
            aws_media_bucket: vars
                .get(AWS_MEDIA_BUCKET)
                .ok_or(VarNotFound(AWS_MEDIA_BUCKET))?
                .clone(),
            aws_profile: vars
                .get(AWS_PROFILE)
                .unwrap_or(&gen_aws_default_profile())
                .clone(),
            aws_region: vars.get(AWS_REGION).ok_or(VarNotFound(AWS_REGION))?.clone(),
            database_host: vars
                .get(POSTGRESS_HOST)
                .ok_or(VarNotFound(POSTGRESS_HOST))?
                .clone(),
            database_name: vars
                .get(POSTGRESS_DATABASE)
                .ok_or(VarNotFound(POSTGRESS_DATABASE))?
                .clone(),
            database_username: vars
                .get(POSTGRESS_USERNAME)
                .ok_or(VarNotFound(POSTGRESS_USERNAME))?
                .clone(),
            database_password: vars
                .get(POSTGRESS_PASSWORD)
                .ok_or(VarNotFound(POSTGRESS_PASSWORD))?
                .clone(),
            database_dialect: gen_default_database_dialect(),
            jwt_secret: vars.get(JWT_SECRET).ok_or(VarNotFound(JWT_SECRET))?.clone(),
            jwt_token_timeout: timeout,
        })
    }
}

fn init_dotenv() -> Result<HashMap<String, String>, dotenv::Error> {
    dotenv::dotenv()?;
    Ok(dotenv::vars().collect::<HashMap<String, String>>())
}
