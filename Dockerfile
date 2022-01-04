FROM rust:alpine as builder
WORKDIR /usr/src/app
COPY . .
RUN apk add libc-dev
RUN cargo install --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/kv-store /usr/local/bin/kv-store

EXPOSE 80/tcp
ENV REDIS_HOST=localhost

CMD ["kv-store", "-vv", "-p", "80", "-s", "0.0.0.0"]
