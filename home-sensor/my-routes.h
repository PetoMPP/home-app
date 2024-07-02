#pragma once

#include <WiFi.h>
#include <ArduinoJson.h>
#include "my-http.h"
#include "my-data.h"
#include "my-pairing.h"

enum Route {
  rSENSOR_GET,
  rSENSOR_POST,
  rPAIR,
  rPAIR_CONFIRM,
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
  if (strcmp(route, "/pair") == 0 && strcmp(method, "POST") == 0) {
    return rPAIR;
  }
  if (strcmp(route, "/pair/confirm") == 0 && strcmp(method, "POST") == 0) {
    return rPAIR_CONFIRM;
  }

  return rNOT_FOUND;
}

void write_response(NetworkClient* client, Request* req, Route route) {
  bool paired = false;
  int pair_len = 0;
  struct PairStore pair_store = get_pair_store(&pair_len);
  char pk[64] = { 0 };
  get_header_value(req, "X-Pair-Id", pk);
  const char* pair_key = pk;
  if (pair_key != NULL) {
    if (has_pair_key(&pair_store, pair_key)) {
      paired = true;
    }
  }
  switch (route) {
    case rSENSOR_GET:
      {
        int data_len = 0;
        JsonDocument json = data_store_to_json(get_data_store(&data_len));
        json["pairing"] = pairing;
        json["paired_keys"] = pair_store.count;
        JsonDocument jobj;
        JsonObject usage = jobj.to<JsonObject>();
        usage["data_used"] = data_len;
        usage["data_total"] = DATA_STORE_SIZE;
        usage["pair_used"] = pair_len;
        usage["pair_total"] = PAIR_STORE_SIZE;
        json["usage"] = usage;
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
        if (!paired) {
          client->println(get_status_header(sUNAUTHORIZED));
          client->println();
          return;
        }

        JsonDocument json;
        DeserializationError err = deserializeJson(json, req->body, 1024);
        if (err) {
          client->println(get_status_header(sBAD_REQUEST));
          client->println();
          return;
        }
        int data_len = 0;
        struct DataStore store = get_data_store(&data_len);
        struct DataStore changes = json_to_data_store(json);
        merge_data_stores(&store, &changes);
        set_data_store(store, &data_len);
        client->println(get_status_header(sOK));
        client->println();
        return;
      }
    case rPAIR:
      {
        JsonDocument json;
        if (!pairing) {
          json["error"] = "To connect use /pair endpoint and pairing button on the device.";
          char* json_str = new char[1024];
          int json_len = serializeJsonPretty(json, json_str, 1024);
          client->println(get_status_header(sUNAUTHORIZED));
          client->println("Content-Type: application/json");
          client->printf("Content-Length: %d\r\n", json_len);
          client->println();
          client->print(json_str);
          return;
        }

        json["id"] = generate_pair_id();
        char* json_str = new char[1024];
        int json_len = serializeJsonPretty(json, json_str, 1024);
        client->println(get_status_header(sOK));
        client->println("Content-Type: application/json");
        client->printf("Content-Length: %d\r\n", json_len);
        client->println();
        client->print(json_str);
        return;
      }
    case rPAIR_CONFIRM:
      {
        JsonDocument json;
        if (!pairing) {
          json["error"] = "To connect use /pair endpoint and pairing button on the device.";
          char* json_str = new char[1024];
          int json_len = serializeJsonPretty(json, json_str, 1024);
          client->println(get_status_header(sUNAUTHORIZED));
          client->println("Content-Type: application/json");
          client->printf("Content-Length: %d\r\n", json_len);
          client->println();
          client->print(json_str);
          return;
        }

        if (pair_key == NULL || !has_pair_key(&curr_pair_keys, pair_key)) {
          json["error"] = "Pair key missing or invalid!";
          char* json_str = new char[1024];
          int json_len = serializeJsonPretty(json, json_str, 1024);
          client->println(get_status_header(sBAD_REQUEST));
          client->println("Content-Type: application/json");
          client->printf("Content-Length: %d\r\n", json_len);
          client->println();
          client->print(json_str);
          return;
        }

        strcpy(pair_store.keys[pair_store.count], pair_key);
        set_pair_store(pair_store, &pair_len);
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