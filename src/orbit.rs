//! A module for interacting with <https://orbit.love/>

// TODO: remove
#![allow(unused)]

use eyre::{Result, WrapErr};
use reqwest::{IntoUrl, Method};
use serde_json::{json, Value as Json};

#[derive(Debug)]
pub struct Client {
    api_token: String,
}

impl Client {
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }

    pub async fn current_user_workspaces(&self) -> Result<Vec<Workspace>> {
        self.request(
            Method::GET,
            "https://app.orbit.love/api/v1/workspaces",
            None,
        )
        .await
    }

    async fn request<ResponseJson: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        url: impl IntoUrl,
        body: Option<&Json>,
    ) -> Result<ResponseJson> {
        let request = reqwest::Client::new()
            .request(method, url)
            .header("accept", "application/json")
            .header("authorization", format!("Bearer {}", self.api_token));
        let request = match body {
            None => request,
            Some(json) => request.json(json),
        };
        request
            .send()
            .await
            .wrap_err("Unable to make HTTP request to Orbit API")?
            .error_for_status()
            .wrap_err("Got unexpected response from Orbit API")?
            .json()
            .await
            .wrap_err("Got expected response body from Orbit API")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Workspace {
    id: usize,
    name: String,
    slug: String,
}
