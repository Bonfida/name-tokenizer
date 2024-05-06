FROM rust:1.75.0

ENV HOME="/root"
ENV PATH="${HOME}/.local/share/solana/install/active_release/bin:${PATH}"

# 
RUN  apt-get update -qq && apt-get upgrade -qq && apt-get install -qq \
    build-essential libssl-dev libudev-dev


# Install Solana tools.
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.18.11/install)"

RUN cargo install bonfida-cli --version 0.4.4

RUN bonfida autoproject dummy

WORKDIR /dummy/program

RUN cargo build-sbf

WORKDIR /

RUN rm -r dummy

WORKDIR /workdir