use crate::github;
use eyre::WrapErr;
use std::collections::{HashMap, HashSet};

/// Data about the state of Embark in general, to be used by multiple checks
/// across all open source projects.
/// This is fetched in advance to prevent multiple checks from having to fetch
/// the same data, which would be wasteful and run the risk of hitting rate
/// limits.
#[derive(Debug)]
pub struct Context {
    pub embark_github_organisation_members: HashSet<String>,
    pub embark_github_repos: HashMap<String, github::Repo>,
    pub rust_ecosystem_readme: String,
    pub opensource_website_projects: Vec<OpenSourceWebsiteDataProject>,
}

impl Context {
    pub async fn get(github_api_token: Option<String>) -> eyre::Result<Self> {
        let client = github::Client::new(github_api_token);

        let (
            opensource_website_projects,
            embark_github_organisation_members,
            embark_github_repos,
            rust_ecosystem_readme,
        ) = futures::join!(
            download_projects_list(),
            client.public_organisation_members("EmbarkStudios"),
            client.organisation_repos("EmbarkStudios"),
            github::download_repo_file("EmbarkStudios", "rust-ecosystem", "main", "README.md")
        );

        Ok(Self {
            embark_github_organisation_members: embark_github_organisation_members?,
            opensource_website_projects: opensource_website_projects?,
            rust_ecosystem_readme: rust_ecosystem_readme?,
            embark_github_repos: embark_github_repos?,
        })
    }

    /// Get the names of all projects found on GitHub and in the opensource
    /// website data.json. We lookup from both as a project may accidentally be
    /// in one but not the other.
    pub fn all_projects(&self) -> HashSet<String> {
        let website_projects = self.opensource_website_projects.iter().map(|proj| {
            proj.repo
                .as_ref()
                .and_then(|repo_url| repo_url.rfind('/').map(|i| &repo_url[i + 1..]))
                .unwrap_or(&proj.name)
                .to_owned()
        });
        let github_projects = self
            .embark_github_repos
            .values()
            .filter(|project| project.is_public_active_source_project())
            .map(|project| project.name.to_string());
        website_projects.chain(github_projects).collect()
    }
}

async fn download_projects_list() -> eyre::Result<Vec<OpenSourceWebsiteDataProject>> {
    let data = github::download_repo_json_file::<OpenSourceWebsiteData>(
        "EmbarkStudios",
        "opensource-website",
        "main",
        "static/data/data.json",
    )
    .await
    .wrap_err("Unable to get list of open source Embark projects")?;
    Ok(data.projects)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenSourceWebsiteData {
    pub projects: Vec<OpenSourceWebsiteDataProject>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenSourceWebsiteDataProject {
    pub name: String,
    pub repo: Option<String>,
    #[serde(default)]
    pub tags: HashSet<String>,
}
