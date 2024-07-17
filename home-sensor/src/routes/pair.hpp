#pragma once

#include "route.hpp"
#include "../services.hpp"

class PairRoute : public Route
{
private:
    PairingService *pairing_service;

public:
    PairRoute(PairingService *p)
    {
        pairing_service = p;
    }
    bool match(Request *req) override
    {
        const char *method = req->method;
        const char *route = req->route;
        return strcmp(method, "POST") == 0 && strcmp(route, "/pair") == 0;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        JsonDocument json;
        if (!pairing_service->pairing)
        {
            json["error"] = PairingService::ERROR_MESSAGE;
            write_json(client, &json, sUNAUTHORIZED);
            return;
        }

        json["id"] = pairing_service->generate();
        write_json(client, &json);
    }
};

class PairConfirmRoute : public Route
{
private:
    PairingService *pairing_service;

public:
    PairConfirmRoute(PairingService *p)
    {
        pairing_service = p;
    }

    bool match(Request *req) override
    {
        const char *method = req->method;
        const char *route = req->route;
        return strcmp(method, "POST") == 0 && strcmp(route, "/pair/confirm") == 0;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        JsonDocument json;
        if (!pairing_service->pairing || !pairing_service->pair(req))
        {
            json["error"] = PairingService::ERROR_MESSAGE;
            write_json(client, &json, sUNAUTHORIZED);
            return;
        }

        json["result"] = "success";
        write_json(client, &json);
    }
};