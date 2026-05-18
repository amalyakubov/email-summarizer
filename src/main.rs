use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Json, Router};
use chrono::Local;
use chrono::TimeDelta;
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

impl AppState {
    fn new(client_id: String, redirect_uri: String, client_secret: String) -> Self {
        AppState {
            refresh_token: Arc::new(Mutex::new(None)),
            access_token: Arc::new(Mutex::new(None)),
            expires_in: Arc::new(Mutex::new(None)),
            refresh_token_expires_in: Arc::new(Mutex::new(None)),
            client_id: client_id,
            redirect_uri: redirect_uri,
            client_secret: client_secret,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Params {
    code: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AuthResponse {
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

    let shared_state = AppState::new(client_id, redirect_uri, client_secret);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/auth", get(auth_handler))
        .route("/auth/callback", get(auth_callback_handler))
        .route("/emails", get(get_list_of_user_emails))
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

#[derive(Deserialize)]
struct TokenRefreshResponse {
    access_token: String,
    expires_in: i32,
    scope: String,
    token_type: String,
}

async fn refresh_access_token(
    client_id: &String,
    client_secret: &String,
    refresh_token: &String,
) -> (String, i32) {
    let client = reqwest::Client::new();
    let url = format!(
        "https://oauth2.googleapis.com/token?client_id={client_id}&refresh_token={refresh_token}&grant_type=refresh_token"
    );

    let token_refresh_response_object = client
        .post(url)
        .send()
        .await
        .unwrap()
        .json::<TokenRefreshResponse>()
        .await
        .unwrap();

    return (
        token_refresh_response_object.access_token,
        token_refresh_response_object.expires_in,
    );
}

async fn auth_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (client_id, redirect_uri) = (state.client_id, state.redirect_uri);
    let uri = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=https://www.googleapis.com/auth/gmail.readonly&access_type=offline&prompt=consent"
    ); // lack of prompt=consent parameter made my life more difficult
    return Redirect::to(&uri).into_response();
}

async fn auth_callback_handler(params: Query<Params>, state: State<AppState>) -> impl IntoResponse {
    let (client_id, redirect_uri, client_secret) =
        (&state.client_id, &state.redirect_uri, &state.client_secret);

    match params.0.error {
        Some(_e) => {
            return  "Error authorizing the user. Authorization code not received from the OAuth ApiAPI"
                    .into_response()
        },
        None => (),
    }
    let access_token = params.0.code.unwrap();

    let client = reqwest::Client::new();

    let uri = format!(
        "https://oauth2.googleapis.com/token?client_id={client_id}&code={access_token}&grant_type=authorization_code&redirect_uri={redirect_uri}&client_secret={client_secret}"
    );

    let response = match client.post(&uri).send().await {
        Ok(response) => match response.text().await {
            Ok(response) => response,
            Err(_e) => return "Failed to convert oauth2 response into text".into_response(),
        },
        Err(e) => {
            return format!(
                "Failed to get the response code from oauth2 authorizatoin service, error: {}",
                e
            )
            .into_response();
        }
    };

    let data: AuthResponse = serde_json::from_str(&response).unwrap();

    //TODO: add beter error handling
    *state.access_token.lock().unwrap() = Some(data.access_token.clone());
    *state.expires_in.lock().unwrap() = Some(data.expires_in.clone());
    *state.refresh_token.lock().unwrap() = Some(data.refresh_token.clone());
    *state.refresh_token_expires_in.lock().unwrap() = Some(data.refresh_token_expires_in.clone());

    let access_token = state.access_token.lock().unwrap().clone().unwrap();
    println!("Access token: {access_token}");
    let refresh_token = state.refresh_token.lock().unwrap().clone().unwrap();
    println!("Refresh token: {refresh_token}");
    let expires_in = state.expires_in.lock().unwrap().clone().unwrap();
    println!("Expires inn: {expires_in}");

    return Json(json!(data)).into_response();
}

#[derive(Serialize, Deserialize)]
struct MessageStruct {
    id: String,
    #[serde(rename(serialize = "threadId", deserialize = "threadId"))]
    thread_id: String,
}

#[derive(Serialize, Deserialize)]
struct EmailsListResponse {
    messages: Vec<MessageStruct>,
    #[serde(rename(serialize = "resultSizeEstimate", deserialize = "resultSizeEstimate"))]
    result_size_estimate: i32,
}

async fn get_list_of_user_emails(state: State<AppState>) -> impl IntoResponse {
    let current_time = Local::now().date_naive();
    let yesterday = current_time - TimeDelta::days(1);
    let uri = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages?max_results=500&q=after:{}",
        yesterday
    );
    println!("{}", yesterday);
    let client = reqwest::Client::new();

    let access_token = state.access_token.lock().unwrap().clone().unwrap();

    let response = client
        .get(uri)
        .bearer_auth(access_token)
        .send()
        .await
        .unwrap();

    let emails_list_response = response.json::<EmailsListResponse>().await.unwrap();

    return Json(emails_list_response);
}
