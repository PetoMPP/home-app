#include <Preferences.h>

#include "src/services.hpp"
#include "src/secret.h"

Preferences prefs;
LedService *led_service = new LedService();
WifiService *wifi_service = new WifiService(ssid, pass);
PairingService *pairing_service = new PairingService(&prefs);
SensorService *sensor_service = new SensorService(&prefs);
DhtService *dht_service = new DhtService(&prefs);

std::vector<Route *> all_routes = {
    new GetSensorRoute(sensor_service, pairing_service),
    new PostSensorRoute(sensor_service, pairing_service),
    new PairRoute(pairing_service),
    new PairConfirmRoute(pairing_service),
    new DhtRoute(dht_service, pairing_service),
    new LedRoute(led_service, pairing_service),
};

ServerService *server_service = new ServerService(all_routes);

void setup()
{
  Serial.begin(9600);
  led_service->init();
  led_service->set(HIGH);
  wifi_service->init();
  dht_service->init();
  configTime(0, 0, "pool.ntp.org", "time.nist.gov");
  server_service->init();
  pairing_service->init();
  sensor_service->init();
  led_service->set(LOW);
}

void loop()
{
  ulong led_elapsed = led_service->handle();
  ulong dht_elapsed = dht_service->handle();
  ulong pairing_elapsed = pairing_service->handle();
  ulong wifi_elapsed = wifi_service->handle();
  ulong server_elapsed = server_service->handle();
  ulong elapsed = led_elapsed + dht_elapsed + pairing_elapsed + wifi_elapsed + server_elapsed;
  if (elapsed <= 10)
  {
    return;
  }
  Serial.println("Long loop time detected:");
  Serial.print("LED: ");
  Serial.println(led_elapsed);
  Serial.print("DHT: ");
  Serial.println(dht_elapsed);
  Serial.print("Pairing: ");
  Serial.println(pairing_elapsed);
  Serial.print("WiFi: ");
  Serial.println(wifi_elapsed);
  Serial.print("Server: ");
  Serial.println(server_elapsed);
  Serial.print("Total: ");
  Serial.println(elapsed);
}