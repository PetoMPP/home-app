#pragma once

#include <DHT.h>
#include <Preferences.h>
#include "service.hpp"

#define DHTPIN 5
#define DHTTYPE DHT11
// #define DHT_MEASUREMENT_TIMEOUT_S 15 * 60
// #define DHT_SAVE_TIMEOUT_S 2 * 60 * 60
#define DHT_MEASUREMENT_TIMEOUT_S 30
#define DHT_SAVE_TIMEOUT_S 2 * 60
#define DHT_STORAGE_ENTRIES 150

struct DhtMeasurement
{
    time_t timestamp;
    float hum;
    float temp_c;
};

#define DHT_STORAGE_SIZE DHT_STORAGE_ENTRIES * sizeof(DhtMeasurement) + 1

class DhtService : public ServiceBase
{
private:
    DHT *dht;
    Preferences *prefs;
    size_t last_measurement_idx;
    void handle_measurement()
    {
        time_t now;
        time(&now);
        if (now - last_measurement->timestamp < DHT_MEASUREMENT_TIMEOUT_S)
        {
            return;
        }
        float h = dht->readHumidity();
        float t = dht->readTemperature();
        if (isnan(h) || h == 0.0 || isnan(t) || t == 0.0)
        {
            return;
        }
        time(&now);
        DhtMeasurement m = {now, h, t};
        Serial.print(F("Time: "));
        Serial.print(now);
        Serial.print(F(" Humidity: "));
        Serial.print(h);
        Serial.print(F("%  Temperature: "));
        Serial.print(t);
        Serial.println(F("°C "));

        last_measurement_idx++;
        if (last_measurement_idx >= DHT_STORAGE_ENTRIES)
        {
            last_measurement_idx = 0;
        }
        measurements[last_measurement_idx] = m;
        last_measurement = &measurements[last_measurement_idx];
    }

public:
    DhtService(Preferences *p)
    {
        prefs = p;
    }
    DhtMeasurement *last_measurement;
    DhtMeasurement measurements[DHT_STORAGE_ENTRIES];
    void handle_save(bool force = false)
    {
        time_t now;
        time(&now);
        if (!force && now - last_measurement->timestamp < DHT_SAVE_TIMEOUT_S)
        {
            return;
        }
        char *readings = new char[DHT_STORAGE_SIZE];
        readings[0] = last_measurement_idx;
        memcpy(readings + 1, measurements, DHT_STORAGE_ENTRIES * sizeof(DhtMeasurement));
        prefs->begin("dht");
        prefs->putBytes("bytes", readings, DHT_STORAGE_SIZE);
        prefs->end();
        Serial.println("Saved readings!");
    }
    void init() override
    {
        dht = new DHT(DHTPIN, DHTTYPE);
        dht->begin();
        prefs->begin("dht");
        char *readings = new char[DHT_STORAGE_SIZE];
        size_t len = prefs->getBytes("bytes", readings, DHT_STORAGE_SIZE);
        prefs->end();
        if (len == 0)
        {
            Serial.println("No readings found!");
            return;
        }
        last_measurement_idx = (size_t)readings[0];
        memcpy(measurements, readings + 1, DHT_STORAGE_ENTRIES * sizeof(DhtMeasurement));
        last_measurement = &measurements[last_measurement_idx];
    }
    void handle() override
    {
        handle_measurement();
        handle_save();
    }
};