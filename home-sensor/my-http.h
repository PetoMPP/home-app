#pragma once

struct Request {
  char* method;
  char* route;
  char* headers;
  char* body;
};

struct Request curr_req;

struct Request* parse_request(uint8_t* req_buff, int len) {
  char buff[len] = { 0 };
  for (int i = 0; i < len; i++) {
    buff[i] = req_buff[i];
  }
  curr_req = { new char[5], new char[64], new char[512], new char[1024] };
  const char* start = buff;
  // first line `GET / HTTP/1.1`
  const char* end = strstr(start, "\r\n");
  if (end == NULL) {
    return NULL;
  }
  // method
  char* me = strchr(start, ' ');
  if (me == NULL) {
    return NULL;
  }
  strncpy(curr_req.method, start, me - start);
  curr_req.method[me - start] = '\0';
  // route
  char* re = strchr(me + 1, ' ');
  if (re == NULL) {
    return NULL;
  }
  strncpy(curr_req.route, me + 1, re - (me + 1));
  curr_req.route[re - (me + 1)] = '\0';
  // headers
  start = end + 2;
  end = strstr(start, "\r\n\r\n");
  if (end == NULL) {
    return NULL;
  }
  strncpy(curr_req.headers, start, (end + 2) - start);  // take one line break
  curr_req.headers[(end + 2) - start] = '\0';
  // body
  start = end + 4;
  strcpy(curr_req.body, start);

  return &curr_req;
}

enum Status {
  sOK = 200,
  sBAD_REQUEST = 400,
  sNOT_FOUND = 404,
  sINTERNAL_SERVER_ERROR = 500
};

const char* get_status_header(Status status) {
  switch (status) {
    case sOK:
      return "HTTP/1.1 200 OK";
    case sBAD_REQUEST:
      return "HTTP/1.1 400 Bad Request";
    case sNOT_FOUND:
      return "HTTP/1.1 404 Not Found";
    default:
      return "HTTP/1.1 500 Internal Server Error";
  }
}
