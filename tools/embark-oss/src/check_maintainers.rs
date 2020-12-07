use crate::{codeowners::CodeOwners, github};
use eyre::eyre;
use futures::TryFutureExt;
use itertools::Itertools;
use std::collections::HashSet;

#[derive(Debug)]
struct Project {
    name: String,
    status: Result<HashSet<String>, eyre::Report>,
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
            println!("   {:?}", error);
        }
    }
}

async fn download_projects_list() -> eyre::Result<Vec<OpenSourceWebsiteProject>> {
    let data = github::download_repo_json_file::<OpenSourceWebsiteData>(
        "EmbarkStudios",
        "opensource-website",
        "main",
        "data.json",
    )
    .await?;
    Ok(data.projects)
}

async fn lookup_project(project: OpenSourceWebsiteProject) -> Project {
    let name = project.name;
    Project {
        status: lookup_project_status(&name).await,
        name,
    }
}

async fn lookup_project_status(name: &str) -> eyre::Result<HashSet<String>> {
    // Download CODEOWNERS from one of the accepted branches
    let get =
        |branch| github::download_repo_file("EmbarkStudios", name, branch, ".github/CODEOWNERS");
    let text = get("main").or_else(|_| get("master")).await?;

    // Determine if there is at least 1 primary maintainer listed for each project
    CodeOwners::new(&text)?
        .primary_maintainers()
        .cloned()
        .ok_or(eyre!("No maintainers were found for * the CODEOWNERS file"))
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenSourceWebsiteData {
    projects: Vec<OpenSourceWebsiteProject>,
}

#[derive(Debug, serde::Deserialize)]
pub struct OpenSourceWebsiteProject {
    name: String,
}
