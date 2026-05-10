use axum::Router;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use reqwest::StatusCode;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
struct AppState {
    refresh_token: Arc<Mutex<Option<String>>>,
    access_token: Arc<Mutex<Option<String>>>,
    client_id: String,
    redirect_uri: String,
    client_secret: String,
}

#[derive(Debug, Deserialize)]
struct Params {
    code: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let client_id = dotenv::var("CLIENT_ID").expect("Unable to get client id env variable");
    let redirect_uri =
        dotenv::var("REDIRECT_URI").expect("Unable to get redirect uri env variable");
    let client_secret =
        dotenv::var("CLIENT_SECRET").expect("Unable to get client secret env variable");

    let shared_state = AppState {
        client_id: client_id,
        redirect_uri: redirect_uri,
        refresh_token: Arc::new(Mutex::new(None)),
        access_token: Arc::new(Mutex::new(None)),
        client_secret: client_secret,
    };
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/auth", get(auth_handler))
        .route("/auth/callback", get(auth_callback_handler))
        .with_state(shared_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to create a TCP listener");
    axum::serve(listener, app)
        .await
        .expect("Failed to serve the app");
}

// Request token from Google
async fn index_handler(State(state): State<AppState>) -> impl IntoResponse {
    if state.access_token.lock().unwrap().is_none() {
        return Redirect::to("/auth").into_response();
    }

    return StatusCode::OK.into_response();
}

async fn auth_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (client_id, redirect_uri) = (state.client_id, state.redirect_uri);
    let uri = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=https://www.googleapis.com/auth/gmail.addons.current.message.readonly&access_type=offline"
    );
    return Redirect::to(&uri).into_response();
}

async fn auth_callback_handler(params: Query<Params>, state: State<AppState>) -> impl IntoResponse {
    let (client_id, redirect_uri, client_secret) =
        (&state.client_id, &state.redirect_uri, &state.client_secret);
    let access_token = params.0.code;

    let client = reqwest::Client::new();

    let uri = format!(
        "https://oauth2.googleapis.com/token?client_id={client_id}&code={access_token}&grant_type=authorization_code&redirect_uri={redirect_uri}&client_secret={client_secret}"
    );

    match client.post(uri).send().await {
        Ok(response) => response.text().await.unwrap().into_response(),
        Err(e) => e.to_string().into_response(),
    }
}

async fn get_list_of_emails(user_id: String) {}
