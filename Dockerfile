# ── Build stage ──────────────────────────────────────────────
FROM rust:1-bookworm AS builder

WORKDIR /app

# Everything needed to compile, copied before the build.
#
# `migrations/` is NOT optional here: src/bin/migrate.rs calls
# `refinery::embed_migrations!("./migrations")`, which reads that directory at
# COMPILE time and embeds the SQL into the binary. Omit it and the build dies
# with "cannot find `migrations` in `embedded`".
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

# One honest build of every binary.
#
# The usual "build stub sources first to cache dependencies" trick is
# deliberately NOT used: `COPY` preserves the build context's mtimes, which end
# up older than the stub build's outputs, so Cargo sees the real sources as
# already fresh and ships the stubs. That failure is silent — the binaries run
# and exit 0 doing nothing. Docker layer caching (cache-from/to: type=gha in
# the workflow) already skips this layer when nothing changed; if compile times
# ever become a problem, reach for cargo-chef rather than hand-rolled stubs.
#
# `--locked` fails if Cargo.lock would need changing, so the image contains
# exactly the dependency versions the repo committed.
RUN cargo build --release --locked --bins

# ── Runtime stage ────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Both binaries: `migrate` applies the schema, `bulb-api` serves traffic.
# docker-compose.prod.yml runs `migrate` as a one-shot service that must exit 0
# before the server is allowed to start.
COPY --from=builder /app/target/release/bulb-api /app/bulb-api
COPY --from=builder /app/target/release/migrate  /app/migrate

# Drop privileges — nothing here needs root.
RUN useradd --no-create-home --uid 10001 app && chown -R app:app /app
USER app

EXPOSE 3000

CMD ["./bulb-api"]
