#pragma once

#include "route.hpp"

class NotFoundRoute : public Route {
    public:
        bool match(Request* req) override {
            return true;
        }

        void write_response(NetworkClient* client, Request* req) override {
            client->println("HTTP/1.1 404 Not Found");
            client->println("Content-Type: text/plain");
            client->println();
            client->println("Not Found");
        }
};