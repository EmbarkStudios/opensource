# Embark OSS tooling

This program is used to validate that our open source projects adhere to our open source guidelines.

## Periodic jobs

### Project validation

Every weekday the [`validate-all`](#cargo-run-validate-all) command is run on GitHub actions, sending us a Slack notification if any problems are found.

## Commands

### `cargo run validate-all`

This command checks to see if every project listed in the Embark [opensource-website data.json][data.json] conforms to our open source guidelines to the extent that this tool can detect.

[data.json]: https://github.com/EmbarkStudios/opensource-website/blob/main/data.json

#### Flags

- `--slack-webhook-url`: An optional Slack webhook URL that is used to report problems.
- `--github-api-token`: An optional API token used to raise the rate limit of the GitHub API. Likely only needed on CI where we share an IP with other GitHub API users.

### `cargo run validate PROJECT_REPO_NAME`

This command checks to see if a given Embark open source project conforms to our open source guidelines to the extent that this tool can detect.

## Testing

This tool has unit tests. Run them like so:

```shell
cargo test
```
