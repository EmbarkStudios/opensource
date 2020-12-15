use crate::github;
use std::collections::HashSet;

/// Data about the state of Embark in general, to be used by multiple checks
/// across all open source projects.
/// This is fetched in advance to prevent multiple checks from having to fetch
/// the same data, which would be wasteful and run the risk of hitting rate
/// limits.
#[derive(Debug)]
pub struct Context {
    pub embark_github_organisation_members: HashSet<String>,
    pub rust_ecosystem_readme: String,
}

impl Context {
    pub async fn get() -> eyre::Result<Self> {
        let embark_github_organisation_members =
            github::public_organisation_members("EmbarkStudios").await?;

        let rust_ecosystem_readme =
            github::download_repo_file("EmbarkStudios", "rust-ecosystem", "main", "README.md")
                .await?;

        Ok(Self {
            embark_github_organisation_members,
            rust_ecosystem_readme,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenSourceWebsiteData {
    pub projects: Vec<OpenSourceWebsiteDataProject>,
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenSourceWebsiteDataProject {
    pub name: String,
    #[serde(default)]
    pub tags: HashSet<String>,
}
