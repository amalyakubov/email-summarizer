use std::fmt::format;

use axum::Router;
use axum::http::Request;
use axum::routing::get;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let client_id = dotenv::var("CLIENT_ID").expect("UNABLE TO fetch an env variable");
    let app = Router::new().route(
        "/",
        get(|| async {
            return get_token();
        }),
    );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to create a TCP listener");
    axum::serve(listener, app)
        .await
        .expect("Failed to serve the app");
}

// TODO: Finish implementing
pub async fn get_token() {
    let client_id = dotenv::var("CLIENT_ID").unwrap();
    let redirect_uri = dotenv::var("REDIRECT_URI").unwrap();
    let uri = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=https://www.googleapis.com/auth/gmail.addons.current.message.readonly"
    );
    let request = Request::get(uri).body(()).unwrap();
}

async fn get_list_of_emails(user_id: String) {}
