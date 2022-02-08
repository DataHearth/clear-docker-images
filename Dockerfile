FROM rust:1.58 as builder

WORKDIR /app
COPY . .

RUN cargo install --path .

FROM debian:buster-slim

LABEL maintainer="Antoine <DataHearth> Langlois"
LABEL repository="https://github.com/DataHearth/clear-docker-images"
LABEL org.opencontainers.image.source=&quot;https://github.com/DataHearth/clear-docker-images&quot;

RUN apt-get update && \
  # apt-get -qy full-upgrade && \
  apt-get install -qy curl && \
  curl -sSL https://get.docker.com/ | sh

COPY --from=builder /usr/local/cargo/bin/clear-docker-images /usr/local/bin/clear-docker-images

VOLUME ["/var/run/docker.sock"]

ENTRYPOINT ["clear-docker-images"]