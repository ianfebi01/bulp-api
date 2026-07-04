# ── Build stage ──────────────────────────────────────────────
FROM rust:1-bookworm AS builder

WORKDIR /app

# Cache dependencies: build deps with a stub, then touch after
# copying real source so cargo detects the change.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Runtime stage ────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/bulb-api /app/bulb-api

EXPOSE 3000

# Data dir for the SQLite database — mount a volume here
RUN mkdir /app/data
ENV BULB_DB_PATH=/app/data/bulb.db

CMD ["./bulb-api"]
