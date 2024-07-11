#pragma once

#include <WiFi.h>
#include "http.h"
#include "routes.h"

WiFiServer server(42069);

void server_init() {
  server.begin();
}

#define REQ_BUFF_LEN 2048

uint8_t* req_buff = new uint8_t[REQ_BUFF_LEN];

void handle_client() {
  if (server.hasClient()) {
    NetworkClient client = server.accept();
    if (client) {
      int len = client.read(req_buff, REQ_BUFF_LEN);
      Request* req = parse_request(req_buff, len);
      if (req == NULL) {
        client.println(get_status_header(sBAD_REQUEST));
        client.println();
      } else {
        Route route = match_route(req);
        write_response(&client, req, route);
      }

      client.flush();
      client.stop();
    }
  }
}
