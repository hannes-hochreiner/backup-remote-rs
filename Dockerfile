FROM rust:slim AS builder
RUN apt update && apt install librust-openssl-dev -y
RUN mkdir -p /opt/backup-remote-rs
COPY src /opt/backup-remote-rs/src
COPY Cargo.* /opt/backup-remote-rs/
RUN cd /opt/backup-remote-rs && cargo build --release --locked

FROM debian:stable-slim AS updater
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
RUN apt update && apt install openssl ca-certificates -y
COPY --from=builder /opt/backup-remote-rs/target/release/backup-remote-updater /opt/backup-remote-updater
CMD ["/opt/backup-remote-updater"]

FROM debian:stable-slim AS worker
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
RUN apt update && apt install openssl ca-certificates -y
COPY --from=builder /opt/backup-remote-rs/target/release/backup-remote-worker /opt/backup-remote-worker
CMD ["/opt/backup-remote-worker"]
