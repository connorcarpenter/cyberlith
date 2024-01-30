# ONE DAY, WE WILL AUTOMATE THESE BUILDS AGAIN!

## Build
#FROM rust:1.58.1 as build
#
#ENV PKG_CONFIG_ALLOW_CROSS=1
#
#WORKDIR /usr/src
#COPY Cargo.toml .
#COPY Cargo.lock .
#
#WORKDIR /usr/src/crates
#COPY crates/. .
#
#WORKDIR /usr/src/apps
#COPY apps/. .
#
#RUN cargo install --path /usr/src/apps/content

# Runtime
FROM gcr.io/distroless/cc-debian12

ARG server_name
ENV server_name1 $server_name

COPY $server_name /usr/local/bin/server

CMD ["server"]