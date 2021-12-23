FROM rust as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/kv-store /usr/local/bin/kv-store

EXPOSE 80/tcp
ENV REDIS_HOST=localhost

CMD ["/bin/sh", "-c", "kv-store -vv -p 80 -s 0.0.0.0 -r $REDIS_HOST"]