use eyre::{eyre, WrapErr};
use serde::de::DeserializeOwned;

pub async fn download_repo_file(
    org: &str,
    repo: &str,
    branch: &str,
    file: &str,
) -> eyre::Result<Option<String>> {
    match download_file(org, repo, branch, file).await? {
        None => Ok(None),
        Some((name, response)) => response
            .text()
            .await
            .map(Some)
            .wrap_err(eyre!("Failed to decode {}", name)),
    }
}

pub async fn download_repo_json_file<Json: DeserializeOwned>(
    org: &str,
    repo: &str,
    branch: &str,
    file: &str,
) -> eyre::Result<Option<Json>> {
    match download_file(org, repo, branch, file).await? {
        None => Ok(None),
        Some((name, response)) => response
            .json()
            .await
            .map(Some)
            .wrap_err(eyre!("Failed to decode {}", name)),
    }
}

pub async fn download_file(
    org: &str,
    repo: &str,
    branch: &str,
    file: &str,
) -> eyre::Result<Option<(String, reqwest::Response)>> {
    let name = format!("{}/{}/{}/{}", org, repo, branch, file);
    let url = format!("https://raw.githubusercontent.com/{}", name);
    let response = reqwest::get(&url)
        .await
        .wrap_err(format!("Failed to download {}", name))?;

    // Ensure the file was successfully downloaded
    if response.status() == 404 {
        return Ok(None);
    }
    if response.status() != 200 {
        return Err(eyre!("Expected status code 200, got {}", response.status()))
            .wrap_err(format!("Unable to download {}", name))?;
    }

    Ok(Some((name, response)))
}
