mod codeowners;

use std::collections::{HashMap, HashSet};

pub use codeowners::CodeOwners;

use eyre::{eyre, WrapErr};
use lazy_static::lazy_static;
use regex::Regex;
use serde::de::DeserializeOwned;

/// A GitHub API client that optionally authenticates requests.
pub struct Client {
    github_api_token: Option<String>,
}

impl Client {
    pub fn new(github_api_token: Option<String>) -> Self {
        Self { github_api_token }
    }

    // https://docs.github.com/en/free-pro-team@latest/rest/reference/orgs#members
    pub async fn public_organisation_members(
        &self,
        organisation: &str,
    ) -> eyre::Result<HashSet<String>> {
        #[derive(Debug, serde::Deserialize)]
        pub struct Member {
            login: String,
        }

        let url = format!(
            "https://api.github.com/orgs/{}/members?per_page=100",
            organisation
        );
        Ok(self
            .api_list(url)
            .await
            .wrap_err("Unable to get public members for organisation")?
            .into_iter()
            .map(|member: Member| member.login)
            .collect())
    }

    // https://docs.github.com/en/free-pro-team@latest/rest/reference/repos#list-organization-repositories
    pub async fn organisation_repos(
        &self,
        organisation: &str,
    ) -> eyre::Result<HashMap<String, Repo>> {
        let url = format!(
            "https://api.github.com/orgs/{}/repos?type=archived&per_page=100",
            organisation
        );
        Ok(self
            .api_list(url)
            .await
            .wrap_err("Unable to get archived repos for organisation")?
            .into_iter()
            .map(|repo: Repo| (repo.name.clone(), repo))
            .collect())
    }

    /// Perform a GET request to a paginated GitHub URL that returns a JSON array per
    /// page. All pages will be traversed and retuned as a single collection.
    async fn api_list<Json: DeserializeOwned>(&self, url: String) -> eyre::Result<Vec<Json>> {
        let mut collection = Vec::new();
        let mut next_url = Some(url);
        while let Some(url) = next_url {
            let response = self.api_get_response(&url).await?;
            next_url = next_pagination_page(&response)?;
            let items: Vec<Json> = response
                .json()
                .await
                .wrap_err("Unable to parse JSON response")?;
            collection.extend(items);
        }

        Ok(collection)
    }

    async fn api_get_response(&self, url: &str) -> eyre::Result<reqwest::Response> {
        let request = reqwest::Client::new()
            .get(url)
            .header("accept", "application/vnd.github.v3+json")
            .header("user-agent", "embark-oss");
        let request = match &self.github_api_token {
            Some(token) => request.header("authorization", format!("token {}", token)),
            _ => request,
        };
        let response = request
            .send()
            .await
            .wrap_err(format!("Failed to get {}", url))?
            .error_for_status()?;
        Ok(response)
    }
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
pub struct Repo {
    pub name: String,
    pub archived: bool,
    pub private: bool,
    pub fork: bool,
}

impl Repo {
    pub fn is_public_active_source_project(&self) -> bool {
        match self.name.as_str() {
            "opensource-template" | ".github" => false,
            _ => !(self.archived || self.private || self.fork),
        }
    }
}

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

fn next_pagination_page(response: &reqwest::Response) -> eyre::Result<Option<String>> {
    match response.headers().get("link") {
        None => Ok(None),
        Some(link) => Ok(next_pagination_page_from_link_header(link)
            .wrap_err("Unable to find next pagination page url in link header")?),
    }
}

fn next_pagination_page_from_link_header(
    header: &reqwest::header::HeaderValue,
) -> eyre::Result<Option<String>> {
    Ok(header
        .to_str()
        .wrap_err("Header was not valid unicode")?
        .split(',')
        .find_map(parse_next_link_url))
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
    use super::*;

    #[test]
    fn test_parse_next_link_url() {
        assert_eq!(parse_next_link_url(""), None);

        // A "rel" value other than "next" is not accepted
        assert_eq!(
            parse_next_link_url(r#"<https://example.com/members?page=2>; rel="last""#),
            None
        );

        assert_eq!(
            parse_next_link_url(r#"<https://example.com/members?page=2>; rel="next""#),
            Some("https://example.com/members?page=2".to_string())
        );

        assert_eq!(
            parse_next_link_url(r#"   <https://example.com/members?page=2>;    rel="next"    "#),
            Some("https://example.com/members?page=2".to_string())
        );
    }

    fn make_repo() -> Repo {
        Repo {
            name: "name".to_string(),
            archived: false,
            private: false,
            fork: false,
        }
    }

    #[test]
    fn repo_is_public_active_source_project() {
        let repo = make_repo();
        assert!(repo.is_public_active_source_project());

        // Archived are inactive
        let mut repo = make_repo();
        repo.archived = true;
        assert!(!repo.is_public_active_source_project());

        // Forks are inactive
        let mut repo = make_repo();
        repo.fork = true;
        assert!(!repo.is_public_active_source_project());

        // Private are inactive
        let mut repo = make_repo();
        repo.private = true;
        assert!(!repo.is_public_active_source_project());

        // .github is skipped as it's meta information
        let mut repo = make_repo();
        repo.name = ".github".to_string();
        assert!(!repo.is_public_active_source_project());

        // opensource-template is skipped as it cannot conform without making
        // the templates errors prone. i.e. forgetting to configure the project
        // means it is incorrectly detected as complying.
        let mut repo = make_repo();
        repo.name = "opensource-template".to_string();
        assert!(!repo.is_public_active_source_project());
    }
}
