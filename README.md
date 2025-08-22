# Fullstack Workspace

This is a monorepo containing a fullstack application with
Rust (backend, WASM) and SolidJS (frontend) components.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (for frontend)
- [wasm-bindgen-cli](https://rustwasm.github.io/wasm-bindgen/reference/cli.html) `cargo install wasm-bindgen-cli`
- [sqlx-cli](https://crates.io/crates/sqlx-cli) `cargo install sqlx-cli`
- [mailtutan](https://github.com/mailtutan/mailtutan) `cargo install mailtutan`

### Email Testing (Mailtutan)

To test email functionality locally,
you can use [mailtutan](https://github.com/mailtutan/mailtutan),
a simple SMTP server for development

```sh
mailtutan --ip 127.0.0.1
```

This will start a local SMTP server on port 1025 by default.
Configure your `.env` or server launch to use `127.0.0.1:1025` as the SMTP relay.

## Frontend (SolidJS) setup

```sh
cd ui
npm install
```

### Start the development server

```sh
npm run dev
```

### Build for production

```sh
npm run build
```

The output will be in the `ui/dist` folder.

## Backend (Rust) Setup

### Database

**Important:**
There is a chicken-and-egg problem: the SQLite database is expected in the `target` directory, but that directory does not exist until the codebase is built. However, building the codebase (with sqlx in online mode) requires the database to be present and `DATABASE_URL` to be set.

**Workaround:**
Manually create the `target` directory before running sqlx commands:

```sh
mkdir target
```

Set the `DATABASE_URL` environment variable

#### Linux / macOS (bash / zsh)

```sh
export DATABASE_URL="sqlite://target/data.db"
```

#### Windows (Powershell)

```sh
$env:DATABASE_URL="sqlite://target/data.db"
```

#### Windows (Command Prompt)

```sh
set DATABASE_URL=sqlite://target/data.db
```

Then setup the database as described.
This creates the database specified in your DATABASE_URL and runs any pending migrations.

```sh
sqlx database setup --source .\migrations\
```

### WASM (Rust → JS)

```sh
cargo build -p wasm --target wasm32-unknown-unknown --profile web
wasm-bindgen ./target/wasm32-unknown-unknown/web/wasm.wasm --out-dir ./ui/lib/wasm --target web
```

### Run the development server

```sh
cargo run --bin server
```

### Make Release Build

```sh
cargo build --bin server --release
```

The release binary will be in `target/release/server.exe` (Windows) or `target/release/server` (Linux/macOS).

## Feature Flags

This workspace uses Cargo feature flags to enable optional functionality in various crates. You can enable features at build or run time using `--features`.

### Server Crate Features

The following features are available for the `server` binary crate:

- **client-ip**: Enables listing the client-ip of the incoming request in application logs.
- **openapi**: Enables openapi documentation support.
- **profiles**: Enables use of profiles like `dev`, `staging`, `prod`, etc... by setting the `RUST_PROFILE` environment variable. Requires having `.env.<profile>` files like `.env`(default profile), `.env.dev`, `.env.staging`, `.env.prod`, etc... in the current working directory.
- **rate-limit**: Enables rate limiting middleware.
- **serve-dir**: Enables serving the frontend UI from the backend.
- **smtp**: Enables SMTP email sending support (adds `lettre`, `tera`, and `token` dependencies, and enables related features in `lettre`).
- **smtp--no-tls**: Enables SMTP support without TLS (used for testing purposes only).
- **tracing**: Enables application logs. Log level can be set using the `RUST_LOG` environment variable.

You can enable these features at build or run time using the `--features` flag. For example:

```sh
cargo run --bin server --features smtp,rate-limit,tracing
```

Or for a release build:

```sh
cargo build --bin server --release --features smtp,rate-limit,tracing
```

## Deployment

- Backend: Deploy the Rust server as you would any Axum-based service.
- Frontend: Deploy the contents of `ui/dist` as static files.
- WASM: Ensure the generated WASM files are available in the frontend's `lib/wasm` directory.

## Useful Links

- [SolidJS Documentation](https://solidjs.com)
- [Vite Documentation](https://vite.dev/guide/static-deploy.html)
- [Rust WASM Book](https://rustwasm.github.io/book/)
