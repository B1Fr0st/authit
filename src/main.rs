use poem::{Route, Server, get, listener::TcpListener};

pub mod types;
pub mod handlers;



#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().nest(
        "/api/v1",Route::new().nest(
            "/public", Route::new().at("/health",get(crate::handlers::public::health))
        ).nest(
            "/private", Route::new()
        )
    );
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}