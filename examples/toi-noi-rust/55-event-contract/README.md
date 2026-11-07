# Lab 55: hợp đồng sự kiện xuyên ngôn ngữ (protobuf, JSON Schema, version)
`LedgerEntryPosted` được định nghĩa hai lần: protobuf (`schemas/ledger_entry_posted.v1.proto`) và JSON Schema (`schemas/*.schema.json`).
Rust sinh fixture nhị phân và JSON (`fixtures/`, commit sẵn); consumer C#, Go, Python đọc cùng fixture. Sáu thay đổi schema (`schemas/experiments/`) được đo ở hai chiều và ba dạng đọc.
Lab không dựng schema registry: tương thích được kiểm bằng so descriptor (`src/compat.rs`) cạnh số đo thật.

```bash
cargo run  --manifest-path examples/toi-noi-rust/55-event-contract/Cargo.toml              # in bản mẫu ba dạng
cargo run  --manifest-path examples/toi-noi-rust/55-event-contract/Cargo.toml -- --verify  # không cần .NET/Go/Python, không cần hạ tầng
cargo test --manifest-path examples/toi-noi-rust/55-event-contract/Cargo.toml              # contract test với fixture commit sẵn
# cần .NET SDK 10, Go, Python có protobuf (venv, trỏ bằng TOI_NOI_PYTHON; requirements: consumers/python/requirements.txt):
cargo run  --manifest-path examples/toi-noi-rust/55-event-contract/Cargo.toml -- --with-consumers
```

`--with-consumers` thoát mã khác 0 nếu thiếu công cụ, consumer lỗi hoặc lệch `fixtures/expected_matrix.json` (`--accept` ghi lại kỳ vọng); `--write-fixtures` sinh lại fixture khi cố ý đổi hợp đồng.
`bin/` và `obj/` của .NET nằm trong `target/dotnet/` (đã gitignore). Số đo trên máy viết bài, không phải SLA.
