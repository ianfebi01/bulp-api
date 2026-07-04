/*
 * ESP8266 Bulb Client — polls the Bulb API and drives an active-high relay.
 *
 * Wiring (active-high relay module, e.g. SRD-05VDC):
 *   ESP8266      Relay
 *   -------      -----
 *   D1 (GPIO5) → IN
 *   3.3V       → VCC  (use 5V if relay module has a 5V jumper)
 *   GND        → GND
 *
 *   Relay        Bulb / Load
 *   -----        -----------
 *   COM (common) → Live wire from mains
 *   NO (norm open) → Bulb → Neutral
 *
 * The relay triggers on GPIO HIGH (active-high).
 */

#include <ESP8266WiFi.h>
#include <ESP8266HTTPClient.h>

// ── Config ──────────────────────────────────────────────────────────
const char* WIFI_SSID    = "ianfebi01";
const char* WIFI_PASS    = "huihuihui";
const char* SERVER_URL   = "https://bulb-api.ianfebisastrataruna.my.id";

const int   RELAY_PIN    = 0;   // D1 (GPIO5)
const unsigned long POLL_MS = 2000;  // poll server every 2 s
// ─────────────────────────────────────────────────────────────────────

bool currentState = false;       // true = on, false = off
unsigned long lastPoll = 0;

void setup() {
  Serial.begin(115200);
  Serial.println("\n💡 ESP8266 Bulb Client");

  pinMode(RELAY_PIN, OUTPUT);
  digitalWrite(RELAY_PIN, LOW);   // off at boot

  // Connect WiFi
  WiFi.begin(WIFI_SSID, WIFI_PASS);
  Serial.print("Connecting");
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.print("\nWiFi connected — IP: ");
  Serial.println(WiFi.localIP());

  // Initial state from server
  fetchState();
  Serial.print("Initial relay state: ");
  Serial.println(currentState ? "ON" : "OFF");
  Serial.println("Ready — waiting for state changes...\n");
}

void loop() {
  unsigned long now = millis();

  // ── Poll server ────────────────────────────────────────
  if (now - lastPoll >= POLL_MS) {
    lastPoll = now;
    fetchState();
  }
}

// ── Fetch bulb state from GET /bulb ─────────────────────
void fetchState() {
  if (WiFi.status() != WL_CONNECTED) {
    Serial.println("Poll skipped: WiFi not connected");
    return;
  }

  WiFiClientSecure client;
  client.setInsecure();  // skip TLS cert validation (ESP8266 lacks root CAs)

  HTTPClient http;
  http.useHTTP10(true);
  http.begin(client, String(SERVER_URL) + "/bulb");

  Serial.print("[HTTP] GET ");
  Serial.print(SERVER_URL);
  Serial.println("/bulb");

  int code = http.GET();
  Serial.print("[HTTP] Response code: ");
  Serial.println(code);

  if (code > 0) {
    String payload = http.getString();
    bool serverState = payload.indexOf("\"is_on\":true") >= 0;

    Serial.print("[HTTP] Body: ");
    Serial.println(payload);
    Serial.print("[HTTP] Parsed is_on: ");
    Serial.println(serverState ? "true" : "false");

    applyState(serverState);
  } else {
    Serial.print("[HTTP] Error: ");
    Serial.println(http.errorToString(code).c_str());
  }
  http.end();
}

// ── Drive relay ─────────────────────────────────────────
void applyState(bool on) {
  if (on == currentState) return;
  currentState = on;
  digitalWrite(RELAY_PIN, on ? HIGH : LOW);
  Serial.print("Relay ");
  Serial.println(on ? "ON" : "OFF");
}
