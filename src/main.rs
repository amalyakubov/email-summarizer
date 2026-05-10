use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Json, Router};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
struct AppState {
    refresh_token: Arc<Mutex<Option<String>>>,
    access_token: Arc<Mutex<Option<String>>>,
    expires_in: Arc<Mutex<Option<i32>>>,
    refresh_token_expires_in: Arc<Mutex<Option<i32>>>,
    client_id: String,
    redirect_uri: String,
    client_secret: String,
}

#[derive(Debug, Deserialize)]
struct Params {
    code: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseData {
    access_token: String,
    expires_in: i32,
    refresh_token: String,
    scope: String,
    token_type: String,
    refresh_token_expires_in: i32,
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
        expires_in: Arc::new(Mutex::new(None)),
        refresh_token_expires_in: Arc::new(Mutex::new(None)),
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
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=https://www.googleapis.com/auth/gmail.addons.current.message.readonly&access_type=offline&prompt=consent"
    ); // lack of prompt=consent parameter made my life more difficult
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

    let response = match client.post(&uri).send().await {
        Ok(response) => match response.text().await {
            Ok(response) => response,
            Err(e) => return e.to_string().into_response(),
        },
        Err(e) => return e.to_string().into_response(),
    };

    let data: ResponseData = serde_json::from_str(&response).unwrap();

    //TODO: add beter error handling
    *state.access_token.lock().unwrap() = Some(data.access_token.clone());
    *state.expires_in.lock().unwrap() = Some(data.expires_in.clone());
    *state.refresh_token.lock().unwrap() = Some(data.refresh_token.clone());
    *state.refresh_token_expires_in.lock().unwrap() = Some(data.refresh_token_expires_in.clone());

    return Json(json!(data)).into_response();
}

async fn get_list_of_emails(user_id: String) {}
