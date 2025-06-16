#!/bin/bash
# docker_build_run.sh
# Purpose: Build and run the ocultum application in a Docker container for Raspberry Pi Zero 2W.
# Usage: ./docker_build_run.sh [local]
# - If 'local' is provided as an argument, the script will run the container locally
#   with a simulated I2C environment for testing purposes.
# - Without arguments, it's designed to run on Raspberry Pi Zero 2W.

set -e

CONTAINER_NAME="ocultum-app"
IMAGE_NAME="ocultum:latest"

# Check if we're running locally or on the Raspberry Pi
if [[ "$1" == "local" ]]; then
  echo "=== Building Docker image for local testing ==="
  DOCKER_BUILDKIT=1 docker build -t $IMAGE_NAME -f - . <<EOF
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
EOF

  echo "=== Running Docker container locally (SIMULATED I2C) ==="
  docker run --rm -it --name $CONTAINER_NAME $IMAGE_NAME
else
  # Running on Raspberry Pi with actual I2C access
  echo "=== Building Docker image for Raspberry Pi Zero 2W ==="
  DOCKER_BUILDKIT=1 docker build -t $IMAGE_NAME -f - . <<EOF
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
EOF

  echo "=== Running Docker container with I2C access ==="
  docker run --rm -it --device /dev/i2c-1 --privileged --name $CONTAINER_NAME $IMAGE_NAME
fi
