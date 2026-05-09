use axum::Router;
use axum::routing::get;

use axum::response::Html;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let app = Router::new().route(
        "/",
        get(|| async {
            match get_token().await {
                Ok(response) => Html(response),
                Err(e) => Html("Error".to_string()),
            }
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
pub async fn get_token() -> reqwest::Result<String> {
    let client_id = dotenv::var("CLIENT_ID").unwrap();
    let redirect_uri = dotenv::var("REDIRECT_URI").unwrap();
    let uri = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=https://www.googleapis.com/auth/gmail.addons.current.message.readonly"
    );
    let request = reqwest::get(uri).await?.text().await?;
    return Ok(request);
}

async fn get_list_of_emails(user_id: String) {}
