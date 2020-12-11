use eyre::{eyre, WrapErr};
use serde::de::DeserializeOwned;

pub async fn download_repo_file(
    org: &str,
    repo: &str,
    branch: &str,
    file: &str,
) -> eyre::Result<String> {
    let (name, response) = download_file(org, repo, branch, file).await?;
    response
        .text()
        .await
        .wrap_err(eyre!("Failed to decode {}", name))
}

pub async fn download_repo_json_file<Json: DeserializeOwned>(
    org: &str,
    repo: &str,
    branch: &str,
    file: &str,
) -> eyre::Result<Json> {
    let (name, response) = download_file(org, repo, branch, file).await?;
    response
        .json()
        .await
        .wrap_err(eyre!("Failed to decode {}", name))
}

pub async fn download_file(
    org: &str,
    repo: &str,
    branch: &str,
    file: &str,
) -> eyre::Result<(String, reqwest::Response)> {
    let path = format!("{}/{}/{}/{}", org, repo, branch, file);
    let name = format!("{}/{}:{}", org, repo, file);
    let url = format!("https://raw.githubusercontent.com/{}", path);
    let response = reqwest::get(&url)
        .await
        .wrap_err(format!("Failed to download {}", name))?;

    // Ensure the file was successfully downloaded
    if response.status() == 404 {
        return Err(eyre!("File not found in repo"))
            .wrap_err(format!("Unable to download {}", name))?;
    }
    if response.status() != 200 {
        return Err(eyre!("Expected status code 200, got {}", response.status()))
            .wrap_err(format!("Unable to download {}", name))?;
    }

    Ok((name, response))
}
