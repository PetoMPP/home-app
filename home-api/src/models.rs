use deref_derive::Deref;

#[derive(Debug, Clone, Default, Deref)]
pub struct NormalizedString(String);

impl NormalizedString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into().to_lowercase())
    }
}

pub mod auth {
    use super::db::UserEntity;
    use deref_derive::Deref;
    use hmac::{digest::KeyInit, Hmac};
    use jwt::{SignWithKey, VerifyWithKey};
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
            let salt: [u8; 16] = rng.next();
            let salt = salt.iter().map(|x| format!("{:x}", x)).collect::<String>();
            let salty_password = password + &salt;
            let mut hasher = Sha256::new();
            hasher.update(&salty_password);
            let result = hasher.finalize();
            let hash = format!("{:x}", result);
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
        pub fn new(user: &UserEntity) -> Result<String, Box<dyn std::error::Error>> {
            let key: Hmac<Sha256> = Hmac::new_from_slice(env!("API_SECRET").as_bytes()).unwrap();
            let claims: BTreeMap<String, String> = Claims::try_from(user.clone())?.into();
            Ok(claims.sign_with_key(&key)?)
        }

        pub fn validate_token(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
            let key: Hmac<Sha256> = Hmac::new_from_slice(env!("API_SECRET").as_bytes()).unwrap();
            let token_data: BTreeMap<String, String> = token.verify_with_key(&key)?;

            Claims::try_from(token_data)
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
            match (exp as i64) - chrono::Utc::now().timestamp() {
                ref x if x < &0 => Result::Err("Token expired")?,
                _ => Ok(Self { sub, exp, acs }),
            }
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
}

pub mod db {
    use super::{auth::Password, NormalizedString};
    use crate::database::FromRow;
    use home_common::models::Sensor;
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

    #[derive(Debug, Clone)]
    pub struct SensorEntity {
        pub name: String,
        pub location: String,
        pub features: u32,
        pub pair_id: String,
    }

    impl FromRow for SensorEntity {
        fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
            Ok(SensorEntity {
                name: row.get::<_, String>(0)?,
                location: row.get::<_, String>(1)?,
                features: row.get(2)?,
                pair_id: row.get::<_, String>(3)?,
            })
        }
    }

    impl Into<Sensor> for SensorEntity {
        fn into(self) -> Sensor {
            Sensor {
                name: heapless::String::from_str(self.name.as_str()).unwrap(),
                location: heapless::String::from_str(self.location.as_str()).unwrap(),
                features: self.features,
            }
        }
    }
}
