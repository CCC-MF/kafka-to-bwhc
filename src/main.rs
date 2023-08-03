/*
 * This file is part of ETL-Processor
 *
 * Copyright (c) 2023  Comprehensive Cancer Center Mainfranken
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

mod mtbfile;
mod bwhc_client;

use std::env;
use std::error::Error;
use std::fmt::{Debug as FmtDebug, Display, Formatter};
use std::str::FromStr;
use std::time::Duration;

use log::{debug, info, warn};
use log::LevelFilter::Debug;
use rdkafka::{ClientConfig, ClientContext, Message, TopicPartitionList};
use rdkafka::consumer::{Consumer, ConsumerContext, Rebalance, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::json;
use simple_logger::SimpleLogger;

use crate::AppError::{ConnectionError, HttpError, MissingConfig};
use crate::bwhc_client::{BwhcClient, HttpResponse};
use crate::mtbfile::MTBFileWithConsent;

struct CustomContext;

impl ClientContext for CustomContext {}

type LoggingConsumer = StreamConsumer<CustomContext>;

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        debug!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        debug!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        debug!("Committing offsets: {:?}", result);
    }
}

pub enum AppError {
    ConnectionError(String),
    MissingConfig(String),
    HttpError(String),
}

impl Error for AppError {}

impl FmtDebug for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ConnectionError(s) => write!(f, "ConnectionError: {}", s),
            MissingConfig(s) => write!(f, "Missing config: {}", s),
            HttpError(s) => write!(f, "HTTP error: {}", s),
        }
    }
}

enum KafkaResponsePayload {
    SuccessfulConnection(HttpResponse),
    NoConnection
}

impl KafkaResponsePayload {
    fn to_payload(&self) -> String {
        match self {
            KafkaResponsePayload::SuccessfulConnection(s) => format!(
                "{{\"status_code\":{}, \"status_body\":{}}}",
                s.status_code,
                if s.status_body.trim().is_empty() {
                    String::from("{}")
                } else {
                    s.status_body.clone()
                }
            ),
            KafkaResponsePayload::NoConnection => json!({
                "status_code": 900,
                "status_body" : {
                    "issues": [{
                        "severity": "error",
                        "message": "No HTTP connection"
                    }]
                }
            }).to_string()
        }
    }
}

async fn send_kafka_response(producer: &FutureProducer, topic: &str, key: &str, payload: KafkaResponsePayload) {
    if let Err(e) = producer.send(
        FutureRecord::to(topic)
            .key(key)
            .payload(payload.to_payload().as_str()),
        Duration::from_secs(1),
    ).await {
        warn!("Response not sent: {}", e.0)
    };
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    SimpleLogger::new().with_level(Debug).init().unwrap();

    #[cfg(not(debug_assertions))]
    SimpleLogger::new().with_level(Info).init().unwrap();

    let context = CustomContext;

    match env::var("APP_REST_URI") {
        Ok(_) => { /* OK */ }
        Err(_) => panic!("Missing configuration 'APP_REST_URI'")
    }

    let boostrap_servers = env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or("kafka:9092".into());
    let src_topic = env::var("APP_KAFKA_TOPIC").unwrap_or("etl-processor".into());
    let dst_topic = env::var("APP_KAFKA_RESPONSE_TOPIC").unwrap_or(format!("{}_response", src_topic));
    let group_id = env::var("APP_KAFKA_GROUP_ID").unwrap_or(format!("{}_group", src_topic));

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", boostrap_servers.as_str())
        .set("auto.offset.reset", "earliest")
        .create_with_context(context)
        .expect("Kafka consumer created");

    consumer
        .subscribe([src_topic.as_str()].as_ref())
        .map_err(|e| ConnectionError(e.to_string()))?;

    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", boostrap_servers.as_str())
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    info!("Application started");

    loop {
        match consumer.recv().await {
            Ok(msg) => match msg.payload_view::<str>() {
                Some(Ok(s)) => {
                    match msg.key_view::<str>() {
                        Some(Ok(key)) => {
                            if let Ok(with_consent) = MTBFileWithConsent::from_str(s) {
                                if with_consent.has_consent() {
                                    match BwhcClient::send_mtb_file(s).await {
                                        Ok(response) => {
                                            send_kafka_response(producer, dst_topic.as_str(), key, KafkaResponsePayload::SuccessfulConnection(response)).await
                                        }
                                        Err(_) => {
                                            send_kafka_response(producer, dst_topic.as_str(), key, KafkaResponsePayload::NoConnection).await
                                        }
                                    }
                                } else {
                                    match BwhcClient::send_delete(with_consent.patient_id().as_str()).await {
                                        Ok(response) => {
                                            send_kafka_response(producer, dst_topic.as_str(), key, KafkaResponsePayload::SuccessfulConnection(response)).await
                                        }
                                        Err(_) => {
                                            send_kafka_response(producer, dst_topic.as_str(), key, KafkaResponsePayload::NoConnection).await
                                        }
                                    }
                                }
                            } else {
                                warn!("Cannot parse MTB File for consent")
                            }
                        }
                        _ => warn!("Unable to use key!")
                    }
                }
                _ => warn!("Unable to use payload!")
            }
            _ => warn!("Unable to consume message"),
        }
    }
}
