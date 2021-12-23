FROM rust as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/kv-store /usr/local/bin/kv-store

EXPOSE 80/tcp
ARG redis_host=localhost

CMD ["/bin/sh", "-c", "kv-store -vv -p 80 -s 0.0.0.0 -r $redis_host"]