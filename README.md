## Deputy

Deputy is a Digital Library for cybersecurity exercise artifacts. Deputy functionality is
divided into 3 categories:

- `deputy-package-server` Executable that acts as a repository for the artifacts

- `deputy` CLI program that acts as a client for the repository.

- `deputy-library` Rust library containing shared code and structures between server and the client
program


## Development

### Deputy-CLI

Use attached `.devcontainer` in `vscode` for better development experience.

Executable at `target/debug/deputy` is automatically added to the path and working configuration
is specified at `/home/vscode/.deputy/configuration.toml`.

For now testing out changes in `deputy` envolves two steps

1. `cargo build -p deputy`

2. Test the `deputy` command in CLI
