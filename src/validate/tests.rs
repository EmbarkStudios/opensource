use std::collections::{HashMap, HashSet};

use super::*;

fn make_context() -> Context {
    Context {
        embark_github_organisation_members: HashSet::new(),
        embark_github_repos: HashMap::new(),
        rust_ecosystem_readme: "Readme!".to_string(),
        opensource_website_projects: Vec::new(),
    }
}

fn make_website_project(name: &str) -> OpenSourceWebsiteDataProject {
    OpenSourceWebsiteDataProject {
        name: name.to_string(),
        repo: None,
        tags: HashSet::new(),
    }
}

#[test]
fn check_website_data_inclusion_ok() {
    let name = "some-project";
    let project = Project::new(name.to_string());
    let mut context = make_context();

    // OK if the project is in the website data.json
    context
        .opensource_website_projects
        .push(make_website_project(name));
    assert!(project.check_website_data_inclusion(&context).is_ok())
}

#[test]
fn check_website_data_inclusion_ko() {
    let name = "some-project";
    let project = Project::new(name.to_string());
    let context = make_context();

    // Error if the project is not in the website data.json
    assert!(project.check_website_data_inclusion(&context).is_err());
}
