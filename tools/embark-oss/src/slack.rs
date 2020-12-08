use eyre::WrapErr;
use serde_json::json;

#[derive(Debug)]
pub enum Block {
    Divider,
    Text(String),
}

impl Block {
    pub fn to_json(self) -> serde_json::Value {
        match self {
            Self::Divider => json!({ "type": "divider" }),
            Self::Text(text) => json!({
                "type": "section",
                "text": { "type": "mrkdwn", "text": text }
            }),
        }
    }
}

fn blocks_json(blocks: Vec<Block>) -> serde_json::Value {
    let blocks = blocks.into_iter().map(|b| b.to_json()).collect();
    json!({
        "blocks": serde_json::Value::Array(blocks),
    })
}

pub async fn send_webhook(webhook_url: &str, blocks: Vec<Block>) -> eyre::Result<()> {
    reqwest::Client::new()
        .post(webhook_url)
        .json(&blocks_json(blocks))
        .send()
        .await
        .wrap_err("Unable to send webhook to Slack")?
        .error_for_status()
        .wrap_err("Unable to send webhook to Slack")
        .map(|_| ())
}
