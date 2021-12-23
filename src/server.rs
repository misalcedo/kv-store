use crate::errors::KeyValueError;
use axum::{
    body::{Body, Bytes},
    error_handling::HandleErrorLayer,
    extract::{ContentLengthLimit, Extension, Path},
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use redis::{AsyncCommands, Client};
use std::{borrow::Cow, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    add_extension::AddExtensionLayer, auth::RequireAuthorizationLayer,
    compression::CompressionLayer, trace::TraceLayer,
};

pub fn build_server(client: Client) -> Router<Body> {
    // Build our application by composing routes
    Router::new()
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
                .layer(AddExtensionLayer::new(client))
                .into_inner(),
        )
}

async fn kv_get(
    Path(key): Path<String>,
    Extension(state): Extension<Client>,
) -> Result<Bytes, KeyValueError> {
    let mut connection = state.get_async_connection().await?;
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
    Extension(state): Extension<Client>,
) -> Result<(), KeyValueError> {
    let mut connection = state.get_async_connection().await?;

    connection.set(key, &bytes[..]).await?;

    Ok(())
}

async fn list_keys(Extension(state): Extension<Client>) -> Result<String, KeyValueError> {
    let mut connection = state.get_async_connection().await?;
    let keys: Vec<String> = connection.keys("*").await?;

    Ok(keys.join("\n"))
}

fn admin_routes() -> Router {
    async fn delete_all_keys(Extension(state): Extension<Client>) -> Result<(), KeyValueError> {
        let mut connection = state.get_async_connection().await?;
        redis::cmd("FLUSHALL").query_async(&mut connection).await?;

        Ok(())
    }

    async fn remove_key(
        Path(key): Path<String>,
        Extension(state): Extension<Client>,
    ) -> Result<(), KeyValueError> {
        let mut connection = state.get_async_connection().await?;
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
