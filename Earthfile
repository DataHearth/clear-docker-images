VERSION 0.6

FROM rust:1.59

ARG VERSION
ARG DOCKER_IMG
ARG GHCR_IMG

WORKDIR /clear-docker-images

build-linux:
  COPY . .
  RUN rustup target add x86_64-unknown-linux-gnu
	RUN cargo build --release --target x86_64-unknown-linux-gnu
  SAVE ARTIFACT target/x86_64-unknown-linux-gnu /x86_64-unknown-linux-gnu AS LOCAL target/x86_64-unknown-linux-gnu

build-musl:
  COPY . .
  RUN rustup target add x86_64-unknown-linux-musl
	RUN cargo build --release --target x86_64-unknown-linux-musl
  SAVE ARTIFACT target/x86_64-unknown-linux-musl /x86_64-unknown-linux-musl AS LOCAL target/x86_64-unknown-linux-musl

build-images:
  FROM docker:20.10.12-dind-alpine3.15

  LABEL maintainer="Antoine <DataHearth> Langlois"
  LABEL repository="https://github.com/DataHearth/clear-docker-images"
  LABEL org.opencontainers.image.source=&quot;https://github.com/DataHearth/clear-docker-images&quot;

  COPY +build-musl/x86_64-unknown-linux-musl/release/clear-docker-images /usr/local/bin/clear-docker-images

  VOLUME ["/var/run/docker.sock"]

  ENTRYPOINT ["clear-docker-images"]

  SAVE IMAGE --push ${DOCKER_IMG}:${VERSION}
  SAVE IMAGE --push ${DOCKER_IMG}:latest
  SAVE IMAGE --push ${GHCR_IMG}:${VERSION}
  SAVE IMAGE --push ${GHCR_IMG}:latest