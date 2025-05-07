use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::application::service::RedTokenService;
use crate::core::token::Honeytoken;

// API response types
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// Request types
#[derive(Debug, Deserialize)]
struct TokenQuery {
    token: String,
}

#[derive(Debug, Deserialize)]
struct CreateTokenRequest {
    file_path: String,
    value: Option<String>,
    file_type: Option<String>,
}

// State to hold the application service
struct AppState {
    service: Arc<RedTokenService>,
}

// Routes
pub async fn start_server(service: Arc<RedTokenService>, port: u16) -> anyhow::Result<()> {
    let app_state = Arc::new(AppState { service });

    let app = Router::new()
        .route("/api/tokens", get(list_tokens).post(create_token))
        .route("/api/tokens/:id", get(get_token).delete(delete_token))
        .route("/api/check", get(check_token))
        .route("/health", get(health_check))
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", port);
    info!("Starting server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Handler implementations
#[axum::debug_handler]
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

#[axum::debug_handler]
async fn list_tokens(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.service.list_tokens().await {
        Ok(tokens) => {
            let response = ApiResponse {
                success: true,
                data: Some(tokens),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            let response = ApiResponse::<Vec<Honeytoken>> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

#[axum::debug_handler]
async fn get_token(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Uuid::parse_str(&id) {
        Ok(uuid) => {
            // This would need to be implemented in the service
            let response = ApiResponse::<Honeytoken> {
                success: false,
                data: None,
                error: Some("Not implemented".to_string()),
            };
            (StatusCode::NOT_IMPLEMENTED, Json(response))
        }
        Err(_) => {
            let response = ApiResponse::<Honeytoken> {
                success: false,
                data: None,
                error: Some("Invalid UUID format".to_string()),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

#[axum::debug_handler]
async fn create_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    let result = state
        .service
        .inject_token(
            &request.file_path,
            request
                .value
                .unwrap_or_else(|| format!("fake_token_{}", Uuid::new_v4())),
        )
        .await;

    match result {
        Ok(token) => {
            let response = ApiResponse {
                success: true,
                data: Some(token),
                error: None,
            };
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            let response = ApiResponse::<Honeytoken> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

#[axum::debug_handler]
async fn delete_token(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match Uuid::parse_str(&id) {
        Ok(uuid) => match state.service.remove_token(uuid).await {
            Ok(_) => {
                let response = ApiResponse::<()> {
                    success: true,
                    data: None,
                    error: None,
                };
                (StatusCode::OK, Json(response))
            }
            Err(e) => {
                let response = ApiResponse::<()> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
            }
        },
        Err(_) => {
            let response = ApiResponse::<()> {
                success: false,
                data: None,
                error: Some("Invalid UUID format".to_string()),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

#[axum::debug_handler]
async fn check_token(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TokenQuery>,
) -> impl IntoResponse {
    info!("Token check request received");

    match state.service.check_token(&params.token).await {
        Ok(_) => {
            // Always return OK to not reveal if token was valid
            let response = ApiResponse::<()> {
                success: true,
                data: None,
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Error checking token: {}", e);
            // Still return OK to not reveal if token was valid
            let response = ApiResponse::<()> {
                success: true,
                data: None,
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
    }
}
