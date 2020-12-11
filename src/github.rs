mod codeowners;

use std::collections::HashSet;

pub use codeowners::CodeOwners;

use eyre::{eyre, WrapErr};
use lazy_static::lazy_static;
use regex::Regex;
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
        return Err(eyre!("Expected status code 200, got 404"))
            .wrap_err("File not found in repo")
            .wrap_err(format!("Unable to download {}", name))?;
    }
    if response.status() != 200 {
        return Err(eyre!("Expected status code 200, got {}", response.status()))
            .wrap_err(format!("Unable to download {}", name))?;
    }

    Ok((name, response))
}

pub async fn public_organisation_members(organisation: &str) -> eyre::Result<HashSet<String>> {
    #[derive(Debug, serde::Deserialize)]
    pub struct Member {
        login: String,
    }

    let url = format!(
        "https://api.github.com/orgs/{}/members?per_page=100",
        organisation
    );
    Ok(api_list(url)
        .await
        .wrap_err("Unable to get public members for organisation")?
        .into_iter()
        .map(|member: Member| member.login)
        .collect())
}

/// Perform a GET request to a paginated GitHub URL that returns a JSON array per
/// page. All pages will be traversed and retuned as a single collection.
pub async fn api_list<Json: DeserializeOwned>(url: String) -> eyre::Result<Vec<Json>> {
    let mut collection = Vec::new();
    let mut next_url = Some(url);
    while let Some(url) = next_url {
        let response = api_get_response(&url).await?;
        next_url = next_pagination_page(&response)?;
        let items: Vec<Json> = response
            .json()
            .await
            .wrap_err("Unable to parse JSON response")?;
        collection.extend(items);
    }

    Ok(collection)
}

async fn api_get_response(url: &str) -> eyre::Result<reqwest::Response> {
    Ok(reqwest::Client::new()
        .get(url)
        .header("accept", "application/vnd.github.v3+json")
        .header("user-agent", "embark-oss")
        .send()
        .await
        .wrap_err(format!("Failed to get {}", url))?
        .error_for_status()?)
}

fn next_pagination_page(response: &reqwest::Response) -> eyre::Result<Option<String>> {
    match response.headers().get("link") {
        None => Ok(None),
        Some(link) => Ok(Some(
            next_pagination_page_from_link_header(link)
                .wrap_err("Unable to find next pagination page url in link header")?,
        )),
    }
}

fn next_pagination_page_from_link_header(
    header: &reqwest::header::HeaderValue,
) -> eyre::Result<String> {
    header
        .to_str()?
        .split(',')
        .find_map(parse_next_link_url)
        .ok_or_else(|| eyre!("Could not determine `next` url in header"))
}

fn parse_next_link_url(content: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"<(?P<url>.+)>; *rel="next""#).unwrap();
    }
    RE.captures(content)?
        .name("url")
        .map(|s| s.as_str().to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_next_link_url() {
        assert_eq!(super::parse_next_link_url(""), None);

        // A "rel" value other than "next" is not accepted
        assert_eq!(
            super::parse_next_link_url(r#"<https://example.com/members?page=2>; rel="last""#),
            None
        );

        assert_eq!(
            super::parse_next_link_url(r#"<https://example.com/members?page=2>; rel="next""#),
            Some("https://example.com/members?page=2".to_string())
        );

        assert_eq!(
            super::parse_next_link_url(
                r#"   <https://example.com/members?page=2>;    rel="next"    "#
            ),
            Some("https://example.com/members?page=2".to_string())
        );
    }
}
