FROM ekidd/rust-musl-builder AS build
WORKDIR /usr/src/
USER root

# Add compilation target for later scratch container
ENV RUST_TARGETS="x86_64-unknown-linux-musl"
RUN rustup target install x86_64-unknown-linux-musl

# Creating a placeholder project
RUN USER=root cargo new proj
WORKDIR /usr/src/proj

# moving deps info
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Caching deps
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN rm -rf target/x86_64-unknown-linux-musl/release/deps/rust*

# Replacing with actual src
RUN rm src/*.rs
COPY ./src ./src

# Only code changes should need to compile
RUN cargo build --target x86_64-unknown-linux-musl --release

# This creates a TINY container with the executable! Like 4-5mb srsly
FROM scratch
COPY --from=build /usr/src/proj/target/x86_64-unknown-linux-musl/release/proj .
USER 1000
CMD ["./proj"]
