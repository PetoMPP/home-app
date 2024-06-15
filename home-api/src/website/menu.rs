use crate::models::User;
use askama::Template;

#[derive(Template)]
#[template(path = "components/menu.html")]
pub struct MenuTemplate {
    pub current_user: Option<User>,
}
