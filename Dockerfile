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
COPY pisi-core/Cargo.toml pisi-core/
COPY pisi-spec/Cargo.toml pisi-spec/
COPY pisi-builder/Cargo.toml pisi-builder/
COPY pisi/Cargo.toml pisi/

# Create dummy source files to cache dependencies
RUN mkdir -p pisi-core/src pisi-spec/src pisi-builder/src pisi/src
RUN echo "fn main() {}" > pisi/src/main.rs
RUN echo "" > pisi-core/src/lib.rs
RUN echo "" > pisi-spec/src/lib.rs
RUN echo "" > pisi-builder/src/lib.rs

RUN cargo build --workspace --release
RUN rm -rf pisi-core/src pisi-spec/src pisi-builder/src pisi/src

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

COPY --from=builder /app/target/release/pisi /usr/local/bin/pisi
COPY --from=builder /app/locales /app/locales

# Create necessary directories
RUN mkdir -p /var/lib/pisi/db /var/cache/pisi /var/pisi /run/lock/subsys /etc/pisi

# Default config
RUN echo 'general = { destination_directory = "/", architecture = "x86_64" }' > /etc/pisi/pisi.conf

ENTRYPOINT ["pisi"]
CMD ["--help"]