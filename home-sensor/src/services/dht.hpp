#pragma once

#include <DHT.h>
#include <Preferences.h>
#include "service.hpp"

#define DHTPIN 5
#define DHTTYPE DHT11
#define DHT_MEASUREMENT_TIMEOUT_M 15
#define DHT_MEASUREMENT_TIMEOUT_S DHT_MEASUREMENT_TIMEOUT_M * 60
#define DHT_SAVE_TIMEOUT_MS 2 * 60 * 60 * 1000
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
    ulong next_save;
    void handle_measurement()
    {
        time_t now;
        time(&now);
        time_t rounded_now = round_to_timeout(now);
        time_t diff = now - rounded_now;
        if (diff < 0 || diff > 2) // up to 2s after rounded time
        {
            return;
        }
        time_t timestamp = round_to_timeout(measurements[last_measurement_idx].timestamp);
        if (now - timestamp < DHT_MEASUREMENT_TIMEOUT_S)
        {
            return;
        }
        float h = dht->readHumidity();
        float t = dht->readTemperature();
        if (isnan(h) || h == 0.0 || isnan(t) || t == 0.0)
        {
            Serial.println("Error reading from DHT!");
            return;
        }
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
    }
    void handle_save(ulong *start_ms, bool force = false)
    {
        if (!force && *start_ms < next_save)
        {
            return;
        }
        next_save = *start_ms + DHT_SAVE_TIMEOUT_MS;
        char *readings = new char[DHT_STORAGE_SIZE];
        readings[0] = last_measurement_idx;
        memcpy(readings + 1, measurements, DHT_STORAGE_ENTRIES * sizeof(DhtMeasurement));
        prefs->begin("dht");
        prefs->putBytes("bytes", readings, DHT_STORAGE_SIZE);
        prefs->end();
        Serial.println("Saved readings!");
    }

    time_t round_to_timeout(time_t timestamp)
    {
        // Convert the timestamp to minutes to avoid overflow
        int32_t minutes = timestamp / 60;
        // Round to the nearest 15 minutes in minutes
        int32_t rounded_minutes = ((minutes + 7) / DHT_MEASUREMENT_TIMEOUT_M) * DHT_MEASUREMENT_TIMEOUT_M;
        // Convert back to seconds
        return rounded_minutes * 60;
    }

protected:
    void handle_inner(ulong *start_ms) override
    {
        handle_measurement();
        handle_save(start_ms);
    }

public:
    size_t last_measurement_idx = 0;
    DhtService(Preferences *p)
    {
        prefs = p;
    }
    DhtMeasurement measurements[DHT_STORAGE_ENTRIES];
    void init() override
    {
        dht = new DHT(DHTPIN, DHTTYPE);
        dht->begin();
        prefs->begin("dht");
        char *readings = new char[DHT_STORAGE_SIZE];
        size_t len = prefs->getBytes("bytes", readings, DHT_STORAGE_SIZE);
        prefs->end();
        next_save = millis() + DHT_SAVE_TIMEOUT_MS;
        if (len == 0)
        {
            Serial.println("No readings found!");
            return;
        }
        last_measurement_idx = (size_t)readings[0];
        memcpy(measurements, readings + 1, DHT_STORAGE_ENTRIES * sizeof(DhtMeasurement));
    }

    void save()
    {
        ulong now = millis();
        handle_save(&now, true);
    }
};