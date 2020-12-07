use crate::{codeowners::CodeOwners, github};
use eyre::eyre;
use futures::TryFutureExt;
use itertools::Itertools;
use std::collections::HashSet;

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
    let result = github::download_repo_json_file::<OpenSourceWebsiteData>(
        "EmbarkStudios",
        "opensource-website",
        "main",
        "data.json",
    )
    .await;
    match result {
        Ok(Some(data)) => Ok(data.projects),
        Ok(None) => Err(eyre!("opensource-website/data.json not found")),
        Err(e) => Err(e),
    }
}

async fn lookup_project(project: OpenSourceWebsiteProject) -> Project {
    let name = project.name;
    Project {
        status: lookup_project_status(&name).await,
        name,
    }
}

async fn lookup_project_status(name: &str) -> Result<HashSet<String>, Error> {
    let get = |branch| lookup_project_status_for_branch(name, branch);
    get("main")
        .or_else(|_| get("master"))
        .or_else(|_| get("trunk"))
        .or_else(|_| get("develop"))
        .await
}

async fn lookup_project_status_for_branch(
    name: &str,
    branch: &str,
) -> Result<HashSet<String>, Error> {
    let result =
        github::download_repo_file("EmbarkStudios", name, branch, ".github/CODEOWNERS").await;
    let text = match result {
        Ok(Some(text)) => text,
        Ok(None) => return Err(Error::NoCodeOwnersFile),
        Err(e) => return Err(Error::UnknownError(e)),
    };

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
