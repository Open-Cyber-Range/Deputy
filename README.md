## Deputy

Deputy is a Digital Library for cybersecurity exercise artifacts. Deputy functionality is
divided into 3 categories:

- `deputy-package-server` Executable that acts as a repository for the artifacts

- `deputy` CLI program that acts as a client for the repository.

- `deputy-library` Rust library containing shared code and structures between server and the client program

## Development

### Deputy-CLI

Use attached `.devcontainer` in `vscode` for better development experience.

Executable at `target/debug/deputy` is automatically added to the path and working configuration
is specified at `/home/vscode/.deputy/configuration.toml`.

For now testing out changes in `deputy` involves two steps

1. `cargo build -p deputy`

2. Test the `deputy` command in CLI

### Deputy Front End

Running on Next.js, located in the `web-client` directory.

To use the hot reloading feature:

1. `yarn` Build initial packages

2. `yarn dev` Run the local server

To run the production build (no hot reloading):

1. `yarn` Build initial packages

2. `yarn build` Build production artifacts

3. `yarn start` Run the local server. If there are conflicts with the default port `3000` then assign your own `PORT` environment variable before the `next start` command in `package.json`

Additional configuration:

- Modify the `.env` file to your liking, for example, set the `DOCUMENTATION_URL` to your own documentation page

### Deputy Package Server

#### Database

Deputy package server uses a MySQL database for saving metadata of packages. Default credentials are `mysql_user:mysql_pass`.

##### Testing

For local testing, change the URL of the database in `deputy-package-server/example-config.yml` from `mariadb` to `127.0.0.1`, remove previous containers, if necessary.

Get deputy-package-server running with 
`cargo run -p deputy-package-server --  deputy-package-server/example-config.yml`

For front end, run
`yarn dev`

If there are no test packages showing, you need to first upload them with `deputy publish` inside the test package folder.
