VERSION 0.6

FROM rust:1.59

ARG VERSION
ARG DOCKER_IMG
ARG GHCR_IMG

WORKDIR /clear-docker-images

build-linux:
  COPY . .
	RUN cargo build --release --target x86_64-unknown-linux-gnu
  SAVE ARTIFACT target/x86_64-unknown-linux-gnu /x86_64-unknown-linux-gnu AS LOCAL target/x86_64-unknown-linux-gnu

build-images:
  FROM docker:20.10.12-dind-alpine3.15

  LABEL maintainer="Antoine <DataHearth> Langlois"
  LABEL repository="https://github.com/DataHearth/clear-docker-images"
  LABEL org.opencontainers.image.source=&quot;https://github.com/DataHearth/clear-docker-images&quot;

  COPY +build-linux/x86_64-unknown-linux-gnu /usr/local/bin/clear-docker-images

  VOLUME ["/var/run/docker.sock"]

  ENTRYPOINT ["clear-docker-images"]

  SAVE IMAGE ${DOCKER_IMG}:${VERSION}
  SAVE IMAGE ${DOCKER_IMG}:latest
  SAVE IMAGE ${GHCR_IMG}:${VERSION}
  SAVE IMAGE ${GHCR_IMG}:latest