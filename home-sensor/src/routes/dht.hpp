#pragma once

#include <ArduinoJson.h>

#include "route.hpp"
#include "../services.hpp"

struct DhtRequest
{
    time_t timestamp;
    size_t count;
};

class DhtRoute : public Route
{
private:
    DhtService *dht_service;
    PairingService *pairing_service;
    JsonDocument measurement_as_json(DhtMeasurement *m)
    {
        JsonDocument json;
        json["timestamp"] = m->timestamp;
        json["temperature"] = m->temp_c;
        json["humidity"] = m->hum;
        return json;
    }

public:
    DhtRoute(DhtService *d, PairingService* p) : Route("/dht", "GET")
    {
        dht_service = d;
        pairing_service = p;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        JsonDocument json;
        JsonDocument arr;
        if (!pairing_service->is_paired(req))
        {
            json["error"] = PairingService::ERROR_MESSAGE;
            write_json(client, &json, sUNAUTHORIZED);
            return;
        }
        time_t now;
        time(&now);
        DhtRequest dht_request = {now, 1};
        JsonDocument req_json;
        DeserializationError err = deserializeJson(req_json, req->body, 1024);
        if (!err)
        {
            if (req_json.containsKey("timestamp"))
            {
                dht_request.timestamp = req_json["timestamp"];
            }
            if (req_json.containsKey("count"))
            {
                dht_request.count = req_json["count"];
            }
        }
        size_t added = 0;
        size_t offset = dht_service->last_measurement_idx;
        for (size_t i = 0; i < DHT_STORAGE_ENTRIES; i++)
        {
            DhtMeasurement *m = &dht_service->measurements[(offset - i) % DHT_STORAGE_ENTRIES];
            if (m->timestamp == 0)
            {
                break;
            }
            if (m->timestamp > dht_request.timestamp)
            {
                continue;
            }
            arr.add(measurement_as_json(m));
            added++;
            if (added >= dht_request.count)
            {
                break;
            }
        }

        json["measurements"] = arr;
        write_json(client, &json);
        return;
    }
};
