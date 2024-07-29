#pragma once

#include <WiFi.h>
#include "../http.hpp"
#include "../routes.hpp"
#include "pairing.hpp"
#include "sensor.hpp"

#define REQ_BUFF_LEN 2048

class ServerService : public ServiceBase
{
private:
    WiFiServer server;
    std::vector<Route *> routes;
    Route *not_found_route = new NotFoundRoute();
    char *req_buff = new char[REQ_BUFF_LEN];
    char *last_state = "";
    char *handle_server()
    {
        if (!server)
            return "Not listening";

        if (!server.hasClient())
            return "No client";

        NetworkClient client = server.accept();
        if (!client)
            return "No client accepted";

        int len = client.read((uint8_t*)req_buff, REQ_BUFF_LEN);
        Request *req = parse_request(req_buff, len);
        if (req == NULL)
        {
            client.println(get_status_header(sBAD_REQUEST));
            client.println();
        }
        else
        {
            Route *route = not_found_route;
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
        return "OK";
    }

protected:
    void handle_inner(ulong *start_ms) override
    {
        char *state = handle_server();
        if (strcmp(state, last_state) != 0)
        {
            Serial.print("Server state: ");
            Serial.println(state);
            last_state = state;
        }
    }

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
};