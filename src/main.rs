use axum::Router;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Result};
use axum::routing::get;
use std::sync::Arc;

struct AppState {
    access_token: Option<String>,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState { access_token: None });
    dotenv::dotenv().ok();
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/auth/callback", get(auth_handler))
        .with_state(shared_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to create a TCP listener");
    axum::serve(listener, app)
        .await
        .expect("Failed to serve the app");
}

// Request token from Google
async fn index_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if state.access_token.is_none() {
        return Redirect::to("/auth/callback").into_response();
    }

    return "HEY".to_string().into_response();
}

async fn auth_handler() -> impl IntoResponse {
    let client_id = dotenv::var("CLIENT_ID").unwrap();
    let redirect_uri = dotenv::var("REDIRECT_URI").unwrap();
    let uri = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=https://www.googleapis.com/auth/gmail.addons.current.message.readonly&access_type=offline"
    );
    return Redirect::to(&uri).into_response();
}

async fn get_list_of_emails(user_id: String) {}
