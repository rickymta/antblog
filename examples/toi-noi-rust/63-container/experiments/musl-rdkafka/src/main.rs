//! Chỉ cần liên kết được librdkafka vào binary tĩnh và tạo được một `ClientConfig`.
fn main() {
    let mut cfg = rdkafka::config::ClientConfig::new();
    cfg.set("bootstrap.servers", "127.0.0.1:9092");
    println!("rdkafka linked: {}", rdkafka::util::get_rdkafka_version().1);
}
