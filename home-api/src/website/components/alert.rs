use askama::Template;
use reqwest::StatusCode;
use std::fmt::Display;

#[derive(Template, Default)]
#[template(path = "components/alert.html")]
pub struct AlertTemplate {
    pub alert_type: Option<AlertType>,
    pub alert_message: Option<String>,
}

#[allow(dead_code)]
#[derive(Default, Clone)]
pub enum AlertType {
    Success,
    #[default]
    Info,
    Warning,
    Error,
}

impl Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::Success => write!(f, "alert-success"),
            AlertType::Info => write!(f, "alert-info"),
            AlertType::Warning => write!(f, "alert-warning"),
            AlertType::Error => write!(f, "alert-error"),
        }
    }
}

impl From<StatusCode> for AlertType {
    fn from(value: StatusCode) -> Self {
        match value.as_u16() {
            200..=299 => Self::Success,
            400..=499 => Self::Warning,
            _ => Self::Error,
        }
    }
}
