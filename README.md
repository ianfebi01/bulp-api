# 💡 Bulb API

A minimal Rust server API with SQLite for controlling a smart bulb from an ESP32 (or any HTTP client).

## Prerequisites

- [Rust](https://rustup.rs) (stable toolchain)

No system SQLite required — the `rusqlite` crate bundles its own copy.

## Quick Start

```bash
# Build
cargo build

# Run (debug mode, fine for development)
cargo run
```

For production:

```bash
# Build optimized binary
cargo build --release

# Run it
./target/release/bulb-api
```

The server starts on **`http://0.0.0.0:3000`**. The SQLite database file (`bulb.db`) is created automatically in the working directory on first run.

## API Endpoints

| Method | Path         | Description              | Request Body            |
|--------|-------------|--------------------------|-------------------------|
| `GET`  | `/bulb`     | Get current bulb state   | —                       |
| `POST` | `/bulb/on`  | Turn bulb on             | —                       |
| `POST` | `/bulb/off` | Turn bulb off            | —                       |
| `PUT`  | `/bulb`     | Set bulb state           | `{"is_on": true/false}` |

All responses share the same shape:

```json
{
  "is_on": true,
  "updated_at": "2026-06-27 04:15:00"
}
```

### Example requests

```bash
# Check state
curl http://localhost:3000/bulb

# Turn on
curl -X POST http://localhost:3000/bulb/on

# Turn off
curl -X POST http://localhost:3000/bulb/off

# Set via JSON body
curl -X PUT http://localhost:3000/bulb \
  -H "Content-Type: application/json" \
  -d '{"is_on": true}'
```

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
| **1** | Start the server: `cargo run` | `💡 Bulb API listening on http://0.0.0.0:3000` |
| **2** | Test from terminal: `curl -X POST http://localhost:3000/bulb/on` | `{"is_on":true,"updated_at":"..."}` |
| **3** | Power up ESP8266 via USB | Serial monitor shows WiFi connected, relay state |
| **4** | Turn on via server: `curl -X POST http://localhost:3000/bulb/on` | Relay clicks ON, bulb lights |
| **5** | Turn off via server: `curl -X POST http://localhost:3000/bulb/off` | Relay clicks OFF, bulb off |
| **6** | Press the physical button | Relay toggles, server state syncs (`PUT /bulb`) |
| **7** | Verify sync: `curl http://localhost:3000/bulb` | Returns `is_on` matching the relay |

### How the polling works

```
Every 2 seconds:
  ESP8266 ── GET /bulb ──► Server
  ESP8266 ◄── {"is_on":true} ── Server
  ESP8266 sets GPIO5 = HIGH → relay ON → bulb ON

Button press (instant):
  ESP8266 toggles relay locally
  ESP8266 ── PUT /bulb {"is_on":false} ──► Server
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
├── main.rs      # Router, CORS, server startup (port 3000)
├── db.rs        # SQLite schema, get_state, set_state
├── handlers.rs  # Axum route handlers
└── models.rs    # Request/response serde structs

esp8266/
└── bulb-client.ino  # ESP8266 firmware (polls API, drives relay)
```

## Database

Single SQLite table, auto-created on startup:

```sql
CREATE TABLE bulb_state (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    is_on      INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
```

Only one row ever exists (`id = 1`), updated in place.

## CORS

All origins, methods, and headers are allowed — your ESP32 can call the API from any network without preflight issues.
