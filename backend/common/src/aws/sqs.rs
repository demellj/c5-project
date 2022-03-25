use std::{error::Error, fmt::Display};

use aws_config::{self, meta::region::RegionProviderChain};
use aws_sdk_sqs::output::DeleteMessageOutput;
use aws_sdk_sqs::{Client, Region};

use crate::config::Config;

use std::time::Duration;

pub use aws_sdk_sqs::output::ReceiveMessageOutput;
pub use aws_sdk_sqs::output::SendMessageOutput;

pub static DEFAULT_MAX_WAIT_TIME_IN_SEC: u64 = 20;

pub struct SQSQueue {
    queue_url: String,
    client: Client,
}

#[derive(Debug)]
pub struct QueueNotFoundError;

impl SQSQueue {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let region_provider =
            RegionProviderChain::first_try(Region::new(config.aws_region.clone()))
                .or_default_provider()
                .or_else("us-east-1");

        let shared_config = aws_config::from_env().region(region_provider).load().await;

        let client = Client::new(&shared_config);

        let queues = client
            .list_queues()
            .queue_name_prefix(&config.aws_sqs_queue)
            .send()
            .await?;

        let urls = queues.queue_urls().unwrap_or_default();
        let queue_url = urls.first().ok_or(QueueNotFoundError)?;
        let queue_url = queue_url.clone();

        Ok(SQSQueue { queue_url, client })
    }

    pub async fn send(
        &self,
        msg_body: &String,
        msg_group_id: &String,
    ) -> Result<SendMessageOutput, Box<dyn Error>> {
        let rsp = self
            .client
            .send_message()
            .queue_url(&self.queue_url)
            .message_body(msg_body)
            .message_group_id(msg_group_id)
            .send()
            .await?;

        Ok(rsp)
    }

    pub async fn receive(
        &self,
        max_wait_time: Duration,
    ) -> Result<ReceiveMessageOutput, Box<dyn Error>> {
        let rcv_message_output = self
            .client
            .receive_message()
            .wait_time_seconds(max_wait_time.as_secs() as i32)
            .queue_url(&self.queue_url)
            .send()
            .await?;

        Ok(rcv_message_output)
    }

    pub async fn delete_message(
        &self,
        receipt_handle: impl Into<String>,
    ) -> Result<DeleteMessageOutput, Box<dyn Error>> {
        let result = self
            .client
            .delete_message()
            .queue_url(&self.queue_url)
            .receipt_handle(receipt_handle)
            .send()
            .await?;

        Ok(result)
    }
}

impl Display for QueueNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SQS Queue not Found")
    }
}

impl Error for QueueNotFoundError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
