#pragma once

#include <WiFi.h>
#include <ArduinoJson.h>
#include "my-http.h"
#include "my-store.h"

enum Route {
  rSENSOR_GET,
  rSENSOR_POST,
  rNOT_FOUND = 100
};

Route match_route(Request* req) {
  const char* route = req->route;
  const char* method = req->method;
  if (strcmp(route, "/sensor") == 0) {
    if (strcmp(method, "GET") == 0) {
      return rSENSOR_GET;
    }
    if (strcmp(method, "POST") == 0) {
      return rSENSOR_POST;
    }
  }

  return rNOT_FOUND;
}

void write_response(NetworkClient* client, Request* req, Route route) {
  switch (route) {
    case rSENSOR_GET:
      {
        JsonDocument json = store_to_json(get_data_store());
        char* json_str = new char[1024];
        int json_len = serializeJsonPretty(json, json_str, 1024);
        client->println(get_status_header(sOK));
        client->println("Content-Type: application/json");
        client->printf("Content-Length: %d\r\n", json_len);
        client->println();
        client->print(json_str);
        return;
      }
    case rSENSOR_POST:
      {
        JsonDocument json;
        DeserializationError err = deserializeJson(json, req->body, 1024);
        if (err) {
          client->println(get_status_header(sBAD_REQUEST));
          client->println();
          return;
        }
        struct DataStore store = get_data_store();
        struct DataStore changes = json_to_store(json);
        Serial.printf("Before: n: %s, l: %s, f: %d\r\n", store.name, store.location, store.features);
        merge_stores(&store, &changes);
        Serial.printf("After: n: %s, l: %s, f: %d\r\n", store.name, store.location, store.features);
        set_data_store(store);
        client->println(get_status_header(sOK));
        client->println();
        return;
      }

    default:
      client->println(get_status_header(sNOT_FOUND));
      client->println();
      return;
  }
}