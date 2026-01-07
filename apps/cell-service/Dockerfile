FROM rust:1.85-slim as builder

WORKDIR /app

# RUN apt-get update && apt-get install -y \
#     pkg-config \
#     libssl-dev \
#     build-essential \
#     && rm -rf /var/lib/apt/lists/*

# cache cheat
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# real src
COPY src ./src
RUN touch src/main.rs && \
    cargo build --release

# runner
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# blessed be binaries
COPY --from=builder /app/target/release/cell-service /app/cell-service

ENV RUST_LOG=info
ENV PORT=3000
ENV CELL_ID=unknown

EXPOSE 3000

CMD ["/app/cell-service"]
