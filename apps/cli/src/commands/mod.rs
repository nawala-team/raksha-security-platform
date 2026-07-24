pub mod status;
pub mod agents;
pub mod alerts;
pub mod config;
pub mod scan;

use serde::Deserialize;

/// Shared API client for CLI commands.
pub struct ApiClient {
    pub client: reqwest::Client,
    pub base_url: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String, token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { client, base_url, token }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()).into());
        }
        Ok(resp.json().await?)
    }

    pub async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()).into());
        }
        Ok(resp.json().await?)
    }

    pub async fn delete(
        &self,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()).into());
        }
        Ok(())
    }
}
