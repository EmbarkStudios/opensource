# How to Open Source at Embark

> This document covers the basics of working with open source projects at Embark

## New Projects

### Should I open-source it?

You've got a new project - great! At Embark, we believe most things should be open-sourced by default, but there are a few questions you should ask yourself before publishing.

- **Is it useful to someone else?**
  - The answer is usually yes (even for small or "obvious" things) but consider whether your project could be useful to someone outside the organisation.

- **Does it rely on internal or proprietary systems?**
  - We don't want to release anything that could harm Embark from a security or intellectual property perspective. We also don't want to release something that doesn't work for non-Embarkers because it relies on an internal system. **You need to consult with your manager to assess any risks associated with releasing your project.**

- **Can you commit to maintaining the project for the foreseeable future?**
  - See the [maintenance guidelines](maintenance-guidelines.md) to see what is required from an Embark open source maintainer.
  - If no, you can still release ["as-is"](#repository-types) in an archived GitHub repository.

- **Is it high enough quality?**
  - Not every project will be mindblowing and innovative (and that's perfectly ok!), but our open source work is still a representation of the company. All repositories should have a minimum level of documentation and the code should be clean and readable. The community team and your manager should review the repository before release and suggest improvements.
  - Keep an eye out for stray comments and commit messages - the public will see them!

### Step-by-step

1. Create a new private or internal repository on GitHub using the [open source template](https://github.com/EmbarkStudios/opensource-template). If you already have a repository and want to maintain the commit history, you can instead add all the files from the template repository to your project.
1. Edit the `.github/CODEOWNERS` file to contain the GitHub usernames of the project maintainers.
1. If the project is a Rust project edit `.github/workflows/rust-ci.yml` to your needs, resolving all the comments marked `TODO`.
1. If the project is not a Rust project remove the `.github/workflows/rust-ci.yml` file.
1. Customise the README for your project by adding the appropriate name, description, links, and badges. This is also a great time to pick an emoji for the project!
1. Add the [EmbarkStudios / Open Source Admins](https://github.com/orgs/EmbarkStudios/teams/open-source-admins) group as admins in the repo access settings. Ask for assistance on slack if you do not have access to the repo settings.
1. Post the private repo link with a description of what it is to the Embark #opensource Slack channel and ask for green-light to go ahead. Do @mention your manager also for visibility.
1. Make the repository public.
1. Add the project to the [embark.dev list of open source projects](https://github.com/EmbarkStudios/opensource-website/blob/main/static/data/data.json).
1. If the project is a Rust project add it to the [embark.rs list of open source Rust projects](https://github.com/EmbarkStudios/rust-ecosystem#open-source).
1. Announce the release on [our Discord](https://discord.gg/dAuKfZS) opensource channel and any other forums such as [/r/rust](https://reddit.com/r/rust). The ecosystem team can help you with this step. They will have the "community team" role on the Discord.

## Publishing new versions

If the project is a Rust project and the steps above have been completed then new versions can be released by following these steps.

1. Update the `## Unreleased` section to include all of the relevant/important changes since the last release
1. Install [`cargo-release`](https://github.com/crate-ci/cargo-release) if you have not already done so.
1. Run `cargo release --execute <major|minor|patch|exact>` which will do the following
    1. Bump the crate version to the next `<major|minor|patch>` or `<exact>` version.
    1. Replace any `pre-release-replacements` with the new version.
    1. Commit
    1. Publishes to crates.io (you can provide your API token with `--token` if you have a non-Embark account as your default)
    1. Tags the commit with the version message specified in release.toml
    1. Pushes the commits and tags

See the [cargo-release docs](https://github.com/crate-ci/cargo-release/tree/master/docs) if you need to do something more advanced or just different.

## Repository Types

**maintained**: this repository has a maintainer who is actively developing it and reviewing contributions

**as-is**: this repository could still be useful, but doesn't have an active maintainer. It is archived on GitHub.

## Communication

### Email

We receive incoming email at opensource@embark-studios.com. The community team is responsible for answering or forwarding to the relevant Embarkers.

### Discord

We have a [public Discord](https://discord.gg/8TW9nfF) for the developer community. There are channels here specifically for open source projects we maintain, which can be helpful for coordinating work, getting feedback, or talking to contributors. Not every project needs its own channel, but if you are a maintainer and you want one to be created for your project, message a Discord admin!

### Newsletter

We publish a monthly developer newsletter which covers updates on our open source work. You can [view the archive here](https://www.embark.dev/newsletter/). As a maintainer, you can let #opensource team on Slack or Discord know about updates you'd like to be included.
