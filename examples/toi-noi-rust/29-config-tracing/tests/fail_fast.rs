//! Chạy binary thật với cấu hình sai: phải thoát mã 2, in mọi vấn đề ra stderr, không log gì ra stdout.

use std::process::Command;

fn run(envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ledger_29_config_tracing"));
    command.env_clear();
    for (k, v) in envs {
        command.env(k, v);
    }
    command.output().expect("chạy được binary")
}

#[test]
fn bad_config_exits_with_code_2_and_lists_all_problems() {
    let output = run(&[
        ("LEDGER__DATABASE__MAX_CONNECTIONS", "0"),
        ("LEDGER__SERVER__PORT", "0"),
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        output.stdout.is_empty(),
        "chưa được log gì trước khi cấu hình hợp lệ"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("3 vấn đề"), "{stderr}");
    assert!(stderr.contains("server.port"));
    assert!(stderr.contains("max_connections"));
    assert!(stderr.contains("database.password"));
}

#[test]
fn good_config_starts_and_prints_json_lines_without_the_password() {
    let output = run(&[
        ("LEDGER__DATABASE__PASSWORD", "lab-only-secret"),
        ("LEDGER__DATABASE__MAX_CONNECTIONS", "20"),
    ]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ledger khởi động"));
    assert!(stdout.contains("max_connections: 20"));
    assert!(!stdout.contains("lab-only-secret"));
    assert!(
        stdout
            .lines()
            .all(|l| l.starts_with('{') && l.ends_with('}'))
    );
}
