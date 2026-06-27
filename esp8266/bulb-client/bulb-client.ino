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
const char* SERVER_URL   = "http://192.168.1.8:3000";  // your Mac's IP

const int   RELAY_PIN    = 5;   // D1 (GPIO5)
// const int   BTN_PIN      = 4;   // D2 (GPIO4) — uncomment if button wired
const unsigned long POLL_MS = 2000;  // poll server every 2 s
// ─────────────────────────────────────────────────────────────────────

bool currentState = false;       // true = on, false = off
unsigned long lastPoll = 0;
unsigned long lastBtn = 0;
const unsigned long DEBOUNCE = 300;

void setup() {
  Serial.begin(115200);
  Serial.println("\n💡 ESP8266 Bulb Client");

  pinMode(RELAY_PIN, OUTPUT);
  digitalWrite(RELAY_PIN, LOW);   // off at boot
  // pinMode(BTN_PIN, INPUT_PULLUP); // uncomment if button wired

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

  // ── Button press (optional) — uncomment if button wired ─
  // if (digitalRead(BTN_PIN) == LOW && (now - lastBtn > DEBOUNCE)) {
  //   lastBtn = now;
  //   toggleAndSync();
  // }

  // ── Poll server ────────────────────────────────────────
  if (now - lastPoll >= POLL_MS) {
    lastPoll = now;
    fetchState();
  }
}

// ── Fetch bulb state from GET /bulb ─────────────────────
void fetchState() {
  if (WiFi.status() != WL_CONNECTED) return;

  WiFiClient client;
  HTTPClient http;
  http.useHTTP10(true);
  http.begin(client, String(SERVER_URL) + "/bulb");

  int code = http.GET();
  if (code > 0) {
    String payload = http.getString();
    bool serverState = payload.indexOf("\"is_on\":true") >= 0;
    Serial.print("Poll: code=");
    Serial.print(code);
    Serial.print(" is_on=");
    Serial.println(serverState ? "true" : "false");
    applyState(serverState);
  } else {
    Serial.print("Poll failed: code=");
    Serial.println(code);
  }
  http.end();
}

// ── Toggle local + sync to server ───────────────────────
void toggleAndSync() {
  bool newState = !currentState;
  applyState(newState);

  WiFiClient client;
  HTTPClient http;
  http.useHTTP10(true);
  http.begin(client, String(SERVER_URL) + "/bulb");
  http.addHeader("Content-Type", "application/json");

  String body = newState ? "{\"is_on\":true}" : "{\"is_on\":false}";
  http.PUT(body);
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
