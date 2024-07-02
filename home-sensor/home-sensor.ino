#include "my-dht.h"
#include "my-wifi.h"
#include "my-server.h"
#include "my-pairing.h"
#include "my-secret.h"

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