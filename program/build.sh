#!/bin/bash
# Check if the Docker image exists
set -e
docker build -t name_tokenizer .


solana program dump metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s target/deploy/mpl_token_metadata.so
solana program dump namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX target/deploy/spl_name_service.so


if [[ ${1} == "build-only" ]]; then
    echo "Only building..."
    docker run -it --net=host --mount type=bind,source=$(pwd),target=/workdir name_tokenizer:latest /bin/bash -c "cargo build-sbf"
elif [[ ${1} == "test" ]]; then
    echo "Running tests..."
    docker run -it --net=host --mount type=bind,source=$(pwd),target=/workdir name_tokenizer:latest /bin/bash -c "cargo test-sbf --features devnet"
else
    echo "Running tests + building..."
    docker run -it --net=host --mount type=bind,source=$(pwd),target=/workdir name_tokenizer:latest /bin/bash -c "cargo test-sbf --features devnet && cargo build-sbf"
fi