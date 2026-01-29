use crate::types::ClipboardItem;
use anyhow::Result;
use reqwest::Client;
use serde::Serialize;

#[derive(Debug)]
pub struct ServerClient {
    base_url: String,
    api_key: String,
    client: Client,
}

#[derive(Serialize)]
struct UploadRequest {
    title: Option<String>,
    content: String,
    content_type: String,
    tags: Vec<String>,
    created_at: String,
}

impl ServerClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            client: Client::new(),
        }
    }

    pub async fn upload(&self, item: &ClipboardItem) -> Result<()> {
        let url = format!("{}/api/clipboard", self.base_url);

        let request = UploadRequest {
            title: item.title.clone(),
            content: item.content.clone(),
            content_type: format!("{:?}", item.content_type),
            tags: item.tags.clone(),
            created_at: item.created_at.to_rfc3339(),
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("Upload failed: {}", response.status())
        }
    }
}
