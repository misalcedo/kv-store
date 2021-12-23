# Key-Value Store
A Key-Value store that uses Redis to store data. Built using an async web framework in Rust with a full Command-Line interface and logging.

## Goals
The goal of this project is to serve as a samplw to deploying applications in Kubernetes; with and without Helm.

## Dockerfile
A `Dockerfile` is included that builds the executable in release mode and listens on TCP port 80 for all interfaces in the container.

## Usage
### With Docker
```bash
docker run --rm -p 6379:6379 redis
docker build . -t kv-store
docker run --rm -p 80:80 kv-store
```

### With Docker Compose
```bash
docker-compose up
```

### Locally
Run development server:
```bash
cargo run -- -h
```

Run release server:
```bash
cargo install --path .
kv-store -h
```
