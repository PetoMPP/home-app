#pragma once

#include "route.hpp"
#include "../services.hpp"

class PairRoute : public Route
{
private:
    PairingService *pairing_service;

public:
    PairRoute(PairingService *p) : Route("/pair", "POST")
    {
        pairing_service = p;
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
    PairConfirmRoute(PairingService *p) : Route("/pair/confirm", "POST")
    {
        pairing_service = p;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        JsonDocument json;
        Status status = sOK;
        if (!pairing_service->pairing)
        {
            json["error"] = PairingService::ERROR_MESSAGE;
            status = sUNAUTHORIZED;
        }
        else if (!pairing_service->pair(req))
        {
            json["error"] = "Invalid pairing key";
            status = sBAD_REQUEST;
        }
        else
        {
            json["result"] = "success";
        }

        write_json(client, &json, status);
    }
};