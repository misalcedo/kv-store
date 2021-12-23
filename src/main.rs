use axum::response::Response;
use axum::{
    body::{self, Bytes},
    error_handling::HandleErrorLayer,
    extract::{ContentLengthLimit, Extension, Path},
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use redis::AsyncCommands;
use std::{borrow::Cow, net::SocketAddr, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    add_extension::AddExtensionLayer, auth::RequireAuthorizationLayer,
    compression::CompressionLayer, trace::TraceLayer,
};

#[derive(thiserror::Error, Debug)]
pub enum KeyValueError {
    #[error("Unable to find key '{0}'.")]
    NotFound(String),
    #[error("Unable to perform Redis operation: {0}.")]
    Redis(#[from] redis::RedisError),
}

impl IntoResponse for KeyValueError {
    fn into_response(self) -> Response {
        let status_code = match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = body::boxed(body::Full::from(self.to_string()));

        Response::builder().status(status_code).body(body).unwrap()
    }
}

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "k8s_server=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    // Build our application by composing routes
    let app = Router::new()
        .route(
            "/:key",
            // Add compression to `kv_get`
            get(kv_get.layer(CompressionLayer::new()))
                // But don't compress `kv_set`
                .post(kv_set),
        )
        .route("/keys", get(list_keys))
        // Nest our admin routes under `/admin`
        .nest("/admin", admin_routes())
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                // Handle errors from middleware
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .layer(AddExtensionLayer::new(SharedState::default()))
                .into_inner(),
        );

    // Run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

type SharedState = Arc<State>;

struct State {
    db: redis::Client,
}

impl Default for State {
    fn default() -> Self {
        let db = redis::Client::open("redis://localhost/").expect("Unable to connect to Redis");

        State { db }
    }
}

async fn kv_get(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<Bytes, KeyValueError> {
    let mut connection = state.db.get_async_connection().await?;
    let bytes: Option<Vec<u8>> = connection.get(key.as_str()).await?;

    if let Some(value) = bytes {
        Ok(value.into())
    } else {
        Err(KeyValueError::NotFound(key))
    }
}

async fn kv_set(
    Path(key): Path<String>,
    ContentLengthLimit(bytes): ContentLengthLimit<Bytes, { 1024 * 5_000 }>, // ~5mb
    Extension(state): Extension<SharedState>,
) -> Result<(), KeyValueError> {
    let mut connection = state.db.get_async_connection().await?;

    connection.set(key, &bytes[..]).await?;

    Ok(())
}

async fn list_keys(Extension(state): Extension<SharedState>) -> Result<String, KeyValueError> {
    let mut connection = state.db.get_async_connection().await?;
    let keys: Vec<String> = connection.keys("*").await?;

    Ok(keys.join("\n"))
}

fn admin_routes() -> Router {
    async fn delete_all_keys(
        Extension(state): Extension<SharedState>,
    ) -> Result<(), KeyValueError> {
        let mut connection = state.db.get_async_connection().await?;
        redis::cmd("FLUSHALL").query_async(&mut connection).await?;

        Ok(())
    }

    async fn remove_key(
        Path(key): Path<String>,
        Extension(state): Extension<SharedState>,
    ) -> Result<(), KeyValueError> {
        let mut connection = state.db.get_async_connection().await?;
        let result: usize = connection.del(key.as_str()).await?;

        if result > 0 {
            Ok(())
        } else {
            Err(KeyValueError::NotFound(key))
        }
    }

    Router::new()
        .route("/keys", delete(delete_all_keys))
        .route("/key/:key", delete(remove_key))
        // Require bearer auth for all admin routes
        .layer(RequireAuthorizationLayer::bearer("secret-token"))
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}
