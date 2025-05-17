use futures_util::StreamExt;
use serde_json::de::Read;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
};
use serde::{Deserialize, Serialize};
use std::{time::Duration};
use tokio::{task, time};
use tokio_amqp::LapinTokioExt;
use reqwest::Client;
use anyhow::{Context, Result};
use tracing::{info, error};

#[derive(Debug, Deserialize, Serialize)]
pub struct SlackMessage {
    text: String,
}

pub struct AmqpConfig {
    pub uri: String,
    pub queue_name: String,
    pub slack_webhook: String,
}

pub struct AmqpClient {
    channel: Channel,
    slack_http: Client,
    cfg: AmqpConfig,
}

impl AmqpClient {
    pub async fn new(cfg: AmqpConfig) -> Result<Self> {
        let conn = Connection::connect(
            &cfg.uri,
            ConnectionProperties::default().with_tokio(),
        )
            .await
            .context("RabbitMQ 연결 실패")?;
        info!("RabbitMQ에 연결됨");

        let channel = conn.create_channel().await.context("채널 생성 실패")?;

        // queue
        channel
            .queue_declare(
                &cfg.queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .context("큐 선언 실패")?;
        info!("큐 '{}' 선언됨", &cfg.queue_name);

        let slack_http = Client::new();

        Ok(Self {
            channel,
            slack_http,
            cfg,
        })
    }

    pub async fn publish_slack_message(&self, msg: &SlackMessage) -> Result<()> {
        let payload = serde_json::to_vec(msg)?;
        self.channel
            .basic_publish(
                "",
                &self.cfg.queue_name,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await?
            .await?;
        info!("메시지 발행 완료: {:?}", msg);
        Ok(())
    }

    pub async fn start_consumer(&self) -> Result<()> {
        let mut consumer = self
            .channel
            .basic_consume(
                &self.cfg.queue_name,
                "slack_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .context("컨슈머 등록 실패")?;
        info!("Consumer 등록됨, 대기 중...");

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let channel = self.channel.clone();
                    let slack_http = self.slack_http.clone();
                    let webhook = self.cfg.slack_webhook.clone();
                    task::spawn(async move {
                        let data = delivery.data.clone();
                        let delivery_tag = delivery.delivery_tag;

                        match serde_json::from_slice::<SlackMessage>(&data) {
                            Ok(msg) => {
                                if let Err(err) = send_to_slack(&slack_http, &webhook, &msg).await {
                                    error!("Slack 전송 실패: {:?}", err);
                                    if let Err(e) = channel
                                        .basic_nack(delivery_tag, BasicNackOptions { requeue: true, ..Default::default() })
                                        .await
                                    {
                                        error!("Nack 실패: {:?}", e);
                                    }
                                } else {
                                    if let Err(e) = channel
                                        .basic_ack(delivery_tag, BasicAckOptions::default())
                                        .await
                                    {
                                        error!("Ack 실패: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("메시지 파싱 실패: {:?}", e);
                                let _ = channel
                                    .basic_nack(delivery_tag, BasicNackOptions { requeue: false, ..Default::default() })
                                    .await;
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Consumer 오류: {:?}", e);
                    time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        Ok(())
    }
}

async fn send_to_slack(client: &Client, webhook: &str, msg: &SlackMessage) -> Result<()> {
    let res = client
        .post(webhook)
        .json(msg)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .context("Slack API 요청 실패")?;

    if res.status().is_success() {
        info!("Slack 전송 성공: {:?}", msg);
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Slack 전송 실패, 상태 코드: {}",
            res.status()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::SlackMessage;
    use serde_json;

    #[test]
    fn slack_message_serde_round_trip() {
        let orig = SlackMessage { text: "hello".into() };
        let json = serde_json::to_string(&orig).unwrap();
        let de: SlackMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(de.text, orig.text);
    }
}
