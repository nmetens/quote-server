use axum::{Router, routing::get};
use tokio::net::TcpListener;

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Set up the address:
    let localhost = "127.0.0.1";
    let port = "8000";
    let address = format!("{}:{}", localhost, port);

    // Set up the server:
    let listener = TcpListener::bind(address).await?;
 
    let app = Router::new().route("/", get(|| async {"Famous Quote"}));

    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    serve().await.expect("No famous quote found");
}
