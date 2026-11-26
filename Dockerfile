# Stage 1: Build the Rust binary 
FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app 
COPY . . 
RUN cargo build --release --locked -p suki-backend

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim
RUN apt-get update &amp;&amp; apt-get install -y ca-certificates libssl-dev &amp;&amp; rm -rf /var/lib/apt/lists/\*
WORKDIR /app
COPY --from=builder /app/target/release/suki-api /app/suki-api

EXPOSE 8080
CMD ["/app/suki-api"]
