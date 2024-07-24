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
    new NotFoundRoute(),
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
  led_service->set(LOW);
}

void loop()
{
  dht_service->handle();
  pairing_service->handle();
  server_service->handle();
}