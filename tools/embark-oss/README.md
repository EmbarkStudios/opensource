# Embark OSS tool

This tool is used to validate that our open source projects adhere to our open source guidelines.

## Commands

```shell
cargo run check-maintainers
```

This command checks to see if every project listed in the Embark [opensource-website data.json][data.json] conforms to our open source guidelines to the extent that this tool can detect.

[data.json]: https://github.com/EmbarkStudios/opensource-website/blob/main/data.json

```shell
cargo run check-maintainers PROJECT_REPO_NAME
```

This command checks to see if a given Embark open source project conforms to our open source guidelines to the extent that this tool can detect.

### Flags

- `--slack-webhook-url`: An optional Slack webhook URL that is used to report problems.

## Testing

This tool has unit tests. Run them like so:

```shell
cargo test
```
