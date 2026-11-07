//! Test kiểm chiều phụ thuộc bằng chính đồ thị mà Cargo giải ra (`cargo metadata`).
//!
//! Đây là bản Rust của "architecture test" trong loạt C#: không đọc mã nguồn, chỉ đọc
//! manifest đã được Cargo giải quyết. Thêm `sqlx` vào `crates/domain/Cargo.toml` là test đỏ.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use serde_json::Value;

const MEMBERS: [&str; 4] = ["domain", "application", "infrastructure", "api"];
// Crate chạy IO hoặc kéo runtime: không được nằm trong đồ thị của lõi.
const FORBIDDEN_IN_CORE: [&str; 9] = [
    "tokio", "sqlx", "rdkafka", "axum", "hyper", "tonic", "tower", "mio", "reqwest",
];

struct Graph {
    // tên package -> tên các package nó phụ thuộc (mọi loại cạnh: normal, build, dev)
    edges: BTreeMap<String, BTreeSet<String>>,
}

impl Graph {
    fn load(extra_args: &[&str]) -> Graph {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml");
        let out = Command::new(cargo)
            .args([
                "metadata",
                "--format-version",
                "1",
                "--offline",
                "--manifest-path",
                manifest,
            ])
            .args(extra_args)
            .output()
            .expect("không chạy được cargo metadata");
        assert!(
            out.status.success(),
            "cargo metadata lỗi: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let json: Value = serde_json::from_slice(&out.stdout).expect("metadata không phải JSON");

        let names: BTreeMap<&str, &str> = json["packages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p["id"].as_str().unwrap(), p["name"].as_str().unwrap()))
            .collect();
        let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for node in json["resolve"]["nodes"].as_array().unwrap() {
            let from = names[node["id"].as_str().unwrap()].to_string();
            let tos = node["deps"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| names[d["pkg"].as_str().unwrap()].to_string());
            edges.entry(from).or_default().extend(tos);
        }
        Graph { edges }
    }

    // Mọi package với tới được từ `root` (không tính `root`).
    fn closure(&self, root: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![root.to_string()];
        while let Some(name) = stack.pop() {
            for dep in self.edges.get(&name).into_iter().flatten() {
                if seen.insert(dep.clone()) {
                    stack.push(dep.clone());
                }
            }
        }
        seen
    }

    // Chỉ các cạnh giữa bốn crate của lab.
    fn workspace_deps(&self, name: &str) -> BTreeSet<String> {
        self.edges[name]
            .iter()
            .filter(|d| MEMBERS.contains(&d.as_str()))
            .cloned()
            .collect()
    }
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn dependency_direction_is_exactly_as_designed() {
    let g = Graph::load(&[]);
    assert_eq!(g.workspace_deps("domain"), set(&[]));
    assert_eq!(g.workspace_deps("application"), set(&["domain"]));
    assert_eq!(
        g.workspace_deps("infrastructure"),
        set(&["application", "domain"])
    );
    assert_eq!(
        g.workspace_deps("api"),
        set(&["application", "domain", "infrastructure"])
    );
}

#[test]
fn core_crates_do_not_reach_runtime_or_io_crates() {
    let g = Graph::load(&[]);
    for core in ["domain", "application"] {
        let reach = g.closure(core);
        let leaked: Vec<&str> = FORBIDDEN_IN_CORE
            .iter()
            .copied()
            .filter(|f| reach.contains(*f))
            .collect();
        assert!(
            leaked.is_empty(),
            "`{core}` kéo theo {leaked:?}; dependency trực tiếp: {:?}",
            g.edges[core]
        );
    }
    // Đối chứng: bộ quét không mù. `infrastructure` phải thấy sqlx và tokio.
    let infra = g.closure("infrastructure");
    assert!(infra.contains("sqlx") && infra.contains("tokio"));
}

#[test]
fn kafka_feature_decides_whether_rdkafka_is_in_the_build() {
    let with = Graph::load(&[]);
    assert!(
        with.closure("api").contains("rdkafka"),
        "mặc định phải có rdkafka"
    );

    let without = Graph::load(&["--no-default-features"]);
    assert!(
        !without.closure("api").contains("rdkafka"),
        "--no-default-features còn rdkafka trong api"
    );
    assert!(!without.closure("infrastructure").contains("rdkafka"));
    // Tắt feature Kafka không làm mất sqlx: chỉ adapter Kafka biến mất.
    assert!(without.closure("api").contains("sqlx"));
}
