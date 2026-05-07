use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde::Serialize;

#[derive(Serialize)]
struct ReadyResponse {
    ready: bool,
}

#[derive(Serialize)]
struct FraudScoreResponse {
    approved: bool,
    fraud_score: f32,
}

async fn ready() -> Json<ReadyResponse> {
    Json(ReadyResponse { ready: true })
}

async fn fraud_score(_: Json<serde_json::Value>) -> (StatusCode, Json<FraudScoreResponse>) {
    (
        StatusCode::OK,
        Json(FraudScoreResponse {
            approved: true,
            fraud_score: 0.0,
        }),
    )
}

fn router() -> Router {
    Router::new()
        .route("/ready", get(ready))
        .route("/fraud-score", post(fraud_score))
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999")
        .await
        .expect("bind 0.0.0.0:9999");
    axum::serve(listener, router()).await.expect("serve");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn ready_returns_ok_with_ready_true() {
        let resp = router()
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ready"], serde_json::Value::Bool(true));
    }

    #[tokio::test]
    async fn fraud_score_returns_ok_with_required_fields() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/fraud-score")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"id":"tx-1"}"#))
            .unwrap();
        let resp = router().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            v["approved"].as_bool().is_some(),
            "approved must be boolean"
        );
        assert!(
            v["fraud_score"].as_f64().is_some(),
            "fraud_score must be number"
        );
    }
}
