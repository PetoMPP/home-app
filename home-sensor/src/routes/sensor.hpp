#pragma once

#include "route.hpp"
#include "../services.hpp"

class GetSensorRoute : public Route
{
private:
    SensorService *data_service;
    PairingService *pairing_service;

public:
    GetSensorRoute(SensorService *s, PairingService *p) : Route("/sensor", "GET", false)
    {
        data_service = s;
        pairing_service = p;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        bool is_full = strcmp(req->route, "/sensor/full") == 0;
        if (is_full && !pairing_service->is_paired(req))
        {
            JsonDocument json;
            json["error"] = PairingService::ERROR_MESSAGE;
            write_json(client, json, sUNAUTHORIZED);
            return;
        }

        SensorStore *data_store = data_service->store;
        JsonDocument json;
        json = JsonDocument(*data_store->as_json());
        json["pairing"] = pairing_service->pairing;
        if (is_full)
        {
            PairStore *pair_store = pairing_service->store;
            json["paired_keys"] = pair_store->count;
            JsonDocument jobj;
            JsonObject usage = jobj.to<JsonObject>();
            usage["data_used"] = data_store->len;
            usage["data_total"] = data_store->max_size;
            usage["pair_used"] = pair_store->len;
            usage["pair_total"] = pair_store->max_size;
            json["usage"] = usage;
            json["free_mem"] = esp_get_free_heap_size();
            json["uptime"] = ((double)esp_timer_get_time()) / (1000 * 1000);
        }

        write_json(client, json);
    }
};

class PostSensorRoute : public Route
{
private:
    SensorService *data_service;
    PairingService *pairing_service;

public:
    PostSensorRoute(SensorService *s, PairingService *p) : Route("/sensor", "POST")
    {
        data_service = s;
        pairing_service = p;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        JsonDocument json;
        if (!pairing_service->is_paired(req))
        {
            json["error"] = PairingService::ERROR_MESSAGE;
            write_json(client, json, sUNAUTHORIZED);
            return;
        }
        DeserializationError err = deserializeJson(json, req->body, 1024);
        if (err)
        {
            client->println(get_status_header(sBAD_REQUEST));
            client->println();
            return;
        }

        SensorStore *store = data_service->store;
        store->load_json(json, true);
        store->init_json();
        data_service->save_store();
        JsonDocument response;
        response["result"] = "ok";
        write_json(client, response);
    }
};