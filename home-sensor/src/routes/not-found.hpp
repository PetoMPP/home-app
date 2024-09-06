#pragma once

#include "route.hpp"

class NotFoundRoute : public Route
{
public:
    NotFoundRoute() : Route("", "")
    {
    }
    bool match(Request *req) override
    {
        return true;
    }

    void write_response(NetworkClient *client, Request *req) override
    {
        JsonDocument json;
        json["error"] = "Not found";
        write_json(client, json, sNOT_FOUND);
    }
};