FROM ekidd/rust-musl-builder:beta AS build
WORKDIR /usr/src/
USER root

# Install zigbuild
RUN cargo install cargo-zigbuild
# Add compilation target for later scratch container
ENV RUST_TARGETS="x86_64-unknown-linux-musl"
RUN rustup target install x86_64-unknown-linux-musl

# Placeholder projects
RUN USER=root cargo new crossout-log-common
RUN USER=root cargo new crossout-log-server
# Perapare common deps
COPY crossout-log-common/Cargo.lock crossout-log-common/Cargo.lock
COPY crossout-log-common/Cargo.toml crossout-log-common/Cargo.toml
# Perapare server deps
COPY crossout-log-server/Cargo.lock crossout-log-server/Cargo.lock
COPY crossout-log-server/Cargo.toml crossout-log-server/Cargo.toml
# Cache deps
WORKDIR /usr/src/crossout-log-server
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN rm -rf target/x86_64-unknown-linux-musl/release/deps/rust*

# Replace placeholder projects with actual sources
WORKDIR /usr/src/
RUN rm ./crossout-log-common/src/*.rs
COPY ./crossout-log-common/src ./crossout-log-common/src
RUN rm ./crossout-log-server/src/*.rs
COPY ./crossout-log-server/src ./crossout-log-server/src
COPY ./crossout-log-server/migrations ./crossout-log-server/migrations

# Only code changes should need to compile
WORKDIR /usr/src/crossout-log-server
RUN cargo zigbuild --target x86_64-unknown-linux-musl --release
# RUN cargo build --target x86_64-unknown-linux-musl --release

# Create container
FROM scratch
COPY --from=build /usr/src/crossout-log-server/target/x86_64-unknown-linux-musl/release/crossout-log-server .
USER 1000
CMD ["./crossout-log-server"]
