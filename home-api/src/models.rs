use deref_derive::Deref;

#[derive(Debug, Clone, Default, Deref)]
pub struct NormalizedString(String);

impl NormalizedString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into().to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
}

pub mod json {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct ErrorResponse {
        pub error: String,
    }

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct PairResponse {
        pub id: String,
    }

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    pub struct Sensor {
        pub name: String,
        pub location: String,
        pub features: u32,
    }

    impl From<SensorResponse> for Sensor {
        fn from(sensor: SensorResponse) -> Self {
            Sensor {
                name: sensor.name,
                location: sensor.location,
                features: sensor.features,
            }
        }
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct SensorDto {
        pub name: Option<String>,
        pub location: Option<String>,
        pub features: Option<u32>,
    }

    impl From<Sensor> for SensorDto {
        fn from(val: Sensor) -> Self {
            SensorDto {
                name: Some(val.name),
                location: Some(val.location),
                features: Some(val.features),
            }
        }
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct SensorResponse {
        pub name: String,
        pub location: String,
        pub features: u32,
        pub pairing: bool,
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct SensorFullResponse {
        pub name: String,
        pub location: String,
        pub features: u32,
        pub pairing: bool,
        pub paired_keys: u32,
        pub usage: StoreUsage,
    }

    #[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
    pub struct StoreUsage {
        pub data_used: u32,
        pub data_total: u32,
        pub pair_used: u32,
        pub pair_total: u32,
    }
}

pub mod auth {
    use super::NormalizedString;
    use super::{db::UserEntity, User};
    use crate::database::user_sessions::UserSessionDatabase;
    use crate::database::DbConn;
    use axum::http::HeaderMap;
    use axum::{extract::FromRequestParts, http::request::Parts};
    use deref_derive::Deref;
    use hmac::Hmac;
    use jwt::{SignWithKey, VerifyWithKey};
    use reqwest::header::COOKIE;
    use reqwest::StatusCode;
    use sha2::{Digest, Sha256};
    use std::{collections::BTreeMap, fmt::Display, str::FromStr};

    #[derive(Debug, Clone)]
    pub struct Password {
        hash: String,
        salt: String,
    }

    impl Display for Password {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:{}", self.hash, self.salt)
        }
    }

    impl FromStr for Password {
        type Err = Box<dyn std::error::Error>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(s.split_once(':')
                .ok_or("Invalid password format")
                .map(|(h, s)| Self {
                    hash: h.to_string(),
                    salt: s.to_string(),
                })?)
        }
    }

    impl Password {
        pub fn new(password: String) -> Self {
            let mut rng = urandom::csprng();
            let salt = hex::encode(rng.next::<[u8; 16]>());
            let salty_password = password + &salt;
            let mut hasher = Sha256::new();
            hasher.update(&salty_password);
            let result = hasher.finalize();
            let hash = hex::encode(result);
            Self { hash, salt }
        }

        pub fn verify(&self, password: &str) -> bool {
            let salty_password = password.to_string() + &self.salt;
            let mut hasher = Sha256::new();
            hasher.update(&salty_password);
            let result = hasher.finalize();
            let hash = format!("{:x}", result);
            self.hash == hash
        }
    }

    #[derive(Debug, Deref, Clone)]
    pub struct Token(String);

    impl Token {
        pub fn new(user: &UserEntity) -> Result<Self, Box<dyn std::error::Error>> {
            use hmac::digest::KeyInit;
            let key: Hmac<Sha256> = Hmac::new_from_slice(env!("API_SECRET").as_bytes()).unwrap();
            let claims: BTreeMap<String, String> = Claims::from(user.clone()).into();
            Ok(Self(claims.sign_with_key(&key)?))
        }

        pub async fn get_valid_user(
            opt_self: Option<Self>,
            conn: &DbConn,
        ) -> Result<Option<User>, Box<dyn std::error::Error>> {
            let Some(token) = opt_self else {
                return Ok(None);
            };
            let Ok(claims) = TryInto::<Claims>::try_into(&token) else {
                return Ok(None);
            };
            let normalized_name = NormalizedString::new(&claims.sub);
            Ok(conn
                .get_session(normalized_name, token)
                .await?
                .map(|_| claims.into()))
        }
    }

    impl<S> FromRequestParts<S> for Token {
        type Rejection = StatusCode;

