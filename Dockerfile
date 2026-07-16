# Build stage
FROM rust:1.96-slim-bookworm AS build

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/epeletii-backend /app/server
COPY dictionary.db /app/dictionary.db

WORKDIR /app
EXPOSE 9001

ENV RUST_LOG=info
ENV MONGO_URI=mongodb://<your-mongo-host>:27017/epeletii

CMD ["/app/server"]
