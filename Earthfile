VERSION 0.6

FROM rust:1.59

ARG VERSION
ARG DOCKER_IMG
ARG GITEA_IMG

WORKDIR /clear-docker-images

build-linux:
  COPY . .

  RUN rustup target add x86_64-unknown-linux-musl
	
  RUN cargo build --release --all-features --target x86_64-unknown-linux-gnu
	RUN cargo build --release --all-features --target x86_64-unknown-linux-musl
  
  SAVE ARTIFACT target/x86_64-unknown-linux-gnu /x86_64-unknown-linux-gnu AS LOCAL target/x86_64-unknown-linux-gnu
  SAVE ARTIFACT target/x86_64-unknown-linux-musl /x86_64-unknown-linux-musl AS LOCAL target/x86_64-unknown-linux-musl

build-images:
  FROM gcr.io/distroless/static-debian11

  LABEL maintainer="Antoine Langlois <antoine.l@antoine-langlois>"
  LABEL repository="https://gitea.antoine-langlois.net/DataHearth/clear-docker-images"
  LABEL org.opencontainers.image.source=&quot;https://gitea.antoine-langlois.net/DataHearth/clear-docker-images&quot;

  COPY +build-linux/x86_64-unknown-linux-musl/release/clear-docker-images /usr/local/bin/clear-docker-images

  VOLUME ["/var/run/docker.sock"]

  ENTRYPOINT ["clear-docker-images"]

  SAVE IMAGE --push ${DOCKER_IMG}:${VERSION}
  SAVE IMAGE --push ${DOCKER_IMG}:latest
  SAVE IMAGE --push ${GITEA_IMG}:${VERSION}
  SAVE IMAGE --push ${GITEA_IMG}:latest
