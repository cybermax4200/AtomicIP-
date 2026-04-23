use axum::{routing::get, routing::post, Router};
use axum::middleware;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod auth;
mod handlers;
mod schemas;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Atomic Patent API",
        version = "1.0.0",
        description = "Machine-readable specification for the Atomic Patent Soroban smart contract interface."
    ),
    paths(
        handlers::commit_ip,
        handlers::get_ip,
        handlers::transfer_ip,
        handlers::verify_commitment,
        handlers::list_ip_by_owner,
        handlers::initiate_swap,
        handlers::accept_swap,
        handlers::reveal_key,
        handlers::cancel_swap,
        handlers::cancel_expired_swap,
        handlers::get_swap,
        handlers::login,
        handlers::refresh_token,
    ),
    components(schemas(
        schemas::CommitIpRequest,
        schemas::IpRecord,
        schemas::TransferIpRequest,
        schemas::VerifyCommitmentRequest,
        schemas::VerifyCommitmentResponse,
        schemas::ListIpByOwnerResponse,
        schemas::InitiateSwapRequest,
        schemas::AcceptSwapRequest,
        schemas::RevealKeyRequest,
        schemas::CancelSwapRequest,
        schemas::CancelExpiredSwapRequest,
        schemas::SwapRecord,
        schemas::SwapStatus,
        schemas::ErrorResponse,
        schemas::LoginRequest,
        schemas::AuthResponse,
        schemas::RefreshRequest,
    )),
    tags(
        (name = "IP Registry", description = "Commit and query intellectual property records"),
        (name = "Atomic Swap", description = "Trustless patent sale via atomic swap"),
        (name = "Auth", description = "Authentication and authorization"),
    )
)]
pub struct ApiDoc;

#[tokio::main]
async fn main() {
    // Protected business routes
    let protected = Router::new()
        .route("/ip/commit", post(handlers::commit_ip))
        .route("/ip/{ip_id}", get(handlers::get_ip))
        .route("/ip/transfer", post(handlers::transfer_ip))
        .route("/ip/verify", post(handlers::verify_commitment))
        .route("/ip/owner/{owner}", get(handlers::list_ip_by_owner))
        .route("/swap/initiate", post(handlers::initiate_swap))
        .route("/swap/{swap_id}/accept", post(handlers::accept_swap))
        .route("/swap/{swap_id}/reveal", post(handlers::reveal_key))
        .route("/swap/{swap_id}/cancel", post(handlers::cancel_swap))
        .route("/swap/{swap_id}/cancel-expired", post(handlers::cancel_expired_swap))
        .route("/swap/{swap_id}", get(handlers::get_swap))
        .layer(middleware::from_fn(auth::require_auth));

    // Public routes (auth + docs)
    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .route("/auth/login", post(handlers::login))
        .route("/auth/refresh", post(handlers::refresh_token))
        .merge(protected);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Swagger UI   -> http://localhost:8080/docs");
    println!("OpenAPI JSON -> http://localhost:8080/openapi.json");
    println!("Auth Login   -> http://localhost:8080/auth/login");
    axum::serve(listener, app).await.unwrap();
}
