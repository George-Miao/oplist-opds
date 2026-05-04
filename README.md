# oplist-opds

An [OPDS 1.2](https://specs.opds.io/opds-1.2) catalog server that exposes an
[OpenList](https://github.com/OpenListTeam/OpenList) file server to e-book
reader apps such as **KOReader**, **Panels**, **Yomu**, and any other
OPDS-capable client.

```mermaid
flowchart LR
    C["OPDS Clients\nKOReader · Panels · Yomu"]
    O([oplist-opds])
    L["OpenList"]

    C -- "OPDS 1.2 Atom/XML" --> O
    O -- "REST / JSON" --> L
    O -. "streamed download" .-> C
```

## Features

- **Browse** the OpenList directory tree as an OPDS navigation / acquisition feed
- **Search** files by name via the OpenSearch endpoint
- **Stream downloads** through an optional reverse proxy that injects the
  OpenList bearer token — the reader app never needs to know the token
- **Flexible config** — optional config file (TOML/YAML/JSON) with env var
  overrides; env always wins
- **Minimal Docker image** — Nix closure bundles only the binary and its
  runtime dependencies; no base OS image required

## Endpoints

| URL                        | Description                          |
| -------------------------- | ------------------------------------ |
| `GET /opds`                | OPDS catalog root (navigation feed)  |
| `GET /opds/browse/`        | Root directory listing               |
| `GET /opds/browse/{*path}` | Subdirectory listing                 |
| `GET /opds/search?q=…`     | Full-text search results             |
| `GET /opds/opensearch.xml` | OpenSearch description document      |
| `GET /opds/raw/{*path}`    | Proxied file download (injects auth) |

Point your OPDS client at `http://<host>:3000/opds`.

## Configuration

Settings are loaded in priority order — later sources override earlier ones:

1. Config file passed as the first argument _(optional)_
2. Environment variables _(highest priority)_

The config file format is inferred from the file extension: `.toml`, `.yaml` / `.yml`, or `.json`.

| Key / env var                     | Default                 | Description                                     |
| --------------------------------- | ----------------------- | ----------------------------------------------- |
| `oplist_url` / `OPLIST_URL`       | **required**            | Base URL of the OpenList instance               |
| `oplist_token` / `OPLIST_TOKEN`   | `""`                    | Bearer token; leave empty for public instances  |
| `bind_addr` / `BIND_ADDR`         | `0.0.0.0:3000`          | Address the server listens on                   |
| `base_url` / `BASE_URL`           | `http://localhost:3000` | Public URL of this server (used in feed links)  |
| `catalog_title` / `CATALOG_TITLE` | `OpenList OPDS`         | Title shown in the catalog root                 |
| `root_path` / `ROOT_PATH`         | `/`                     | OpenList path to expose as the OPDS root        |
| `proxy_enabled` / `PROXY_ENABLED` | `false`                 | Route downloads through the proxy (hides token) |
| `RUST_LOG`                        | `info`                  | Log filter (env only)                           |

### `config.toml` example

```toml
oplist_url    = "https://files.example.com"
oplist_token  = "my-secret-token"
bind_addr     = "0.0.0.0:3000"
base_url      = "https://opds.example.com"
catalog_title = "My Books"
root_path     = "/books"
proxy_enabled = true
```

## Running with Docker

The recommended way to run oplist-opds is via the pre-built image published
to the GitHub Container Registry on every push to `main`.

```sh
docker run -d \
  --name oplist-opds \
  -p 3000:3000 \
  -e OPLIST_URL=https://files.example.com \
  -e OPLIST_TOKEN=my-secret-token \
  -e BASE_URL=https://opds.example.com \
  -e CATALOG_TITLE="My Books" \
  ghcr.io/george-miao/oplist-opds:latest
```

Or with a config file:

```sh
docker run -d \
  --name oplist-opds \
  -p 3000:3000 \
  -v ./config.toml:/config.toml:ro \
  ghcr.io/george-miao/oplist-opds:latest \
  /config.toml
```

### Docker Compose

```yaml
services:
  oplist-opds:
    image: ghcr.io/george-miao/oplist-opds:latest
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      OPLIST_URL: https://files.example.com
      OPLIST_TOKEN: my-secret-token
      BASE_URL: https://opds.example.com
      CATALOG_TITLE: My Books
      ROOT_PATH: /books
      PROXY_ENABLED: "true"
```

## Building

### With Nix (recommended)

Produces a scratch Docker image (~10 MB) as a `.tar.gz` archive:

```sh
nix build .#oplist-opds-docker
```

Build just the binary:

```sh
nix build .#oplist-opds
```

Load the image into Docker:

```sh
docker load < result
```

The Nix build uses [crane](https://github.com/ipetkov/crane) and
[fenix](https://github.com/nix-community/fenix) for a reproducible,
dependency-cached build. `dockerTools.buildImage` traces the Nix closure of
the binary and bundles exactly those store paths — no base image needed.

### With Cargo

Requires Rust (edition 2024) and a C linker. For a native debug build:

```sh
# env-only (no config file)
OPLIST_URL=https://files.example.com cargo run

# with a config file as the first argument
cargo run -- config.toml
cargo run -- /etc/oplist-opds/config.yaml
```

For a release build:

```sh
cargo build --release
./target/release/oplist-opds config.toml
```

### Development shell

```sh
nix develop   # drops you into a shell with cargo
```

## Tech Stack

| Layer          | Crate                                                                                           |
| -------------- | ----------------------------------------------------------------------------------------------- |
| Async runtime  | [`compio`](https://github.com/compio-rs/compio) — io_uring on Linux                             |
| HTTP client    | [`cyper`](https://github.com/compio-rs/cyper) — compio-native HTTP                              |
| HTTP server    | [`axum`](https://github.com/tokio-rs/axum) + [`cyper-axum`](https://github.com/compio-rs/cyper) |
| XML output     | [`quick-xml`](https://github.com/tafia/quick-xml) — OPDS Atom feeds                             |
| Config         | [`figment`](https://github.com/SergioBenitez/Figment) — layered config                          |
| Error handling | [`snafu`](https://github.com/shepmaster/snafu) — typed contextual errors                        |

## License

MIT
