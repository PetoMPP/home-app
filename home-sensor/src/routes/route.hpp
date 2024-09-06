#pragma once

#include "../http.hpp"

#define RESPONSE_BODY_LEN 16 * 1024

class Route
{
protected:
    const char *route;
    const char *method;
    bool strict;
    char *json_str = new char[RESPONSE_BODY_LEN];

public:
    Route(const char *r, const char *m, bool s = true)
    {
        route = r;
        method = m;
        strict = s;
    }
    virtual bool match(Request *req)
    {
        if (strict)
        {
            return strcmp(req->method, method) == 0 && strcmp(req->route, route) == 0;
        }
        return strcmp(req->method, method) == 0 && strncmp(req->route, route, strlen(route)) == 0;
    }
    virtual void write_response(NetworkClient *client, Request *req) = 0;
    void write_json(NetworkClient *client, JsonDocument json, Status status = sOK)
    {
        int json_len = serializeJsonPretty(json, json_str, RESPONSE_BODY_LEN);
        client->println(get_status_header(status));
        client->println("Content-Type: application/json");
        client->printf("Content-Length: %d\r\n", json_len);
        client->println();
        client->print(json_str);
    }
};
