# Key-Value Store
A Key-Value store that uses Redis to store data. Built using an async web framework in Rust with a full Command-Line interface and logging.

## Goals
The goal of this project is to serve as a samplw to deploying applications in Kubernetes; with and without Helm.

## Dockerfile
A `Dockerfile` is included that builds the executable in release mode and listens on TCP port 80 for all interfaces in the container.

## Usage
### Redis
```bash
docker run --rm -p 6379:6379 redis
```

### Server
Run in development:
```bash
cargo run
```

Install locally:
```bash
cargo install --path .
kv-store -h
```
