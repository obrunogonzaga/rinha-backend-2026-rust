mod healthcheck;
mod index;
mod vector;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::Serialize;

use index::Index;
use vector::{Payload, vectorize};

#[derive(Clone)]
struct AppState {
    index: Arc<Index>,
}

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

async fn fraud_score(
    State(state): State<AppState>,
    Json(payload): Json<Payload>,
) -> (StatusCode, Json<FraudScoreResponse>) {
    let q = vectorize(&payload);
    let d = state.index.score(&q);
    (
        StatusCode::OK,
        Json(FraudScoreResponse {
            approved: d.approved,
            fraud_score: d.fraud_score,
        }),
    )
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/ready", get(ready))
        .route("/fraud-score", post(fraud_score))
        .with_state(state)
}

fn main() -> ExitCode {
    if std::env::args().any(|a| a == "--healthcheck") {
        return healthcheck::run();
    }
    serve();
    ExitCode::SUCCESS
}

fn serve() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");
    runtime.block_on(async {
        let data_dir: PathBuf = std::env::var("REFS_DATA_DIR")
            .unwrap_or_else(|_| "data".to_string())
            .into();
        let index = Index::load(&data_dir).expect("load index");
        eprintln!(
            "index loaded: count={} dir={}",
            index.count(),
            data_dir.display()
        );
        let state = AppState {
            index: Arc::new(index),
        };

        let listener = tokio::net::TcpListener::bind("0.0.0.0:9999")
            .await
            .expect("bind 0.0.0.0:9999");
        axum::serve(listener, router(state)).await.expect("serve");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Method, Request};
    use index::{DIMS, LABEL_FRAUD, LABEL_LEGIT};
    use tower::ServiceExt;

    fn test_state(labels: Vec<u8>) -> AppState {
        let refs = vec![0i16; labels.len() * DIMS].into_boxed_slice();
        let index = Index::from_parts(refs, labels.into_boxed_slice()).unwrap();
        AppState {
            index: Arc::new(index),
        }
    }

    #[tokio::test]
    async fn ready_returns_ok_with_ready_true() {
        let state = test_state(vec![LABEL_LEGIT; 5]);
        let resp = router(state)
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

    const VALID_PAYLOAD: &str = r#"{
        "id": "tx-1329056812",
        "transaction": { "amount": 41.12, "installments": 2, "requested_at": "2026-03-11T18:45:53Z" },
        "customer": { "avg_amount": 82.24, "tx_count_24h": 3, "known_merchants": ["MERC-016"] },
        "merchant": { "id": "MERC-016", "mcc": "5411", "avg_amount": 60.25 },
        "terminal": { "is_online": false, "card_present": true, "km_from_home": 29.23 },
        "last_transaction": null
    }"#;

    async fn post_fraud_score(state: AppState) -> serde_json::Value {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/fraud-score")
            .header("content-type", "application/json")
            .body(Body::from(VALID_PAYLOAD))
            .unwrap();
        let resp = router(state).oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), 1024).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn fraud_score_all_legit_neighbors_approves() {
        let state = test_state(vec![LABEL_LEGIT; 5]);
        let v = post_fraud_score(state).await;
        assert_eq!(v["approved"], serde_json::Value::Bool(true));
        assert_eq!(v["fraud_score"].as_f64().unwrap(), 0.0);
    }

    #[tokio::test]
    async fn fraud_score_all_fraud_neighbors_rejects() {
        let state = test_state(vec![LABEL_FRAUD; 5]);
        let v = post_fraud_score(state).await;
        assert_eq!(v["approved"], serde_json::Value::Bool(false));
        assert_eq!(v["fraud_score"].as_f64().unwrap(), 1.0);
    }

    #[tokio::test]
    async fn fraud_score_threshold_is_strictly_less_than_0_6() {
        // 3 fraud / 5 = 0.6 → not approved (strict <).
        let state = test_state(vec![
            LABEL_FRAUD,
            LABEL_FRAUD,
            LABEL_FRAUD,
            LABEL_LEGIT,
            LABEL_LEGIT,
        ]);
        let v = post_fraud_score(state).await;
        assert_eq!(v["approved"], serde_json::Value::Bool(false));
        assert!((v["fraud_score"].as_f64().unwrap() - 0.6).abs() < 1e-6);
    }
}
