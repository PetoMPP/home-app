#include <WiFi.h>
#include "my-http.h"

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
        client.println("HTTP/1.1 400 BAD REQUEST");
      } else {
        // ROUTES GO HERE
        client.println("HTTP/1.1 200 OK");
      }

      client.println();
      client.flush();
      client.stop();
    }
  }
}
