FROM rust:bullseye as builder
WORKDIR /usr/src/ocultum
COPY . .
RUN apt-get update && \
    apt-get install -y libi2c-dev && \
    cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libi2c-dev
COPY --from=builder /usr/src/ocultum/target/release/ocultum /usr/local/bin/
CMD ["ocultum"]
