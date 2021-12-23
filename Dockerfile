FROM rust as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/k8s-server /usr/local/bin/app
EXPOSE 3000/tcp
CMD ["app"]
