#pragma once

#include <WiFi.h>
#include "../http.h"
#include "../routes.hpp"
#include "pairing.hpp"
#include "sensor.hpp"

#define REQ_BUFF_LEN 2048

class ServerService : ServiceBase
{
private:
    WiFiServer server;
    std::vector<Route *> routes;
    uint8_t *req_buff = new uint8_t[REQ_BUFF_LEN];

public:
    ServerService(std::vector<Route *> all_routes)
    {
        server = WiFiServer(42069);
        this->routes = all_routes;
    }
    void init() override
    {
        server.begin();
    }
    void handle() override
    {
        if (!server.hasClient())
            return;

        NetworkClient client = server.accept();
        if (!client)
            return;

        int len = client.read(req_buff, REQ_BUFF_LEN);
        Request *req = parse_request(req_buff, len);
        if (req == NULL)
        {
            client.println(get_status_header(sBAD_REQUEST));
            client.println();
        }
        else
        {
            Route *route = new NotFoundRoute();
            for (Route *r : routes)
            {
                if (r->match(req))
                {
                    route = r;
                    break;
                }
            }
            route->write_response(&client, req);
        }

        client.flush();
        client.stop();
    }
};