        #[doc = " Perform the extraction."]
        #[must_use]
        #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
        fn from_request_parts<'life0, 'life1, 'async_trait>(
            parts: &'life0 mut Parts,
            _state: &'life1 S,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Result<Self, Self::Rejection>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                Token::try_from(&parts.headers).map_err(|_| StatusCode::UNAUTHORIZED)
            })
        }
    }

    impl TryInto<Claims> for &Token {
        type Error = Box<dyn std::error::Error>;

        fn try_into(self) -> Result<Claims, Self::Error> {
            use hmac::digest::KeyInit;
            let key: Hmac<Sha256> = Hmac::new_from_slice(env!("API_SECRET").as_bytes()).unwrap();
            let token_data: BTreeMap<String, String> = self.verify_with_key(&key)?;

            Claims::try_from(token_data)
        }
    }

    impl TryFrom<&HeaderMap> for Token {
        type Error = Box<dyn std::error::Error>;

        fn try_from(value: &HeaderMap) -> Result<Self, Self::Error> {
            Ok(Self(
                value
                    .get(COOKIE)
                    .and_then(|cookie| {
                        cookie.to_str().ok().and_then(|cookie| {
                            cookie
                                .split(';')
                                .find(|cookie| cookie.starts_with("session="))
                                .map(|cookie| cookie.trim_start_matches("session="))
                        })
                    })
                    .ok_or("No session cookie")?
                    .to_string(),
            ))
        }
    }

    impl FromStr for Token {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self(s.to_string()))
        }
    }

    #[derive(Clone)]
    pub struct Claims {
        pub sub: String,
        pub exp: u64,
        pub acs: u64,
    }

    const SUB_CLAIM: &str = "sub";
    const EXP_CLAIM: &str = "exp";
    const ACS_CLAIM: &str = "acs";

    impl Claims {
        pub fn validate(&self) -> bool {
            (self.exp as i64) - chrono::Utc::now().timestamp() > 0
        }
    }

    impl From<Claims> for BTreeMap<String, String> {
        fn from(val: Claims) -> Self {
            let mut map = BTreeMap::new();
            map.insert(SUB_CLAIM.to_string(), val.sub.to_string());
            map.insert(EXP_CLAIM.to_string(), val.exp.to_string());
            map.insert(ACS_CLAIM.to_string(), val.acs.to_string());
            map
        }
    }

    impl TryFrom<BTreeMap<String, String>> for Claims {
        type Error = Box<dyn std::error::Error>;

        fn try_from(value: BTreeMap<String, String>) -> Result<Self, Self::Error> {
            let sub = value.get(SUB_CLAIM).ok_or("Missing sub claim")?.parse()?;
            let exp = value.get(EXP_CLAIM).ok_or("Missing exp claim")?.parse()?;
            let acs = value.get(ACS_CLAIM).ok_or("Missing acs claim")?.parse()?;
            Ok(Self { sub, exp, acs })
        }
    }

    impl From<UserEntity> for Claims {
        fn from(value: UserEntity) -> Self {
            Self {
                sub: value.name.parse().unwrap(),
                exp: chrono::Utc::now().timestamp() as u64 + 3600,
                acs: 0,
            }
        }
    }

    impl From<Claims> for User {
        fn from(val: Claims) -> Self {
            User { name: val.sub }
        }
    }
}

pub mod db {
    use super::{
        auth::{Password, Token},
        json::{Sensor, SensorDto},
        NormalizedString, User,
    };
    use crate::{
        database::{sensors::SensorDatabase, FromRow},
        services::{scanner_service::Scannable, sensor_service::SensorService},
    };
    use r2d2_sqlite::rusqlite;
    use std::str::FromStr;

    #[derive(Debug, Clone)]
    pub struct UserEntity {
        pub name: String,
        pub normalized_name: NormalizedString,
        pub password: Password,
    }

    impl FromRow for UserEntity {
        fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
            Ok(UserEntity {
                name: row.get::<_, String>(0)?,
                normalized_name: NormalizedString::new(row.get::<_, String>(1)?),
                password: Password::from_str(&row.get::<_, String>(2)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
            })
        }
    }

    impl From<UserEntity> for User {
        fn from(val: UserEntity) -> Self {
            User { name: val.name }
        }
    }

    #[derive(Debug, Clone)]
    pub struct UserSession {
        pub normalized_name: NormalizedString,
        pub token: Token,
    }

    impl FromRow for UserSession {
        fn from_row(row: &r2d2_sqlite::rusqlite::Row) -> r2d2_sqlite::rusqlite::Result<Self> {
            Ok(UserSession {
                normalized_name: NormalizedString::new(row.get::<_, String>(0)?),
                token: Token::from_str(&row.get::<_, String>(1)?).unwrap(),
            })
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct SensorEntity {
        pub name: String,
        pub location: String,
        pub features: u32,
        pub host: String,
        pub pair_id: Option<String>,
    }

    impl Scannable for SensorEntity {
        type Error = String;

        fn scan(
            client: &reqwest::Client,
            host: &str,
        ) -> impl std::future::Future<
            Output = Result<Result<Self, Self::Error>, Box<dyn std::error::Error + Send + Sync>>,
        > + Send {
            client.get_sensor(host)
        }

        fn check(
            &mut self,
            pool: &crate::database::DbPool,
        ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send
        {
            async move {
                self.pair_id = pool
                    .get()
                    .await
                    .map_err(|e| e.to_string())?
                    .get_sensor(&self.host)
                    .await
                    .map_err(|e| e.to_string())?
                    .and_then(|s| s.pair_id);
                Ok(())
            }
        }
    }

    impl FromRow for SensorEntity {
        fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
            Ok(SensorEntity {
                name: row.get::<_, String>(0)?,
                location: row.get::<_, String>(1)?,
                features: row.get(2)?,
                host: row.get::<_, String>(3)?,
                pair_id: row.get::<_, Option<String>>(4)?,
            })
        }
    }

    impl From<SensorEntity> for Sensor {
        fn from(val: SensorEntity) -> Self {
            Sensor {
                name: val.name,
                location: val.location,
                features: val.features,
            }
        }
    }

    impl From<SensorEntity> for SensorDto {
        fn from(val: SensorEntity) -> Self {
            SensorDto {
                name: Some(val.name),
                location: Some(val.location),
                features: Some(val.features),
            }
        }
    }
}
