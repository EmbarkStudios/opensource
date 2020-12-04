use crate::codeowners::CodeOwners;
use eyre::{eyre, WrapErr};
use itertools::Itertools;
use std::collections::HashSet;

const OPENSOURCE_WEBSITE_DATA_URL: &str =
    "https://raw.githubusercontent.com/EmbarkStudios/opensource-website/main/data.json";

#[derive(Debug)]
enum Error {
    NoPrimaryMaintainer,
    NoCodeOwnersFile,
    UnknownError(eyre::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPrimaryMaintainer => {
                write!(f, "No maintainers were found for * the CODEOWNERS file.")
            }
            Self::NoCodeOwnersFile => {
                write!(f, "No CODEOWNERS file could be read for this project.")
            }
            Self::UnknownError(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
struct Project {
    name: String,
    status: Result<HashSet<String>, Error>,
}

pub async fn main() -> eyre::Result<()> {
    // Download list of projects and download CODEOWNERS file for each one
    let projects = download_projects_list().await?;
    let futures = projects.into_iter().map(lookup_project);
    let projects = futures::future::join_all(futures).await;

    // Print results
    projects.iter().for_each(print_status);

    if projects.iter().any(|p| p.status.is_err()) {
        std::process::exit(1);
    };
    Ok(())
}

fn print_status(Project { name, status }: &Project) {
    match status {
        Ok(maintainers) => println!("✔️ {} ({})", name, maintainers.iter().join(", ")),
        Err(error) => {
            println!("❌ {}", name);
            println!("   {}", error);
        }
    }
}

async fn download_projects_list() -> eyre::Result<Vec<OpenSourceWebsiteProject>> {
    let response = reqwest::get(OPENSOURCE_WEBSITE_DATA_URL)
        .await
        .wrap_err("Failed to download Embark open source website data.json")?;

    if response.status() != 200 {
        return Err(eyre!("Expected status code 200, got {}", response.status()))
            .wrap_err("Unable to download Embark open source website data.json");
    }

    Ok(response
        .json::<OpenSourceWebsiteData>()
        .await
        .wrap_err("Failed to parse OSS website data.json")?
        .projects)
}

async fn lookup_project(project: OpenSourceWebsiteProject) -> Project {
    let name = project.name;
    Project {
        status: lookup_project_status(&name).await,
        name,
    }
}

async fn lookup_project_status(name: &str) -> Result<HashSet<String>, Error> {
    let url = format!(
        "https://raw.githubusercontent.com/EmbarkStudios/{}/main/.github/CODEOWNERS",
        name
    );
    // Download the CODEOWNERS file
    let response = reqwest::get(&url)
        .await
        .wrap_err(format!("Failed to download CODEOWNERS for {}", name))
        .map_err(Error::UnknownError)?;

    // Ensure the CODEOWNERS file was successfully downloaded
    if response.status() == 404 {
        return Err(Error::NoCodeOwnersFile);
    }
    if response.status() != 200 {
        return Err(eyre!("Expected status code 200, got {}", response.status()))
            .wrap_err("Unable to download Embark open source website data.json")
            .map_err(Error::UnknownError)?;
    }

    // Parse the body to get the CODEOWNERS file contents
    let text = response
        .text()
        .await
        .wrap_err(format!("Failed to get CODEOWNERS text body for {}", name))
        .map_err(Error::UnknownError)?;

    // Determine if there is at least 1 primary maintainer listed for each project
    CodeOwners::new(&text)
        .primary_maintainers()
        .cloned()
        .ok_or(Error::NoPrimaryMaintainer)
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenSourceWebsiteData {
    projects: Vec<OpenSourceWebsiteProject>,
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenSourceWebsiteProject {
    name: String,
}
