#pragma once

#include "route.hpp"
#include "../services.hpp"

class GetSensorRoute : public Route
{
private:
    SensorService *data_service;
    PairingService *pairing_service;

public:
    GetSensorRoute(SensorService *s, PairingService *p)
    {
        data_service = s;
        pairing_service = p;
    }
    bool match(Request *req) override
    {
        const char *method = req->method;
        const char *route = req->route;
        return strcmp(method, "GET") == 0 && strcmp(route, "/sensor") == 0;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        SensorStore *data_store = data_service->get_store();
        PairStore *pair_store = pairing_service->get_store();
        JsonDocument json = JsonDocument(*data_store->as_json());
        json["paired_keys"] = pair_store->count;
        JsonDocument jobj;
        JsonObject usage = jobj.to<JsonObject>();
        usage["data_used"] = data_store->len;
        usage["data_total"] = data_store->max_size;
        usage["pair_used"] = pair_store->len;
        usage["pair_total"] = pair_store->max_size;
        json["usage"] = usage;

        write_json(client, &json);
    }
};

class PostSensorRoute : public Route
{
private:
    SensorService *data_service;
    PairingService *pairing_service;

public:
    PostSensorRoute(SensorService *s, PairingService *p)
    {
        data_service = s;
        pairing_service = p;
    }
    bool match(Request *req) override
    {
        const char *method = req->method;
        const char *route = req->route;
        return strcmp(method, "POST") == 0 && strcmp(route, "/sensor") == 0;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        JsonDocument json;
        if (!pairing_service->is_paired(req))
        {
            json["error"] = PairingService::ERROR_MESSAGE;
            write_json(client, &json, sUNAUTHORIZED);
            return;
        }
        DeserializationError err = deserializeJson(json, req->body, 1024);
        if (err)
        {
            client->println(get_status_header(sBAD_REQUEST));
            client->println();
            return;
        }

        SensorStore *store = data_service->get_store();
        store->load_json(json, true);
        data_service->save_store();
        JsonDocument response;
        response["result"] = "ok";
        write_json(client, &response);
    }
};