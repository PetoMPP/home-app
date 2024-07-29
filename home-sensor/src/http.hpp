#pragma once

struct Request
{
  char *method;
  char *route;
  char *headers;
  char *body;
};

void get_header_value(Request *req, const char *header_key, char *header_value)
{
  const char *headers = req->headers;
  const char *key_start = strcasestr(headers, header_key);
  if (key_start == NULL)
  {
    header_value = NULL;
    return;
  }
  const char *value_start = key_start + strlen(header_key) + 2;
  const char *value_end = strstr(value_start, "\r\n");
  if (value_end == NULL)
  {
    header_value = NULL;
    return;
  }

  strncpy(header_value, value_start, value_end - value_start);
  header_value[value_end - value_start] = '\0';
}

#define METHOD_LEN 5
#define ROUTE_LEN 64
#define HEADERS_LEN 512
#define BODY_LEN 1024
// Does not handle binary data
struct Request curr_req = {new char[METHOD_LEN], new char[ROUTE_LEN], new char[HEADERS_LEN], new char[BODY_LEN]};

struct Request *parse_request(char *req_buff, int len)
{
  const char *start = req_buff;
  // first line `GET / HTTP/1.1`
  const char *end = strstr(start, "\r\n");
  if (end == NULL)
  {
    return NULL;
  }
  // method
  char *me = strchr(start, ' ');
  if (me == NULL)
  {
    return NULL;
  }
  size_t ml = me - start;
  if (ml > METHOD_LEN) { ml = METHOD_LEN; }
  strncpy(curr_req.method, start, ml);
  curr_req.method[ml] = '\0';
  // route
  char *re = strchr(me + 1, ' ');
  if (re == NULL)
  {
    return NULL;
  }
  char *rs = me + 1;
  size_t rl = re - rs;
  if (rl > ROUTE_LEN) { rl = ROUTE_LEN; }
  strncpy(curr_req.route, rs, rl);
  curr_req.route[rl] = '\0';
  // headers
  start = end + 2;
  end = strstr(start, "\r\n\r\n");
  if (end == NULL)
  {
    return NULL;
  }
  size_t hl = (end + 2) - start; // take one line break
  if (hl > HEADERS_LEN) { hl = HEADERS_LEN; }
  strncpy(curr_req.headers, start, hl);
  curr_req.headers[hl] = '\0';
  // body
  start = end + 4;
  strncpy(curr_req.body, start, strnlen(start, BODY_LEN)); // take all valid string

  return &curr_req;
}

enum Status
{
  sOK = 200,
  sBAD_REQUEST = 400,
  sUNAUTHORIZED = 401,
  sFORBIDDEN = 403,
  sNOT_FOUND = 404,
  sINTERNAL_SERVER_ERROR = 500
};

const char *get_status_header(Status status)
{
  switch (status)
  {
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
