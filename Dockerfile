# Build stage
FROM rust:1.82-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libudev-dev \
    libdbus-1-dev \
    protobuf-compiler \
    clang \
    llvm \
    liblzma-dev \
    libzstd-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY luppo-core/Cargo.toml luppo-core/
COPY luppo-spec/Cargo.toml luppo-spec/
COPY luppo-builder/Cargo.toml luppo-builder/
COPY luppo/Cargo.toml luppo/

# Create dummy source files to cache dependencies
RUN mkdir -p luppo-core/src luppo-spec/src luppo-builder/src luppo/src
RUN echo "fn main() {}" > luppo/src/main.rs
RUN echo "" > luppo-core/src/lib.rs
RUN echo "" > luppo-spec/src/lib.rs
RUN echo "" > luppo-builder/src/lib.rs

RUN cargo build --workspace --release
RUN rm -rf luppo-core/src luppo-spec/src luppo-builder/src luppo/src

# Copy actual source
COPY . .

RUN cargo build --workspace --release

# Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libssl3 \
    libsqlite3-0 \
    libudev1 \
    libdbus-1-3 \
    zlib1g \
    xz-utils \
    zstd \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/luppo /usr/local/bin/luppo
COPY --from=builder /app/locales /app/locales

# Create necessary directories
RUN mkdir -p /var/lib/luppo/db /var/cache/luppo /var/luppo /run/lock/subsys /etc/luppo

# Default config
RUN echo 'general = { destination_directory = "/", architecture = "x86_64" }' > /etc/luppo/luppo.conf

ENTRYPOINT ["luppo"]
CMD ["--help"]