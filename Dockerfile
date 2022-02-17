FROM rust:1.58 as builder

WORKDIR /app
COPY . .

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM docker:20.10.12-dind-alpine3.15

LABEL maintainer="Antoine <DataHearth> Langlois"
LABEL repository="https://github.com/DataHearth/clear-docker-images"
LABEL org.opencontainers.image.source=&quot;https://github.com/DataHearth/clear-docker-images&quot;

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/clear-docker-images /usr/local/bin/clear-docker-images

VOLUME ["/var/run/docker.sock"]

ENTRYPOINT ["clear-docker-images"]