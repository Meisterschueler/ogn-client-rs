# Rust as the base image
FROM rust:1.67-slim-bullseye as builder

# Build the dependencies
RUN cargo new ogn-client
WORKDIR /ogn-client
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Get the source and build the app
COPY ./src ./src
RUN touch -a -m ./src/main.rs \
    && cargo install --path .

# Create a small final image
FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/ogn-client /usr/local/bin/ogn-client

CMD ["ogn-client"]
