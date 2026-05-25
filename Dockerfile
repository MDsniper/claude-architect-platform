# Build stage
FROM rust:latest as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY src/ ./src/
RUN touch src/main.rs && cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/claude-architect-platform /app/server
COPY guide_en.MD /app/guide_en.MD
COPY src/index.html /app/index.html

RUN mkdir -p /app/data

EXPOSE 8080

CMD ["./server"]