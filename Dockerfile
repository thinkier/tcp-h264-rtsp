FROM rust:bullseye as build
RUN rustup default nightly

WORKDIR /app

COPY Cargo.* ./
RUN mkdir src; echo "fn main() { panic!(\"Cached executable is being used\") }" > _cache_main.rs
RUN sed -i 's#src/main.rs#_cache_main.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#_cache_main.rs#src/main.rs#' Cargo.toml; rm -f _cache_main.rs

COPY . .
RUN cargo build --release

FROM debian:bullseye
RUN apt-get update; apt-get install -y vlc

WORKDIR /app

COPY --from=build /app/tcp-h264-rtsp .

CMD /app/tcp-h264-rtsp
