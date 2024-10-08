use crate::{database::DbConn, DbPool};
use auth::Token;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};
use db::{SensorEntity, SensorFeatures};
use deref_derive::Deref;
use reqwest::StatusCode;

#[allow(dead_code)]
pub struct RequestData {
    pub token: Option<Token>,
    pub user: Option<User>,
    pub is_hx_request: bool,
    pub conn: DbConn,
    pub headers: HeaderMap,
}

impl FromRequestParts<DbPool> for RequestData {
    type Rejection = StatusCode;

    #[doc = " Perform the extraction."]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        state: &'life1 DbPool,
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
            let token = Token::try_from(&parts.headers).ok();
            let conn = state
                .get()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let user = Token::get_valid_user(token.clone(), &conn)
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?;
            let is_hx_request = parts.headers.contains_key("Hx-Request");

            Ok(Self {
                token,
                user,
                is_hx_request,
                conn,
                headers: parts.headers.clone(),
            })
        })
    }
}

#[derive(Debug, Clone, Default, Deref)]
pub struct NormalizedString(String);

impl NormalizedString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into().to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Area {
    pub id: i64,
    pub name: String,
    pub sensors: Vec<SensorEntity>,
}

impl Area {
    pub fn features(&self) -> Vec<SensorFeatures> {
        let mut features = self
            .sensors
            .iter()
            .flat_map(|s| s.features.iter())
            .collect::<Vec<_>>();
        features.sort_by_key(|a| a.bits());
        features
    }
}

pub mod json {
    use super::db::SensorFeatures;
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
    pub struct SensorFormData {
        pub name: String,
        #[serde(rename = "area-id")]
        pub area_id: String,
    }

    impl From<SensorFormData> for SensorDto {
        fn from(val: SensorFormData) -> Self {
            SensorDto {
                name: Some(val.name),
            }
        }
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct SensorDto {
        pub name: Option<String>,
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct SensorResponse {
        pub name: String,
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

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct MeasurementsResponse {
        pub measurements: Vec<Measurement>,
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct Measurement {
        pub timestamp: u64,
        pub temperature: f32,
        pub humidity: f32,
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct ScheduleEntryFormData {
        #[serde(rename = "features-temp")]
        pub features_temp: Option<String>,
        #[serde(rename = "features-motion")]
        pub features_motion: Option<String>,
        pub interval: String,
    }

    impl TryInto<super::db::DataScheduleEntry> for ScheduleEntryFormData {
        type Error = anyhow::Error;

        fn try_into(self) -> Result<super::db::DataScheduleEntry, Self::Error> {
            let mut features = SensorFeatures::empty();
            if self.features_temp.is_some() {
                features |= SensorFeatures::TEMPERATURE;
            }
            if self.features_motion.is_some() {
                features |= SensorFeatures::MOTION;
            }
            let interval_ms = self.interval.splitn(3, ':').enumerate().try_fold(
                0u64,
                |mut interval, (i, s)| {
                    interval += s.parse::<u64>()? * 60u64.pow(2 - i as u32);
                    Result::<_, anyhow::Error>::Ok(interval)
                },
            )? * 1000;
            Ok(crate::models::db::DataScheduleEntry {
                features,
                interval_ms,
            })
        }
    }

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct AreaFormData {
        pub name: String,
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

            let Some(_session) = conn
                .get_session(normalized_name.clone(), token.clone())
                .await?
            else {
                return Ok(None);
            };

            if !claims.validate() {
                conn.delete_session(normalized_name, token).await?;
                return Ok(None);
            }

            Ok(Some(claims.into()))
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
        pub exp: i64,
        pub acs: i64,
    }

    const SUB_CLAIM: &str = "sub";
    const EXP_CLAIM: &str = "exp";
    const ACS_CLAIM: &str = "acs";

    impl Claims {
        pub fn validate(&self) -> bool {
            self.exp - chrono::Utc::now().timestamp() > 0
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
                sub: value.name,
                exp: chrono::Utc::now().timestamp() + 3600,
                acs: value.id,
            }
        }
    }

    impl From<Claims> for User {
        fn from(val: Claims) -> Self {
            User {
                id: val.acs,
                name: val.sub,
            }
        }
    }
}

pub mod db {
    use super::{
        auth::{Password, Token},
        json::SensorDto,
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
        pub id: i64,
        pub name: String,
        pub normalized_name: NormalizedString,
        pub password: Password,
    }

    impl FromRow for UserEntity {
        fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
            Ok(UserEntity {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                normalized_name: NormalizedString::new(row.get::<_, String>(2)?),
                password: Password::from_str(&row.get::<_, String>(3)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
            })
        }
    }

    impl From<UserEntity> for User {
        fn from(val: UserEntity) -> Self {
            User {
                id: val.id,
                name: val.name,
            }
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

    bitflags::bitflags! {
        #[derive(Debug, Default, Clone, Copy, PartialEq)]
        pub struct SensorFeatures: u32 {
            const TEMPERATURE = 1 << 0;
            const MOTION = 1 << 1;

            const _ = !0;
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct SensorEntity {
        pub name: String,
        pub area: Option<AreaEntity>,
        pub features: SensorFeatures,
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

        async fn check(
            &mut self,
            pool: &crate::database::DbPool,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let sensor = pool
                .get()
                .await
                .map_err(|e| e.to_string())?
                .get_sensor(&self.host)
                .await
                .map_err(|e| e.to_string())?;
            self.pair_id = sensor.as_ref().and_then(|s| s.pair_id.clone());
            self.area = sensor.as_ref().and_then(|s| s.area.clone());
            Ok(())
        }
    }

    impl FromRow for SensorEntity {
        fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
            Ok(SensorEntity {
                name: row.get::<_, String>(0)?,
                area: row
                    .get::<_, Option<i64>>(1)?
                    .and_then(|id| row.get::<_, String>(6).ok().map(|name| (id, name)))
                    .map(|(id, name)| AreaEntity { id, name }),
                features: SensorFeatures::from_bits_retain(row.get::<_, i64>(2)? as u32),
                host: row.get::<_, String>(3)?,
                pair_id: row.get::<_, Option<String>>(4)?,
            })
        }
    }

    impl From<SensorEntity> for SensorDto {
        fn from(val: SensorEntity) -> Self {
            SensorDto {
                name: Some(val.name),
            }
        }
    }

    pub type DataSchedule = Vec<DataScheduleEntry>;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DataScheduleEntry {
        pub features: SensorFeatures,
        pub interval_ms: u64,
    }

    impl FromRow for DataScheduleEntry {
        fn from_row(row: &r2d2_sqlite::rusqlite::Row) -> r2d2_sqlite::rusqlite::Result<Self> {
            Ok(DataScheduleEntry {
                features: SensorFeatures::from_bits_retain(row.get::<_, u64>(0)? as u32),
                interval_ms: row.get::<_, u64>(1)?,
            })
        }
    }

    #[derive(Debug, Clone)]
    pub struct TempDataEntry {
        pub host: String,
        pub timestamp: u64,
        pub temperature: f32,
        pub humidity: f32,
    }

    impl FromRow for TempDataEntry {
        fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
            Ok(TempDataEntry {
                host: row.get::<_, String>(0)?,
                timestamp: row.get::<_, u64>(1)?,
                temperature: row.get::<_, f64>(2)? as f32,
                humidity: row.get::<_, f64>(3)? as f32,
            })
        }
    }

    #[derive(Debug, Clone)]
    pub struct AreaEntity {
        pub id: i64,
        pub name: String,
    }

    impl FromRow for AreaEntity {
        fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
            Ok(AreaEntity {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
            })
        }
    }
}
