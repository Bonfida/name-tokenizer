FROM rust:1.87.0

ENV HOME="/root"
ENV PATH="${HOME}/.local/share/solana/install/active_release/bin:${PATH}"

# 
RUN  apt-get update -qq && apt-get upgrade -qq && apt-get install -qq \
    build-essential libssl-dev libudev-dev


# Install Solana tools.
RUN sh -c "$(curl -sSfL https://release.anza.xyz/v2.1.21/install)"

RUN cargo install bonfida-cli --version 0.6.9 --locked

RUN bonfida autoproject dummy

WORKDIR /dummy/program

RUN cargo build-sbf

WORKDIR /

RUN rm -r dummy

WORKDIR /workdir