#pragma once

#include "../http.h"

class Route
{
public:
    Route() {}
    virtual bool match(Request *req) = 0;
    virtual void write_response(NetworkClient *client, Request *req) = 0;
};

void write_json(NetworkClient *client, JsonDocument* json_ptr, Status status = sOK)
{
    JsonDocument json = JsonDocument(*json_ptr);
    char *json_str = new char[1024];
    int json_len = serializeJsonPretty(json, json_str, 1024);
    client->println(get_status_header(status));
    client->println("Content-Type: application/json");
    client->printf("Content-Length: %d\r\n", json_len);
    client->println();
    client->print(json_str);
}