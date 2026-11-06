//! Mỗi kiểm chứng của `verify` là một test; chạy song song được vì không dùng tài nguyên dùng chung.

use ledger_33_tower_middleware::{capture, verify};

macro_rules! check_test {
    ($name:ident) => {
        #[tokio::test]
        async fn $name() {
            capture::init(); // trước mọi callsite của tracing
            if let Err(why) = verify::$name().await {
                panic!("{why}");
            }
        }
    };
}

check_test!(check_order);
check_test!(check_request_id);
check_test!(check_timeout);
check_test!(check_cors);
check_test!(check_jwt);
check_test!(check_authz);
check_test!(check_no_token_in_logs);
check_test!(check_concurrency);
check_test!(check_rate_limit);
check_test!(check_socket);
