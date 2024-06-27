#pragma once

#include <WiFi.h>
#include "my-http.h"

enum Route {
  rSENSOR,
  rNOT_FOUND = 100
};

Route match_route(Request* req) {
  const char* route = req->route;
  const char* method = req->method;
  if (strcmp(route, "/sensor") == 0 && strcmp(method, "GET") == 0) {
    return rSENSOR;
  }

  return rNOT_FOUND;
}

void write_response(NetworkClient* client, Route route) {
  const char* json;
  switch (route) {
    case rSENSOR:
      json = "{\r\n    \"sensor\": \"ok\"\r\n}";
      client->println(get_status_header(sOK));
      client->println("Content-Type: application/json");
      client->printf("Content-Length: %d\r\n", strlen(json));
      client->println();
      client->print(json);
      break;

    default:
      client->println(get_status_header(sNOT_FOUND));
      client->println();
      break;
  }
}