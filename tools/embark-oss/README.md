# Embark OSS tool

This tool is used to validate that our open source projects adhere to our open source guidelines.

## Commands

```shell
cargo run check-maintainers
```

This command checks to see if every project listed in the Embark [opensource-website data.json][data.json] has an official maintainer listed in its `CODEOWNERS` file within its repository.

[data.json]: https://github.com/EmbarkStudios/opensource-website/blob/main/data.json

## Testing

This tool has unit tests. Run them like so:

```shell
cargo test
```
