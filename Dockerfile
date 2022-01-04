FROM rust:alpine as allocator
RUN apk add git build-base cmake linux-headers
RUN cd /; git clone --depth 1 https://github.com/microsoft/mimalloc; cd mimalloc; mkdir build; cd build; cmake ..; make -j$(nproc); make install

FROM rust:alpine as builder
WORKDIR /usr/src/app
COPY . .
RUN apk add libc-dev
RUN cargo install --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/kv-store /usr/local/bin/kv-store
COPY --from=builder /mimalloc/build/*.so.* /lib

RUN ln -s /lib/libmimalloc.so.* /lib/libmimalloc.so
ENV LD_PRELOAD=/lib/libmimalloc.so
ENV MIMALLOC_LARGE_OS_PAGES=1

EXPOSE 80/tcp
ENV REDIS_HOST=localhost

CMD ["kv-store", "-vv", "-p", "80", "-s", "0.0.0.0"]
