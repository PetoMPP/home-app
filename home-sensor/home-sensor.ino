#include "my-dht.h"
#include "my-wifi.h"
#include "my-server.h"
#include "secret.h"

ulong dht_timeout_ms = 20000;
ulong next_dht_read = 0;

void setup() {
  Serial.begin(9600);
  Serial.println(F("DHTxx test!"));

  connect_wifi(ssid, pass);
  dht_init();
  server_init();
  next_dht_read = millis() + dht_timeout_ms;
}

void loop() {
  if (millis() >= next_dht_read) {
    struct DhtMeasurement measurement = read_dht_measurement();
    print_dht_measurement(&measurement);
    next_dht_read = millis() + dht_timeout_ms;
  }

  handle_client();
}