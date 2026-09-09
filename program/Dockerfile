FROM rust:1.95.0

ENV HOME="/nonroot"
ENV PATH="${HOME}/.local/share/solana/install/active_release/bin:${PATH}"
ARG HOST_UID
ARG HOST_GID

RUN mkdir nonroot
RUN mkdir workdir
RUN chown $HOST_UID:$HOST_GID ./nonroot
RUN chown $HOST_UID:$HOST_GID ./workdir

USER $HOST_UID:$HOST_GID


# Install Solana tools.
RUN sh -c "$(curl -sSfL https://release.anza.xyz/v4.2.2/install)"

WORKDIR /nonroot

RUN cargo new temp --edition 2021 --lib
WORKDIR /nonroot/temp
RUN printf "\n[lib]\ncrate-type=[\"cdylib\"]\n" >> Cargo.toml
RUN cargo build-sbf

WORKDIR /workdir