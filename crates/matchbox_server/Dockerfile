# Signaling server as a docker image
#
# to build, run `docker build -f matchbox_server/Dockerfile` from root of the
# repository

FROM rust:1.86-slim-bullseye AS builder

WORKDIR /usr/src/matchbox_server/

COPY README.md /usr/src/README.md
COPY ./Cargo.toml /usr/src/matchbox_server/Cargo.toml
COPY ./src /usr/src/matchbox_server/src

RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/matchbox_server/target/release/matchbox_server /usr/local/bin/matchbox_server
CMD ["matchbox_server"]
