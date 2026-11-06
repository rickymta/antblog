# Bài 05 · List, dict, set, tuple

Lab của bài `python-list-dict-set-tuple`: quản lý 6 chuyến (TN-0001…TN-0006), 3 tài xế (DR-001…DR-003) và 6 thành phố của Tới Nơi hoàn toàn trong bộ nhớ.

```bash
uv run --project examples/toi-noi-python/05-list-dict-set-tuple examples/toi-noi-python/05-list-dict-set-tuple/main.py
uv run --project examples/toi-noi-python/05-list-dict-set-tuple examples/toi-noi-python/05-list-dict-set-tuple/main.py --verify
uv run --project examples/toi-noi-python/05-list-dict-set-tuple examples/toi-noi-python/05-list-dict-set-tuple/main.py --bench
```

- Lệnh đầu in kết quả từng mục: list, tuple, dict, set, alias/copy, hashable, Counter/defaultdict/deque.
- `--verify` phải in `OK 17/17` và thoát với mã 0; sai thì in `FAIL n/m` (kèm tên ca sai) và thoát mã 1.
- `--bench` (tùy chọn) đo tìm phần tử cuối trong list và set 20.000 mã. Số đo thay đổi theo máy.

Không cần mạng, không có thư viện ngoài. Dữ liệu chỉ sống trong process: không bền vững, không transaction.
