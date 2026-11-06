//! Snapshot tài liệu OpenAPI: đổi hợp đồng phải là một quyết định thấy được trong diff.
//! Snapshot nằm ở `tests/snapshots/` và được commit.

use axum::http::{Method, StatusCode};
use ledger_api::api::OPENAPI_PATH;
use ledger_api::testkit::{offline_router, send};

#[tokio::test]
async fn openapi_document_matches_the_committed_snapshot() {
    // Router offline: tài liệu OpenAPI không cần database.
    let reply = send(&offline_router(), Method::GET, OPENAPI_PATH, None, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    insta::assert_json_snapshot!("openapi_document", reply.json());
}

#[tokio::test]
async fn every_error_status_of_post_entry_is_documented() {
    let reply = send(&offline_router(), Method::GET, OPENAPI_PATH, None, None).await;
    let doc = reply.json();
    let responses = doc["paths"]["/v1/entries"]["post"]["responses"]
        .as_object()
        .expect("có responses");
    for status in [
        "200", "201", "400", "401", "403", "409", "413", "415", "422",
    ] {
        assert!(responses.contains_key(status), "thiếu {status}");
    }
}
