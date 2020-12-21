mod context;
mod project;

use self::{context::*, project::Project};
use crate::{github, slack, ValidateAll};
use eyre::{eyre, WrapErr};
use itertools::Itertools;
use std::collections::HashSet;

/// Validate all projects listed in the data.json of the Embark Open Source
/// website.
pub(crate) async fn all(options: ValidateAll) -> eyre::Result<()> {
    let ValidateAll {
        slack_webhook_url,
        github_api_token,
    } = options;

    // Lookup required contextual information
    let context = Context::get(github_api_token).await?;

    // Download list of projects and download CODEOWNERS file for each one
    let projects = download_projects_list().await?;
    let futures = projects
        .into_iter()
        .map(|project| project.validate(&context));
    let projects = futures::future::join_all(futures).await;

    // Print results
    projects.iter().for_each(print_status);

    // Collected the projects with issues
    let problem_projects: Vec<_> = projects
        .into_iter()
        .filter(|project| project.has_errors())
        .collect();

    // If there is no problem we are done and can return
    if problem_projects.is_empty() {
        return Ok(());
    }

    // Send a message to slack if a webhook URL has been given
    if let Some(url) = slack_webhook_url {
        let blocks = slack_notification_blocks(problem_projects.as_slice());
        slack::send_webhook(&url, blocks).await?;
    }

    Err(eyre!("Not all projects conform to our guidelines"))
}

/// Validate a single project from the Embark Studios GitHub organisation.
pub async fn one(project_name: String) -> eyre::Result<()> {
    // Lookup required contextual information
    let context = Context::get(None).await?;

    // Validate project
    let project = Project::new(project_name, HashSet::new())
        .validate(&context)
        .await;
    print_status(&project);
    if project.has_errors() {
        Err(eyre!("The project does not conform to our guidelines"))
    } else {
        Ok(())
    }
}

fn print_status(project: &Project) {
    if let Some(errors) = project.errors_to_string(true) {
        return print!("❌ {}\n{}\n", project.name, errors);
    }

    if let Ok(maintainers) = &project.maintainers {
        return println!("✔️ {} ({})", project.name, maintainers.iter().join(", "));
    }

    unreachable!();
}

async fn download_projects_list() -> eyre::Result<Vec<Project>> {
    let data = github::download_repo_json_file::<OpenSourceWebsiteData>(
        "EmbarkStudios",
        "opensource-website",
        "main",
        "data.json",
    )
    .await
    .wrap_err("Unable to get list of open source Embark projects")?;
    Ok(data
        .projects
        .into_iter()
        .map(Project::from_website_project)
        .collect())
}

fn slack_notification_blocks(projects: &[Project]) -> Vec<slack::Block> {
    use slack::Block::{Divider, Text};

    let head = "The following Embark open source projects have been found to \
have maintainership issues.";
    let foot = "This message was generated by the \
<https://github.com/EmbarkStudios/opensource/tree/main/tools/embark-oss|embark-oss tool> \
on GitHub Actions.";

    let mut blocks = Vec::with_capacity(projects.len() + 4);

    blocks.push(Text(head.to_string()));
    blocks.push(Divider);
    blocks.extend(projects.iter().flat_map(slack_project_block));
    blocks.push(Divider);
    blocks.push(Text(foot.to_string()));
    blocks
}

fn slack_project_block(project: &Project) -> Option<slack::Block> {
    let text = format!(
        ":red_circle: *<https://github.com/EmbarkStudios/{name}|{name}>*\n```{error}```",
        name = &project.name,
        error = project.errors_to_string(false)?,
    );
    Some(slack::Block::Text(text))
}
