use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Sensor {
    pub name: String<64>,
    pub location: String<64>,
    pub features: u32,
}

pub mod http {
    use crate::http::{status::StatusCode, HEADERS_LEN, RESPONSE_BODY_LEN, RESPONSE_HEADER_LEN};
    use core::ops::{Deref, DerefMut};
    use heapless::{FnvIndexMap, Vec};
    use serde::{de::DeserializeOwned, Serialize};

    #[derive(Debug, Default)]
    pub struct Headers<'h>(FnvIndexMap<&'h str, &'h str, HEADERS_LEN>);

    impl<'h> Deref for Headers<'h> {
        type Target = FnvIndexMap<&'h str, &'h str, HEADERS_LEN>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<'h> DerefMut for Headers<'h> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<'r> Headers<'r> {
        fn into_response(self, code: StatusCode) -> Response {
            let mut resp: Response = code.into();
            for (k, v) in self.iter() {
                resp.extend_from_slice(k.as_bytes()).unwrap();
                resp.extend_from_slice(b": ").unwrap();
                resp.extend_from_slice(v.as_bytes()).unwrap();
                resp.extend_from_slice(b"\r\n").unwrap();
            }
            resp.extend_from_slice(b"\r\n").unwrap();
            resp
        }
    }

    #[derive(Debug)]
    pub struct Request<'r> {
        pub route: &'r str,
        pub method: &'r str,
        pub headers: Headers<'r>,
        pub body: &'r str,
    }

    impl<'r> Request<'r> {
        pub fn body<T: DeserializeOwned>(&self) -> Result<T, StatusCode> {
            Ok(serde_json_core::from_str::<'_, T>(self.body)
                .map_err(|e| {
                    log::warn!("{:?}", e);
                    log::warn!("Failed to parse body: {}", self.body);
                    StatusCode::BAD_REQUEST
                })?
                .0)
        }
    }

    impl<'r> TryFrom<&'r [u8]> for Request<'r> {
        type Error = StatusCode;

        fn try_from(value: &'r [u8]) -> Result<Self, Self::Error> {
            let value = unsafe { core::str::from_utf8_unchecked(&value) };
            let Some(he) = value.find("\r\n\r\n") else {
                return Err(StatusCode::BAD_REQUEST);
            };
            let header_str = &value[..he];
            let mut lines = header_str.lines();
            let path_line = lines.next().ok_or(StatusCode::BAD_REQUEST)?;
            let me = path_line.find(" ").ok_or(StatusCode::BAD_REQUEST)?;
            let method = &path_line[..me];
            let path_line = &path_line[me + 1..];
            let route = &path_line[..path_line.find(" ").ok_or(StatusCode::BAD_REQUEST)?];
            let mut headers = FnvIndexMap::new();
            for header in lines {
                let Some((key, value)) = header.split_once(": ") else {
                    continue;
                };
                headers
                    .insert(key, value)
                    .map_err(|_| StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)?;
            }
            let body = &value[he + 4..];

            Ok(Request {
                route,
                method,
                headers: Headers(headers),
                body,
            })
        }
    }

    pub struct Response(Vec<u8, { RESPONSE_HEADER_LEN + RESPONSE_BODY_LEN }>);

    impl Response {
        pub fn new(data: Vec<u8, { RESPONSE_HEADER_LEN + RESPONSE_BODY_LEN }>) -> Self {
            Self(data)
        }
    }

    impl Deref for Response {
        type Target = Vec<u8, { RESPONSE_HEADER_LEN + RESPONSE_BODY_LEN }>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for Response {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[derive(Default)]
    pub struct ResponseBuilder<'h, T: Serialize> {
        pub headers: Headers<'h>,
        code: Option<StatusCode>,
        data: Option<&'h T>,
    }

    impl<'h, T: Serialize> ResponseBuilder<'h, T> {
        pub fn with_code(mut self, code: StatusCode) -> Self {
            self.code = Some(code);
            self
        }

        pub fn with_data(mut self, data: &'h T) -> Self {
            self.data = Some(data);
            self
        }
    }

    impl<T: Serialize> Into<Response> for ResponseBuilder<'_, T> {
        fn into(self) -> Response {
            let code = self.code.unwrap_or(StatusCode::OK);
            let mut headers = self.headers;
            let Some(data) = self.data else {
                return headers.into_response(code);
            };
            let mut ibuffer = itoa::Buffer::new();
            let mut buf = [0u8; RESPONSE_BODY_LEN];
            let pos = serde_json_core::to_slice(&data, &mut buf).unwrap();
            headers.insert("Content-Type", "application/json").unwrap();
            headers
                .insert("Content-Length", ibuffer.format(pos))
                .unwrap();

            let mut response = headers.into_response(code);
            response.extend_from_slice(&buf[..pos]).unwrap();
            response
        }
    }
}

pub mod json {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct Error<'e> {
        pub error: &'e str,
    }

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct PairData<'p> {
        pub id: &'p str,
    }
}

pub mod storage {
    use heapless::{String, Vec};
    use serde::{Deserialize, Serialize};
    use super::Sensor;

    #[derive(Serialize, Deserialize, Debug, Default)]
    pub struct Store {
        pub sensor: Sensor,
        pub paired_keys: Vec<String<64>, 16>,
    }
}
