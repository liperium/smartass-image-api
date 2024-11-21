# Build stage
FROM rust:buster as builder
RUN apt-get update && apt-get install -y openssl libpq-dev
WORKDIR /app
COPY . .
RUN cargo build --release

# Final stage
FROM debian:buster-slim
RUN apt-get update && apt-get install -y openssl libpq-dev curl
WORKDIR /app
COPY --from=builder /app/target/release/hdp-web-server ./
COPY --from=builder /app/Rocket.toml ./
CMD ["./hdp-web-server"]
