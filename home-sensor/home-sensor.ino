#include "my-dht.h"
#include "my-wifi.h"
#include "my-server.h"
#include "secret.h"

#define BUTTON_PIN 6
#define LED_PIN 7

#define DHT_TIMEOUT_MS = 20000;
ulong next_dht_read = 0;

void setup() {
  pinMode(LED_PIN, OUTPUT);
  digitalWrite(LED_PIN, HIGH);
  Serial.begin(9600);
  connect_wifi(ssid, pass);
  dht_init();
  server_init();
  next_dht_read = millis() + DHT_TIMEOUT_MS;
  digitalWrite(7, LOW);
}

void loop() {
  if (millis() >= next_dht_read) {
    struct DhtMeasurement measurement = read_dht_measurement();
    print_dht_measurement(&measurement);
    next_dht_read = millis() + DHT_TIMEOUT_MS;
  }

  handle_client();
}