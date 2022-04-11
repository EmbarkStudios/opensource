use super::context::Context;
use crate::github;
use eyre::{eyre, WrapErr};
use futures::TryFutureExt;
use itertools::Itertools;
use std::{collections::HashSet, ops::Not};

#[derive(Debug)]
pub struct Project {
    pub name: String,
    /// Projects must have a maintainer at Embark
    pub maintainers: eyre::Result<HashSet<String>>,
    /// Projects must not not have an archived repo
    archive_status: eyre::Result<()>,
    // Rust based projects must be included in the rust-ecosystem README.
    rust_ecosystem_registration: eyre::Result<()>,
    // Projects must be included in the opensource website data.json
    website_data_inclusion: eyre::Result<()>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            maintainers: not_yet_checked(),
            archive_status: not_yet_checked(),
            website_data_inclusion: not_yet_checked(),
            rust_ecosystem_registration: not_yet_checked(),
        }
    }

    pub async fn validate(self, context: &Context) -> Self {
        let maintainers = self.lookup_project_maintainers(context).await;
        let rust_ecosystem_registration = self.check_rust_ecosystem_registration(context);
        let archive_status = self.check_archive_status(context);
        let website_data_inclusion = self.check_website_data_inclusion(context);

        Self {
            name: self.name,
            maintainers,
            archive_status,
            website_data_inclusion,
            rust_ecosystem_registration,
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors().is_empty()
    }

    pub fn errors(&self) -> Vec<&eyre::Report> {
        let Self {
            name: _,
            maintainers,
            archive_status,
            website_data_inclusion,
            rust_ecosystem_registration,
        } = self;
        vec![
            maintainers.as_ref().err(),
            archive_status.as_ref().err(),
            website_data_inclusion.as_ref().err(),
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

    pub async fn lookup_project_maintainers(
        &self,
        context: &Context,
    ) -> eyre::Result<HashSet<String>> {
        // Download CODEOWNERS from one of the accepted branches
        let get = |branch| {
            github::download_repo_file("EmbarkStudios", &self.name, branch, ".github/CODEOWNERS")
        };
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
        let mut maintainers_not_in_embark = maintainers
            .difference(&context.embark_github_organisation_members)
            .filter(|user_name| {
                // filter out non-embark users that are explicitly allowed to be maintained
                crate::policy::ALLOWED_NON_EMBARK_MAINTAINERS
                    .iter()
                    .any(|a| a == user_name)
                    .not()
            })
            .peekable();
        if maintainers_not_in_embark.peek().is_some() {
            return Err(eyre!(
                "Maintainers not public EmbarkStudios members: {}",
                maintainers_not_in_embark.join(", "),
            ));
        }

        Ok(maintainers)
    }

    pub fn check_rust_ecosystem_registration(&self, context: &Context) -> eyre::Result<()> {
        let tags = match context
            .opensource_website_projects
            .iter()
            .find(|proj| proj.name == self.name)
        {
            Some(project) => &project.tags,
            None => return Ok(()),
        };
        if tags.contains("rust") && !context.rust_ecosystem_readme.contains(&self.name) {
            Err(eyre!("Rust project not in the rust-ecosystem README"))
        } else {
            Ok(())
        }
    }

    pub fn check_archive_status(&self, context: &Context) -> eyre::Result<()> {
        let repo = context.embark_github_repos.get(&self.name).ok_or_else(|| {
            eyre!("Unable to find project in the EmbarkStudios GitHub organisation")
        })?;
        if repo.archived {
            Err(eyre!("Project has been archived on GitHub"))
        } else {
            Ok(())
        }
    }

    pub fn check_website_data_inclusion(&self, context: &Context) -> eyre::Result<()> {
        if context
            .opensource_website_projects
            .iter()
            .any(|proj| proj.name == self.name)
        {
            Ok(())
        } else {
            Err(eyre!(
                "Project not included in opensource-website data.json"
            ))
        }
    }
}

fn not_yet_checked<T>() -> eyre::Result<T> {
    Err(eyre!("This property has not yet been validated"))
}
