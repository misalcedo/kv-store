FROM rust as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/k8s-server /usr/local/bin/app
EXPOSE 80/tcp
CMD ["app", "-p", "80", "-s", "0.0.0.0"]
