use crate::SqlitePool;
use home::Home;
use tower_http::services::ServeFile;
use yew::prelude::*;
use yew::ServerRenderer;

mod home;

#[derive(Clone, Copy, PartialEq)]
pub enum Route {
    Home,
    NotFound,
}

pub trait WebappService {
    fn register_webapp(self) -> Self;
}

impl WebappService for axum::Router {
    fn register_webapp(self) -> Self {
        self.route("/", axum::routing::get(home))
            .fallback(not_found)
            .nest_service("/output.css", ServeFile::new("output.css"))
    }
}

impl Route {
    pub fn get_html(&self, props: &AppProps) -> Html {
        match &self {
            Route::Home => html! { <Home sensors={props.pool.get_sensors().unwrap()} /> },
            Route::NotFound => html! { <h1 class={"text-error text-2xl"}>{"404 Not Found"}</h1> },
        }
    }

    pub async fn render(&self, pool: SqlitePool) -> axum::response::Html<String> {
        let route = *self;
        let renderer = ServerRenderer::<App>::with_props(move || AppProps { pool, route });
        axum::response::Html(renderer.render().await)
    }
}

pub async fn home(
    axum::Extension(pool): axum::Extension<SqlitePool>,
) -> axum::response::Html<String> {
    Route::Home.render(pool).await
}

pub async fn not_found(
    axum::Extension(pool): axum::Extension<SqlitePool>,
) -> axum::response::Html<String> {
    Route::NotFound.render(pool).await
}

#[derive(Properties, Clone)]
pub struct AppProps {
    pub pool: SqlitePool,
    pub route: Route,
}

impl PartialEq for AppProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[function_component(App)]
pub fn app(props: &AppProps) -> Html {
    html! {
        <html>
            <head>
            <meta charset="UTF-8" />
            <meta http-equiv="X-UA-Compatible" content="IE=edge" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Montserrat" />
            <link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Roboto+Mono" />
            <link rel="stylesheet" href="/output.css" />
            </head>

            <h1 class={"sticky top-0 z-50 w-full bg-base-200 opacity-90 p-2 pb-4"}>
                <a href={"/"} class={"text-4xl font-bold btn-ghost"}>{"Home API"}</a>
            </h1>
            <div class={"w-full h-full flex justify-center py-6"}>
                { props.route.get_html(props) }
            </div>
        </html>
    }
}
