#pragma once

struct Request {
  char* method;
  char* route;
  char* headers;
  char* body;
};

void get_header_value(Request* req, const char* header_key, char* header_value) {
  const char* headers = req->headers;
  const char* key_start = strcasestr(headers, header_key);
  if (key_start == NULL) {
    header_value = NULL;
    return;
  }
  const char* value_start = key_start + strlen(header_key) + 2;
  const char* value_end = strstr(value_start, "\r\n");
  if (value_end == NULL) {
    header_value = NULL;
    return;
  }

  strncpy(header_value, value_start, value_end - value_start);
  header_value[value_end - value_start] = '\0';
}

struct Request curr_req;

struct Request* parse_request(uint8_t* req_buff, int len) {
  char buff[len] = { 0 };
  for (int i = 0; i < len; i++) {
    buff[i] = req_buff[i];
  }
  delete[] curr_req.method;
  delete[] curr_req.route;
  delete[] curr_req.headers;
  delete[] curr_req.body;
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
  sUNAUTHORIZED = 401,
  sFORBIDDEN = 403,
  sNOT_FOUND = 404,
  sINTERNAL_SERVER_ERROR = 500
};

const char* get_status_header(Status status) {
  switch (status) {
    case sOK:
      return "HTTP/1.1 200 OK";
    case sBAD_REQUEST:
      return "HTTP/1.1 400 Bad Request";
    case sUNAUTHORIZED:
      return "HTTP/1.1 401 Unauthorized";
    case sFORBIDDEN:
      return "HTTP/1.1 403 Forbidden";
    case sNOT_FOUND:
      return "HTTP/1.1 404 Not Found";
    default:
      return "HTTP/1.1 500 Internal Server Error";
  }
}
