#pragma once

#include <DHT.h>

#define DHTPIN 5
#define DHTTYPE DHT11
#define DHT_TIMEOUT_MS 20000;

ulong next_dht_read = 0;

DHT dht(DHTPIN, DHTTYPE);

struct DhtMeasurement {
  float hum;
  float temp_c;
  float temp_f;
  float heat_index_c;
  float heat_index_f;
};

void dht_init() {
  dht.begin();
}

struct DhtMeasurement
read_dht_measurement() {
  struct DhtMeasurement result;
  // Reading temperature or humidity takes about 250 milliseconds!
  // Sensor readings may also be up to 2 seconds 'old' (its a very slow sensor)
  float h = dht.readHumidity();
  // Read temperature as Celsius (the default)
  float t = dht.readTemperature();
  // Read temperature as Fahrenheit (isFahrenheit = true)
  float f = dht.readTemperature(true);

  // Check if any reads failed and exit early (to try again).
  if (isnan(h) || isnan(t) || isnan(f)) {
    Serial.println(F("Failed to read from DHT sensor!"));
    return result;
  }

  // Compute heat index in Fahrenheit (the default)
  float hif = dht.computeHeatIndex(f, h);
  // Compute heat index in Celsius (isFahreheit = false)
  float hic = dht.computeHeatIndex(t, h, false);

  result = DhtMeasurement{ h, t, f, hic, hif };

  return result;
}

void print_dht_measurement(DhtMeasurement* measurement) {
  Serial.print(F("Humidity: "));
  Serial.print(measurement->hum);
  Serial.print(F("%  Temperature: "));
  Serial.print(measurement->temp_c);
  Serial.print(F("°C "));
  Serial.print(measurement->temp_f);
  Serial.print(F("°F  Heat index: "));
  Serial.print(measurement->heat_index_c);
  Serial.print(F("°C "));
  Serial.print(measurement->heat_index_f);
  Serial.println(F("°F"));
}

void handle_dht() {
  if (millis() >= next_dht_read) {
    struct DhtMeasurement measurement = read_dht_measurement();
    print_dht_measurement(&measurement);
    next_dht_read = millis() + DHT_TIMEOUT_MS;
  }
}