# 💡 Bulb API

A minimal Rust (axum) server API backed by PostgreSQL for controlling a smart bulb from an
ESP8266/ESP32 — or any HTTP client. Supports cron **schedules** so the bulb can switch itself
on and off at set local times.

## Prerequisites

- [Rust](https://rustup.rs) (stable toolchain)
- PostgreSQL 13+ (`gen_random_uuid()` is used, which is built in from 13 onwards)

Connection settings are read from the environment (a `.env` file is loaded automatically —
see [`.env.example`](.env.example)):

| Variable            | Required | Default        | Purpose                                          |
|---------------------|----------|----------------|--------------------------------------------------|
| `PG_HOST`           | yes      | —              | Postgres host                                    |
| `PG_PORT`           | no       | `5432`         | Postgres port                                    |
| `PG_USER`           | yes      | —              | Postgres user                                    |
| `PG_PASSWORD`       | no       | —              | Postgres password                                |
| `PG_DBNAME`         | yes      | —              | Database name                                    |
| `PG_POOL_MAX_SIZE`  | no       | deadpool's     | Max pooled connections                           |
| `SCHEDULE_TZ`       | no       | `Asia/Jakarta` | IANA timezone that schedule cron fields are read in |

A missing required variable, or an `SCHEDULE_TZ` that is not a real IANA name, stops the
process at startup rather than letting it run misconfigured.

## Quick Start

```bash
# 1. Apply database migrations (creates the tables — run this first, and after every pull)
cargo run --bin migrate

# 2. Run the server (debug mode, fine for development)
cargo run
```

For production:

```bash
cargo build --release
./target/release/migrate      # migrations
./target/release/bulb-api     # server
```

The server starts on **`http://0.0.0.0:3000`**, with interactive API docs (Swagger UI) at
**`http://0.0.0.0:3000/docs`** and the raw OpenAPI document at `/api-docs/openapi.json`.

Set `RUST_LOG=info` to see scheduler activity:

```
INFO bulb_api: scheduler started with 2 enabled schedule(s); cron expressions are read in Asia/Jakarta
INFO bulb_api::api::schedule::runner: schedule fired id=… name=Turn on every morning is_on=true
```

## API Endpoints

### Bulb

| Method | Path           | Description                | Request Body            |
|--------|----------------|----------------------------|-------------------------|
| `GET`  | `/bulb`        | Get bulb state (IoT poll)  | —                       |
| `GET`  | `/v1/bulb`     | Get bulb state             | —                       |
| `PUT`  | `/v1/bulb`     | Set bulb state             | `{"is_on": true/false}` |
| `POST` | `/v1/bulb/on`  | Turn bulb on               | —                       |
| `POST` | `/v1/bulb/off` | Turn bulb off              | —                       |

### Schedules

| Method   | Path                  | Description                        | Request Body                                          |
|----------|-----------------------|------------------------------------|-------------------------------------------------------|
| `POST`   | `/v1/schedules`       | Create a schedule and start it     | `{"name","cron_expr","action"}`                       |
| `GET`    | `/v1/schedules`       | List all schedules                 | —                                                     |
| `GET`    | `/v1/schedules/{id}`  | Get one schedule                   | —                                                     |
| `PUT`    | `/v1/schedules/{id}`  | Update any subset of fields        | any of `{"name","cron_expr","action","enabled"}`      |
| `DELETE` | `/v1/schedules/{id}`  | Delete a schedule and stop its job | —                                                     |

### Response shapes

`GET /bulb` is **unversioned and frozen** — deployed devices poll this exact path and shape,
so it must never change:

```json
{
  "is_on": true,
  "updated_at": "2026-09-09 05:00:00.123456+00"
}
```

Every `/v1/` endpoint wraps its payload in a success envelope:

```json
{
  "success": true,
  "data": { "is_on": true, "updated_at": "2026-09-09T07:02:00.108410Z" }
}
```

Errors use a matching envelope, with `404` for a missing resource, `400` for bad input, and
`500` for anything on our side (the real cause is logged, never returned):

```json
{
  "success": false,
  "error": { "code": "bad_request", "message": "action must be \"on\" or \"off\"" }
}
```

### Example requests

```bash
# Check state (IoT poll — unversioned)
curl http://localhost:3000/bulb

# Get state (v1)
curl http://localhost:3000/v1/bulb

# Turn on / off
curl -X POST http://localhost:3000/v1/bulb/on
curl -X POST http://localhost:3000/v1/bulb/off

# Set via JSON body
curl -X PUT http://localhost:3000/v1/bulb \
  -H "Content-Type: application/json" \
  -d '{"is_on": true}'
```

## Schedules

A schedule turns the bulb on or off on a cron timer. Schedules are stored in Postgres and
run by an in-process cron scheduler.

### Cron format

**Six** space-separated fields — note the **leading seconds field**, which standard 5-field
crontab syntax does not have. A 5-field expression is rejected with a `400`.

```
┌───────────── second       (0-59)
│ ┌─────────── minute       (0-59)
│ │ ┌───────── hour         (0-23)
│ │ │ ┌─────── day of month (1-31)
│ │ │ │ ┌───── month        (1-12 or JAN-DEC)
│ │ │ │ │ ┌─── day of week  (0-6 or SUN-SAT)
│ │ │ │ │ │
0 53 13 * * *     →  13:53:00 every day
0 0 5 * * *       →  05:00:00 every day
0 */30 * * * *    →  every 30 minutes, on the minute
0 0 22 * * FRI    →  22:00:00 every Friday
```

### Timezone

Cron fields are read in the timezone named by **`SCHEDULE_TZ`** (default `Asia/Jakarta`), so
`0 53 13 * * *` means **13:53 local time** — not UTC. Stored timestamps (`created_at`,
`updated_at`) remain UTC.

> **Note for DST timezones:** the underlying scheduler resolves the timezone to a fixed UTC
> offset when a job is built and reuses it for later runs. `Asia/Jakarta` is a fixed
> UTC+07:00 with no DST, so this is exact. Under a DST-observing zone
> (e.g. `Europe/London`), jobs drift by an hour after a transition until the process is
> restarted.

### Example requests

```bash
# Create — turn the bulb on at 05:00 local time, daily
curl -X POST http://localhost:3000/v1/schedules \
  -H "Content-Type: application/json" \
  -d '{"name": "Turn on every morning", "cron_expr": "0 0 5 * * *", "action": "on"}'

# List
curl http://localhost:3000/v1/schedules

# Get one
curl http://localhost:3000/v1/schedules/<id>

# Change the time (takes effect immediately — the running job is replaced)
curl -X PUT http://localhost:3000/v1/schedules/<id> \
  -H "Content-Type: application/json" \
  -d '{"cron_expr": "0 30 6 * * *"}'

# Pause without deleting (stops the job; the row stays)
curl -X PUT http://localhost:3000/v1/schedules/<id> \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'

# Delete (also stops the job)
curl -X DELETE http://localhost:3000/v1/schedules/<id>
```

A created schedule looks like this:

```json
{
  "success": true,
  "data": {
    "id": "d20489fa-0629-4ea7-b51f-c3e11ac6b321",
    "name": "Turn on every morning",
    "cron_expr": "0 0 5 * * *",
    "action": "on",
    "enabled": true,
    "created_at": "2026-09-09 06:52:13.07873+00",
    "updated_at": "2026-09-09 06:52:13.07873+00"
  }
}
```

### Validation

| Input                          | Result                                          |
|--------------------------------|-------------------------------------------------|
| `action` not `"on"` / `"off"`  | `400`, nothing written                          |
| `cron_expr` unparseable        | `400`, nothing written                          |
| `id` not a UUID                | `400`                                           |
| `id` unknown                   | `404`                                           |

The cron expression is validated **before** the database write, so a typo can never leave
behind a stored schedule that could not run.

### Restarts

Cron jobs live only in memory; the `schedules` table is the source of truth. On startup the
server reloads every row with `enabled = true` and re-installs its job, so a restart or crash
resumes cleanly. Updating or deleting a schedule always replaces or removes the running job,
so a changed time never leaves the old one firing.

## ESP8266 Bulb Client

Full firmware at [`esp8266/bulb-client.ino`](esp8266/bulb-client.ino). Polls `GET /bulb` to sync relay state,
plus an optional push-button to toggle locally and `PUT` the new state back to the server.

### What you need

| Item | Purpose |
|------|---------|
| ESP8266 (NodeMCU / Wemos D1 Mini) | Microcontroller with WiFi |
| 5V active-high relay module (e.g. SRD-05VDC) | Switches the bulb |
| Bulb + bulb holder + mains wire | The load |
| Micro-USB cable | Flash & power |
| Jumper wires (F-F or M-F) | Connect everything |
| Push button (optional) | Manual toggle |
| 10kΩ resistor (optional, with button) | External pull-up (built-in pull-up used) |

### Wiring

```
    ESP8266                    Relay Module
  ┌──────────┐              ┌──────────────┐
  │           │              │              │
  │ D1 (GPIO5)│──────────────│ IN           │
  │           │              │              │
  │ VU / VIN  │──────────────│ VCC (5V)     │
  │ GND       │──────────────│ GND          │
  │           │              │              │
  │           │              │   COM ─── Live wire (mains) ──┐
  │           │              │   NO  ─── Bulb ── Neutral ────┘
  └──────────┘              └──────────────┘

  Optional button:
  ┌──────────┐
  │ D2 (GPIO4)│───┬─── button ─── GND
  │           │   │
  │           │   └── 10kΩ pull-up to 3.3V (skip — INPUT_PULLUP used)
  └──────────┘
```

**⚠️ Safety:** The relay COM/NO side carries mains voltage. Double-check connections, insulate exposed contacts, and never touch while plugged in. If you're not experienced with mains wiring, use a pre-made IoT relay board or consult an electrician.

### Flashing (Arduino IDE)

1. **Install boards** — Open Arduino IDE → Preferences → Additional Board Manager URLs, add:
   ```
   https://arduino.esp8266.com/stable/package_esp8266com_index.json
   ```
   Then **Tools → Board → Boards Manager**, search `esp8266`, install.

2. **No extra libraries needed** — the sketch only uses built-in `ESP8266WiFi` and `ESP8266HTTPClient`.

3. **Open the sketch** — `esp8266/bulb-client.ino`

4. **Edit Wi‑Fi and server IP** (lines ~28–30):
   ```cpp
   const char* WIFI_SSID  = "YourWiFi";
   const char* WIFI_PASS  = "YourPassword";
   const char* SERVER_URL = "http://192.168.1.100:3000";  // your Mac's LAN IP
   ```

5. **Select board** — **Tools → Board → ESP8266 → NodeMCU 1.0** (or Wemos D1 Mini)

6. **Plug in USB**, select the port under **Tools → Port**, hit **Upload** (→ arrow).

7. Open **Serial Monitor** (115200 baud) — you should see:
   ```
   💡 ESP8266 Bulb Client
   Connecting.....
   WiFi connected — IP: 192.168.1.42
   Relay ON / OFF
   ```

### Testing step-by-step

| Step | What to do | Expected result |
|------|-----------|-----------------|
| **1** | Start the server: `cargo run` | `INFO bulb_api: listening on http://0.0.0.0:3000` |
| **2** | Test from terminal: `curl -X POST http://localhost:3000/v1/bulb/on` | `{"success":true,"data":{"is_on":true,...}}` |
| **3** | Power up ESP8266 via USB | Serial monitor shows WiFi connected, relay state |
| **4** | Turn on via server: `curl -X POST http://localhost:3000/v1/bulb/on` | Relay clicks ON, bulb lights |
| **5** | Turn off via server: `curl -X POST http://localhost:3000/v1/bulb/off` | Relay clicks OFF, bulb off |
| **6** | Press the physical button | Relay toggles, server state syncs (`PUT /v1/bulb`) |
| **7** | Verify sync: `curl http://localhost:3000/bulb` | Returns `is_on` matching the relay |

### How the polling works

```
Every 2 seconds:
  ESP8266 ── GET /bulb ──► Server
  ESP8266 ◄── {"is_on":true} ── Server
  ESP8266 sets GPIO5 = HIGH → relay ON → bulb ON

Button press (instant):
  ESP8266 toggles relay locally
  ESP8266 ── PUT /v1/bulb {"is_on":false} ──► Server
```

The server is the source of truth. The ESP always follows whatever `GET /bulb` returns. The button is a convenience that syncs back to the server so state stays consistent.

### PlatformIO (alternative to Arduino IDE)

```ini
; platformio.ini
[env:nodemcuv2]
platform = espressif8266
board = nodemcuv2
framework = arduino
```

## Project Structure

```
src/
├── main.rs        # Startup: pool, scheduler, router, CORS, server on :3000
├── lib.rs         # Crate root
├── state.rs       # AppState — shares the pool and the scheduler with handlers
├── db.rs          # Postgres connection pool (deadpool) from env vars
├── error.rs       # AppError + the JSON error envelope
├── openapi.rs     # Root OpenAPI document (paths collected from handlers)
├── api/
│   ├── mod.rs     # Composes every resource router; 404 fallback
│   ├── response.rs# ApiResponse success envelope
│   ├── bulb/      # dto / handlers / repo — bulb state
│   └── schedule/  # dto / handlers / repo + runner.rs (the cron scheduler)
└── bin/
    ├── migrate.rs # Applies migrations/ via refinery
    └── seed.rs    # Optional dev seed data

migrations/        # Versioned SQL, embedded into the migrate binary
esp8266/
└── bulb-client.ino  # ESP8266 firmware (polls API, drives relay)
```

## Database

PostgreSQL, with schema managed by [refinery](https://github.com/rust-db/refinery) migrations
in `migrations/`. They are embedded into the `migrate` binary at compile time, so applying
them is just `cargo run --bin migrate`. **Migrations are never applied automatically by the
server** — run them yourself before starting it, and after pulling changes.

```sql
-- Bulb state: exactly one row, updated in place.
CREATE TABLE bulb_state (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    is_on       BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cron schedules.
CREATE TABLE schedules (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    cron_expr  TEXT NOT NULL,
    action     TEXT NOT NULL CHECK (action IN ('on', 'off')),
    enabled    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

To add a schema change, drop a new `migrations/V{n}__{name}.sql` in place and re-run the
migrate binary. Never edit a migration that has already been applied — refinery compares
checksums and will refuse to run.

## Deployment

Pushing to `main` runs [`.github/workflows/deploy.yml`](.github/workflows/deploy.yml):

1. **Check** — reuses [`ci.yml`](.github/workflows/ci.yml): build, clippy (warnings are
   errors), tests, and a full `docker compose` smoke test that boots Postgres, runs
   migrations, and exercises every endpoint. Nothing ships if this fails.
2. **Build** — builds the image and pushes it to GHCR as both `:latest` and `:<commit-sha>`.
3. **Deploy** — SSHes to the VPS, writes `.env` from repository secrets, then
   `docker compose -f docker-compose.prod.yml up -d --wait`.

The production stack ([`docker-compose.prod.yml`](docker-compose.prod.yml)) enforces startup
order rather than hoping for it:

```
database (healthcheck passes) → migrate (must exit 0) → bulb-api
```

so the server can never boot against a missing or half-applied schema. Postgres is on an
internal network with no published ports; only `bulb-api` joins the external `proxy` network.

### Required repository secrets

**The deploy fails without these** — Postgres refuses to initialise with an empty password.
Add them under *Settings → Secrets and variables → Actions*:

| Secret         | Purpose                                        |
|----------------|------------------------------------------------|
| `VPS_HOST`     | VPS hostname or IP                             |
| `VPS_USER`     | SSH user                                       |
| `VPS_SSH_KEY`  | Private SSH key for that user                  |
| `GH_PAT`       | Token with `read:packages`, for `docker login` on the VPS |
| `PG_USER`      | Postgres user to create and connect as         |
| `PG_PASSWORD`  | Postgres password                              |
| `PG_DBNAME`    | Database name                                  |

Optionally set a repository **variable** `SCHEDULE_TZ` to override the `Asia/Jakarta` default.

`PG_HOST` is not a secret — the workflow always sets it to `database`, the compose service
name. It is not `localhost`: inside the compose network, `localhost` is the API container
itself.

### Rolling back

Every deploy is tagged with its commit SHA, so a bad release can be pinned:

```bash
# on the VPS, in ~/app/bulb-api
sed -i 's|bulb-api:latest|bulb-api:<previous-sha>|' docker-compose.prod.yml
docker compose -f docker-compose.prod.yml up -d --wait
```

### Running the whole stack locally

```bash
# Build the image and run api + Postgres + migrations, same ordering as prod
docker compose up --build          # API on http://localhost:3000

# Or: just Postgres in Docker, API on the host (the usual dev loop)
docker compose -f docker-compose.dev.yml up -d
cargo run --bin migrate && cargo run
```

## CORS

All origins, methods, and headers are allowed — your ESP32 can call the API from any network without preflight issues.
