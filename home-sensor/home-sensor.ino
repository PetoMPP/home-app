#include "my-dht.h"
#include "my-wifi.h"
#include "secret.h"

void setup() {
  Serial.begin(9600);
  Serial.println(F("DHTxx test!"));

  connect_wifi(ssid, pass);

  dht_init();
}

void loop() {
  delay(2000);
  struct DhtMeasurement measurement = read_dht_measurement();
  print_dht_measurement(&measurement);
  Serial.println(F("DONE!!"));
}