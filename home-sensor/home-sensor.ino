#include "src/dht.h"
#include "src/wifi.h"
#include "src/server.h"
#include "src/pairing.h"
#include "src/secret.h"

#define LED_PIN 7

void setup() {
  pinMode(LED_PIN, OUTPUT);
  digitalWrite(LED_PIN, HIGH);
  Serial.begin(9600);
  connect_wifi(ssid, pass);
  dht_init();
  server_init();
  pairing_init();
  digitalWrite(LED_PIN, LOW);
}

void loop() {
  handle_dht();
  handle_pairing();
  handle_client();
}