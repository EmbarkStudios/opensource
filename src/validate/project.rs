use super::context::{Context, OpenSourceWebsiteDataProject};
use crate::github;
use eyre::{eyre, WrapErr};
use futures::TryFutureExt;
use itertools::Itertools;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Project {
    pub name: String,
    tags: HashSet<String>,
    /// Projects must have a maintainer at Embark
    pub maintainers: eyre::Result<HashSet<String>>,
    /// Projects must not be archived
    archive_status: eyre::Result<()>,
    // Rust based projects must be included in the rust-ecosystem README.
    rust_ecosystem_registration: eyre::Result<()>,
}

impl Project {
    pub fn new(name: String, tags: HashSet<String>) -> Self {
        Self {
            name,
            tags,
            maintainers: not_yet_checked(),
            archive_status: not_yet_checked(),
            rust_ecosystem_registration: not_yet_checked(),
        }
    }

    pub fn from_website_project(project: OpenSourceWebsiteDataProject) -> Self {
        Self::new(project.name, project.tags)
    }

    pub async fn validate(self, context: &Context) -> Self {
        let maintainers =
            lookup_project_maintainers(&self.name, &context.embark_github_organisation_members)
                .await;

        let rust_ecosystem_registration =
            check_rust_ecosystem_registration(&self.name, &self.tags, context);

        let archive_status = check_archive_status(&self.name, context);

        Self {
            name: self.name,
            tags: self.tags,
            maintainers,
            archive_status,
            rust_ecosystem_registration,
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors().is_empty()
    }

    pub fn errors(&self) -> Vec<&eyre::Report> {
        let Self {
            name: _,
            tags: _,
            maintainers,
            archive_status,
            rust_ecosystem_registration,
        } = self;
        vec![
            maintainers.as_ref().err(),
            archive_status.as_ref().err(),
            rust_ecosystem_registration.as_ref().err(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn errors_to_string(&self, indent: bool) -> Option<String> {
        let errors = self.errors();
        if errors.is_empty() {
            return None;
        }
        Some(
            errors
                .into_iter()
                .map(|error| crate::error::cause_string(error.as_ref(), indent))
                .join("\n"),
        )
    }
}

fn not_yet_checked<T>() -> eyre::Result<T> {
    Err(eyre!("This property has not yet been validated"))
}

async fn lookup_project_maintainers(
    name: &str,
    members: &HashSet<String>,
) -> eyre::Result<HashSet<String>> {
    // Download CODEOWNERS from one of the accepted branches
    let get =
        |branch| github::download_repo_file("EmbarkStudios", name, branch, ".github/CODEOWNERS");
    let text = get("main")
        .or_else(|_| get("master"))
        .await
        .wrap_err("Unable to determine maintainers")?;

    // Determine if there is at least 1 primary maintainer listed for each project
    let maintainers = github::CodeOwners::new(&text)
        .wrap_err("Unable to determine maintainers")?
        .primary_maintainers()
        .cloned()
        .ok_or_else(|| eyre!("No maintainers were found for * the CODEOWNERS file"))?;

    // Ensure all maintainers are in the EmbarkStudios organisation
    let mut maintainers_not_in_embark = maintainers.difference(members).peekable();
    if maintainers_not_in_embark.peek().is_some() {
        return Err(eyre!(
            "Maintainers not public EmbarkStudios members: {}",
            maintainers_not_in_embark.join(", "),
        ));
    }

    Ok(maintainers)
}

fn check_rust_ecosystem_registration(
    name: &str,
    tags: &HashSet<String>,
    context: &Context,
) -> eyre::Result<()> {
    if tags.contains("rust") && !context.rust_ecosystem_readme.contains(name) {
        Err(eyre!("Rust project not in the rust-ecosystem README"))
    } else {
        Ok(())
    }
}

fn check_archive_status(name: &str, context: &Context) -> eyre::Result<()> {
    let repo = context
        .embark_github_repos
        .get(name)
        .ok_or_else(|| eyre!("Unable to find project in the EmbarkStudios GitHub organisation"))?;
    if repo.archived {
        Err(eyre!("Project has been archived on GitHub"))
    } else {
        Ok(())
    }
}
