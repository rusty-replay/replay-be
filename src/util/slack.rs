use reqwest::Client;
use serde_json::json;

pub async fn send_slack_alert(webhook_url: &str, message: &str) -> Result<(), reqwest::Error> {
    let payload = json!({ "text": message });

    Client::new()
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?; // 2xx 아닌 경우 Err

    Ok(())
}
