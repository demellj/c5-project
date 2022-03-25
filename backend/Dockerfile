FROM rust:1.59

WORKDIR /usr/src/builddir
COPY . .
RUN rm -rf ./target/ || true
RUN cargo build --release
