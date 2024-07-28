#pragma once

#include "route.hpp"
#include "../services.hpp"

class LedRoute : public Route
{
    private:
    LedService *led_service;
    PairingService *pairing_service;
public:
    LedRoute(LedService* l, PairingService *p) : Route("/led", "POST", false)
    {
        led_service = l;
        pairing_service = p;
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
        int next_val = -1;
        char *path = req->route + 4;
        if (strcmp(path, "/on") == 0)
        {
            next_val = 1;
        }
        else if (strcmp(path, "/off") == 0)
        {
            next_val = 0;
        }
        else
        {
            json["error"] = "Invalid path, must be /led/on or /led/off";
            write_json(client, &json, sBAD_REQUEST);
            return;
        }
        led_service->blinking = next_val;
        json["result"] = "ok";
        write_json(client, &json);
    }
};