# Build stage
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

COPY . .

RUN cargo build --release --locked -p suki-backend


# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/suki-api /app/suki-api

EXPOSE 8080

CMD ["/app/suki-api"]
