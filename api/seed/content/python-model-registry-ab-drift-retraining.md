---
slug: python-model-registry-ab-drift-retraining
title: "Model registry, A/B, drift và huấn luyện lại"
summary: "Bài 60: đăng ký phiên bản mô hình vào registry dạng thư mục, chạy shadow mode và A/B theo hash khách, đo drift bằng PSI hai phía, đặt lịch huấn luyện lại và rollback bằng con trỏ. Số đo thật, lặp lại trên ba seed."
category: python-advanced
level: advanced
tags: [python, toi-noi, fraud, mlops, drift]
language: vi
---

Bộ chấm điểm gian lận v1 của [Tới Nơi](/articles/du-an-mau-toi-noi) chạy được vài tuần thì đội dữ liệu đưa ra hai bản huấn luyện lại: v2 học thêm 10 ngày dữ liệu, v3 «học trên dữ liệu mới nhất». Cả hai đều có PR-AUC không thấp hơn v1. Câu hỏi vận hành: ai đổi phiên bản đang phục vụ, thử bản mới thế nào mà không để nó quyết định, khi nào đáng huấn luyện lại, và nếu hỏng thì lui mất bao lâu.

Đây là bài 60 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần quen cách lưu mô hình kèm metadata ở [bài 57](/articles/python-scikit-learn-lightgbm-imbalanced-cross-validation), dùng lại cho cả ba phiên bản, và đường phục vụ ở [bài 59](/articles/python-real-time-scoring-feature-store-redis-latency). Dữ liệu là tổng hợp; điểm rủi ro là đầu vào cho người duyệt. Bài 59 đến 61 dùng một quy tắc gắn cờ: `p × (tiền chuyến + 20.000) > 8.000`. Hai kịch bản do lab đặt vào dữ liệu luôn được gọi là «kịch bản».

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>v3 huấn luyện lại «trên dữ liệu mới nhất» (đến ngày 50, nhãn biết được vào ngày 50). Trên ngày 50 đến 60, PR-AUC của v3 là 0,889, cao hơn v1 (0,886). Bạn có promote v3 không, và nhìn con số nào để thấy v3 hỏng mà chưa cần chờ nhãn?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> PR-AUC chỉ đo thứ tự, còn v3 làm lệch thang xác suất: p trung bình 0,0056 trong khi tỉ lệ gian lận thật là 0,0148, p99 chỉ 0,099, số cờ giảm từ 16,4 xuống 8,9 trên 1.000 chuyến và chi phí giả định tăng từ 8,02 lên 11,46 triệu VND. Tỉ lệ cờ và p99 đo được ngay, không cần nhãn.</p></details>
</div>

## Registry: thư mục, manifest và con trỏ

Registry ở đây là một thư mục có quy ước. Mỗi phiên bản là một thư mục bất biến `out/registry/<model_version>/` gồm `model.joblib` và `model_meta.json` do `save_model` của bài 57 ghi, cộng `manifest.json`: ai đăng ký, bản cha, sha256, số đo holdout. Chỉ hai thứ đổi theo thời gian: `current.json` (bản đang phục vụ) và `history.jsonl` (sự kiện chỉ ghi thêm).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Thư mục out/registry gồm ba thư mục phiên bản bất biến, mỗi thư mục có model.joblib, model_meta.json và manifest.json. Bên phải là current.json trỏ tới phiên bản v2 và history.jsonl ghi các sự kiện registered, promoted, rolled_back. Rollback là phát lại lịch sử và bỏ phần tử cuối của ngăn xếp promote."><defs><marker id="py60-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><text x="20" y="24" class="t-b">out/registry/  (tên rút gọn: v1, v2, v3)</text><rect x="20" y="40" width="380" height="64" rx="8" class="f-phys-soft s-phys"/><text x="34" y="64" class="t-b">fraud-lgbm-f81991ad  (v1)</text><text x="34" y="86" class="t-small">model.joblib · model_meta.json · manifest.json</text><rect x="20" y="116" width="380" height="64" rx="8" class="f-phys-soft s-phys"/><text x="34" y="140" class="t-b">fraud-lgbm-4b482d65  (v2)</text><text x="34" y="162" class="t-small">model.joblib · model_meta.json · manifest.json</text><rect x="20" y="192" width="380" height="64" rx="8" class="f-phys-soft s-phys"/><text x="34" y="216" class="t-b">fraud-lgbm-6fec401c  (v3)</text><text x="34" y="238" class="t-small">model.joblib · model_meta.json · manifest.json</text><rect x="480" y="40" width="380" height="76" rx="8" class="f-pk-soft s-pk"/><text x="494" y="64" class="t-b">current.json</text><text x="494" y="86" class="t-small">{"version": "...4b482d65", "since": ...}</text><text x="494" y="106" class="t-small">đổi bằng os.replace: đọc thấy bản cũ hoặc mới</text><path d="M480 78 L400 142" class="f-none s-pk dash" stroke-width="2" marker-end="url(#py60-0-arrow)"/><rect x="480" y="140" width="380" height="116" rx="8" class="f-warn-soft s-warn"/><text x="494" y="164" class="t-b">history.jsonl  (chỉ ghi thêm)</text><text x="494" y="186" class="t-small">registered x3, promoted v1, v2, v3</text><text x="494" y="206" class="t-small">rolled_back v3 -> v2</text><text x="494" y="230" class="t-small">rollback = phát lại lịch sử, bỏ phần tử cuối</text></svg>
</div>
<figcaption>Hình 1. Bố cục registry của lab: phiên bản bất biến, con trỏ thay nguyên tử, lịch sử chỉ ghi thêm. Rollback chỉ đổi con trỏ, không huấn luyện lại và không xóa gì.</figcaption>
</figure>

Quyết định promote đi qua một hàm, và mọi kiểm tra nằm trước khi con trỏ đổi (`registry.py`, trích):

```python
def promote(self, version, *, actor, reason, force=False):
    if not reason.strip():
        raise RegistryError("promote phải có lý do")
    with self._locked():                      # fcntl.flock: hai promote không chen nhau
        self._check_consistent()              # con trỏ phải khớp lịch sử, nếu không dừng
        try:
            self.load(version)                # sha256 + đặc trưng + phiên bản thư viện
        except ModelLoadError as e:
            raise RegistryError(f"không promote được {version}: {e}") from e
        problems = self._gate(version)        # holdout có, cùng ngày với champion, không chồng train, không kém quá 0,01
        if problems and not force:
            raise RegistryError(f"{version} không qua cổng: " + "; ".join(problems))
        previous = self.current()
        _write_json_atomic(self.root / CURRENT, {"version": version, "since": now_iso()})   # ghi file tạm, fsync, os.replace
        self._append({"event": "promoted", "version": version, "previous": previous, "actor": actor,
                      "reason": reason, "forced_over": problems if force and problems else None})
```

Ba bản dùng cùng công thức bài 57 (LightGBM 100 vòng, hiệu chỉnh sigmoid trên 8 ngày cuối), chỉ khác ngày kết thúc và nhãn. Mỗi bản có ngưỡng p riêng để so sánh; quyết định chính là quy tắc chi phí kỳ vọng. Holdout ghi vào manifest là ngày 40 đến 50, nhãn cuối:

```text
 bản  huấn luyện đến    nhãn   chuyến  gian lận  ngưỡng  PR-AUC holdout  chồng train   KiB  version
  v1         ngày 30   final  100,022     2,011   0.149          0.8835        không   345  fraud-lgbm-f81991ad
  v2         ngày 40   final  133,296     2,859   0.196          0.8908        không   345  fraud-lgbm-4b482d65
  v3         ngày 50   known  166,584     2,890   0.157          0.8874           có   346  fraud-lgbm-6fec401c
```

v3 là bản duy nhất dùng nhãn «known»: chuyến gian lận có `label_known_at` sau ngày 50 vẫn là 0, đúng lỗi nhãn chưa chín của bài 54. Train của v3 gồm cả ngày 40 đến 50 nên holdout chồng lên dữ liệu học, manifest ghi `overlaps_train`, và cổng chặn thay vì đoán. Lab thử các lần từ chối trên một bản sao registry:

```text
  promote v3 (holdout chồng train): fraud-lgbm-6fec401c không qua cổng: holdout chồng lên dữ liệu huấn luyện, số đo không đáng tin
  promote v2 sau khi đổi 1 bit trong file: không promote được fraud-lgbm-4b482d65: sha256 của model.joblib không khớp meta: file hỏng hoặc nhầm file
  promote bản có PR-AUC holdout 0,5 (sửa manifest bản sao): fraud-lgbm-4b482d65 không qua cổng: PR-AUC holdout 0.5000 kém champion fraud-lgbm-f81991ad (0.8835) quá 0.01
  promote bản có holdout khác ngày với champion (sửa manifest bản sao): fraud-lgbm-4b482d65 không qua cổng: holdout của fraud-lgbm-f81991ad ([40, 50]) khác ngày với holdout của fraud-lgbm-4b482d65 ([41, 50]): không so được
  promote bản không có số đo holdout: fraud-lgbm-4b482d65 không qua cổng: chưa có số đo holdout trong manifest
  promote với lý do rỗng: promote phải có lý do
  promote phiên bản không tồn tại: không có phiên bản fraud-lgbm-00000000 trong registry
  rollback khi chỉ có một bản: không có bản trước để rollback
  promote khi con trỏ lệch lịch sử (giả lập chết sau khi đổi con trỏ): con trỏ current (fraud-lgbm-6fec401c) lệch lịch sử (fraud-lgbm-f81991ad): dừng, cần người xem history.jsonl
  rollback khi con trỏ lệch lịch sử: con trỏ current (fraud-lgbm-6fec401c) lệch lịch sử (fraud-lgbm-f81991ad): dừng, cần người xem history.jsonl
  đăng ký lại cùng mô hình (cùng sha256): phiên bản fraud-lgbm-f81991ad đã có; registry không ghi đè
```

Con trỏ và lịch sử là hai file, ghi hai bước. Nếu tiến trình chết giữa hai bước, `current.json` có thể trỏ v3 trong khi lịch sử còn [v1, v2], và một rollback «tin lịch sử» sẽ về v1, sai. Vì thế `promote` và `rollback` từ chối khi hai nguồn lệch (hai ca trước dòng đăng ký lại) và để người vận hành xem `history.jsonl`. Con trỏ ghi qua `fsync` trước `os.replace`; lab không `fsync` thư mục chứa và không xử lý nhiều máy. `joblib` là pickle: nạp file lạ là chạy mã của người ghi, sha256 chỉ phát hiện file hỏng hoặc nhầm file; ai được ghi vào thư mục registry là việc của hạ tầng. Cổng chỉ là một lớp: `force=True` vẫn đi qua, nhưng buộc có lý do và lịch sử ghi lại.

## Shadow mode: chấm điểm mà không quyết định

Shadow mode cho bản mới thấy lưu lượng thật mà không để nó quyết định gì. `ShadowRouter` chấm bằng champion, trả quyết định chỉ theo champion, rồi chấm thêm các mô hình thử và ghi log; mọi lỗi của phần shadow bị bắt và đếm (`routing.py`, trích):

```python
def decide(self, batch):
    p = self.champion.predict_proba(batch)
    flag = decide(p, batch["amount_vnd"].to_numpy().astype(float))     # quy tắc chi phí kỳ vọng
    try:
        for name, scorer in self.shadows.items():
            try:
                out = np.asarray(scorer.predict_proba(batch), dtype=float)
                if out.shape != (batch.height,):
                    raise ValueError(f"độ dài điểm {out.shape} khác số chuyến {batch.height}")
                columns[f"p_{name}"] = out.tolist()
            except Exception as e:      # mô hình thử hỏng hay trả sai độ dài không được kéo theo quyết định thật
                self.shadow_errors[name] += 1
        ...                             # ghi một dòng JSON mỗi chuyến vào log
    except Exception:                   # ví dụ log không ghi được
        self.log_errors += 1
    return flag
```

Lab phát lại ngày 50 đến 60 (33.416 chuyến) theo lô 1.000, v1 quyết định, v2 và v3 chỉ ghi điểm. Ba ca biên có verify: quyết định giống hệt khi champion chạy một mình; một mô hình thử ném lỗi, hoặc trả sai độ dài, hoặc log không ghi được thì quyết định vẫn không đổi:

```text
quyết định của router có shadow giống hệt quyết định của champion chạy một mình: True (547 cờ theo quy tắc chi phí kỳ vọng)
thời gian chấm 33,416 chuyến: champion một mình 0.06 giây; champion + 2 shadow + ghi log 1.74 giây (load average 1/5/15 phút: 92.85 99.01 161.33)
champion: score_one trên 100 chuyến ngẫu nhiên lệch điểm theo lô tối đa 0.00e+00
shadow hỏng: quyết định vẫn giống hệt = True, 34 lô lỗi được ghi; dòng log đầu: {"trip_id": "TN-0166585", "p_v1": 0.002489328019734975, "shadow_errors": {"v-broken": "ValueError: mô hình thử hỏng"}}
shadow trả sai độ dài: quyết định giống hệt = True, 34 lô lỗi; ghi log vào thư mục không có: quyết định giống hệt = True, 34 lô lỗi log
```

Thời gian chỉ để biết cỡ: shadow chạy tuần tự trên đường chính; production chạy nó ngoài đường nóng, ví dụ một consumer riêng đọc cùng event. So ba phiên bản trên nhãn cuối của cùng cửa sổ:

```text
 bản  PR-AUC  p trung bình     p99  PSI điểm
  v1  0.8864        0.0157  0.7933     0.000
  v2  0.8928        0.0172  0.9306     0.374
  v3  0.8891        0.0056  0.0988     0.194

quyết định và chi phí (triệu VND; phí duyệt 8,000 cho mọi cờ, bỏ sót = tiền chuyến + 20,000). Quy tắc chi phí kỳ vọng là quyết định chính; hai ngưỡng để so sánh:
 bản          quy tắc chi phí kỳ vọng                      ngưỡng v1                  ngưỡng riêng của bản
         cờ/1000   chi phí            cờ/1000   chi phí            ngưỡng   cờ/1000   chi phí
  v1        16.4      8.02               18.1      8.33             0.149      18.1      8.33
  v2        18.9      7.95               21.5      8.39             0.196      19.0      8.21
  v3         8.9     11.46                9.0     12.12             0.157       8.9     12.23
```

PR-AUC của v3 (0,8891) không thấp hơn v1 (0,8864): chỉ số xếp hạng không thấy lỗi. Cái hỏng là thang xác suất: p trung bình của v3 là 0,0056 so với tỉ lệ nền 0,0148, kéo theo số cờ giảm một nửa và chi phí tăng; đổi cách đặt ngưỡng không chữa được. Cột «PSI điểm» (phân phối điểm so với v1) không phân biệt được bản hỏng: seed 42 cho v2 0,374 và v3 0,194, nhưng ở seed 123 thứ tự đảo (bảng cuối mục drift). Tỉ lệ cờ và p99 đo thẳng vào chỗ quyết định xảy ra.

v3 khác v2 ở hai biến cùng lúc (thêm 10 ngày dữ liệu và nhãn chưa chín), nên lab thêm bản đối chứng v3f đổi đúng một biến: cùng dữ liệu đến ngày 50 nhưng nhãn cuối. Một seed cũng quá ít, nên lab lặp lại với seed 42, 7 và 123 (đánh giá bằng nhãn cuối):

```text
 seed bản  PR-AUC  cờ/1000  chi phí  ngưỡng v1  cờ/1000  chi phí  cờ/1000  chi phí
   42  v1  0.8864     16.4     8.02                18.1     8.33     18.1     8.33
   42  v2  0.8928     18.9     7.95                21.5     8.39     19.0     8.21
   42  v3  0.8891      8.9    11.46                 9.0    12.12      8.9    12.23
   42 v3f  0.8979     16.9     7.88                18.8     8.22     19.5     8.22
    7  v1  0.8607     19.0    10.09                22.0    10.37     22.0    10.37
    7  v2  0.8288     20.1    10.35                22.7    10.49     21.9    10.65
    7  v3  0.8696      9.9    13.69                11.0    13.15     12.1    11.99
    7 v3f  0.8810     19.8     9.66                22.4     9.90     23.1     9.98
  123  v1  0.8865     18.1     8.89                19.3     9.23     19.3     9.23
  123  v2  0.8961     19.8     8.76                21.2     8.97     19.0     8.64
  123  v3  0.9083      9.4    13.19                 9.8    13.58     10.3    12.94
  123 v3f  0.9024     19.7     8.41                20.9     8.76     18.5     8.82
```

Ở cả ba seed v3 gắn cờ chừng một nửa so với v1 theo quy tắc chi phí kỳ vọng (8,9 so với 16,4; 9,9 so với 19,0; 9,4 so với 18,1 trên 1.000 chuyến) và đắt hơn 1,36 đến 1,48 lần. Bản đối chứng v3f cho biết biến nào gây ra: chỉ đổi nhãn sang nhãn cuối, số cờ về 16,9, 19,8 và 19,7 trên 1.000 chuyến, chi phí 7,88, 9,66 và 8,41 triệu VND, không đắt hơn v1 ở seed nào. Trong bộ này lỗi đến từ nhãn chưa chín, không từ việc thêm dữ liệu. Về v2 thì không vững: PR-AUC cao hơn v1 ở 2 trong 3 seed (+0,0064, −0,0319, +0,0095) và chi phí chênh −0,08, +0,26, −0,13 triệu VND, đổi dấu theo seed.

## A/B theo hash khách

Shadow trả lời «bản mới chấm thế nào», chưa trả lời «quyết định theo bản mới có tốt hơn không». A/B chia khách làm hai nhóm, mỗi nhóm một mô hình. Cách chia cần cùng khách luôn cùng nhóm ở mọi tiến trình, tên thí nghiệm làm «muối» để hai thí nghiệm độc lập nhau, và không dùng `hash()` của Python vì nó đổi theo `PYTHONHASHSEED` (`routing.py`):

```python
def bucket(customer_id: str, experiment: str) -> int:
    digest = hashlib.sha256(f"{experiment}:{customer_id}".encode()).digest()
    return int.from_bytes(digest[:8], "big") % BUCKETS      # BUCKETS = 10_000

def assign_arm(customer_id, experiment, *, treatment_share):
    return "treatment" if bucket(customer_id, experiment) < treatment_share * BUCKETS else "control"
```

Lab này **không** đo phản ứng của người duyệt hay khách: nó có điểm của cả hai mô hình cho mọi chuyến và chỉ mô phỏng nhiễu do chia khách ngẫu nhiên. Lab chia 17.513 khách thành hai nhóm 50 %, kiểm tính ổn định qua hai tiến trình mới với `PYTHONHASHSEED` khác nhau, rồi so chi phí (treatment dùng v2, control giữ v1, cùng quy tắc chi phí kỳ vọng):

```text
chia 17,513 khách (0.0 giây): treatment 8,719 khách / 16,745 chuyến / 260 gian lận; control 8,794 khách / 16,671 chuyến / 234 gian lận
40 khách đầu CU-00000..CU-00039 qua hai tiến trình mới với PYTHONHASHSEED khác nhau: True (CTTCCCTTCCTTCCCCTTCTTCTCTTTCCTTTCCCCTTTC)
hai thí nghiệm khác tên: khách cùng ở treatment của cả hai = 0.250 (độc lập sẽ là 0.251)

chi phí (VND) trên 1.000 chuyến, treatment dùng v2, control dùng v1, cùng quy tắc chi phí kỳ vọng: treatment 250,533, control 226,054, chênh +24,478
khoảng tin cậy 95 % của chênh (bootstrap theo khách, 1.000 lần): [-36,321, +82,970]
A/A (cả hai nhóm dùng v1, 100 cách chia): chênh có độ lệch chuẩn 31,016, 95 % nằm trong [-68,331, +64,000]
toàn cửa sổ, nếu mọi khách dùng v2 thay v1: chi phí/1.000 chuyến thay đổi -2,247 (v1 240,137)
nửa độ rộng khoảng tin cậy 59,645 so với chênh thật 2,247: nửa độ rộng bằng chênh (power chừng 50 %) cần chừng 705 lần lưu lượng 10 ngày này, power 80 % cần chừng 1,440 lần, tức 48.1 triệu chuyến (ngoại suy thô: chuyến độc lập, chênh không đổi; ở lab chênh thật đổi dấu theo seed)
thiết bị có từ 2 khách trở lên: 157; trong đó 90 thiết bị (57%) có khách ở cả hai nhóm
```

Nhóm treatment đắt hơn control 24.478 VND trên 1.000 chuyến, dấu ngược với chênh thật trên toàn cửa sổ (v2 rẻ hơn 2.247 VND; trong lab ta biết điểm của cả hai mô hình cho mọi chuyến, điều A/B thật không có). Khoảng tin cậy (bootstrap theo khách, vì chuyến của một khách không độc lập) là [−36.321, +82.970] và phép A/A cho chênh trong [−68.331, +64.000]: riêng việc ai rơi vào nhóm nào đã gây nhiễu lớn hơn hiệu ứng của v2 hàng chục lần. Power: nửa độ rộng bằng chênh (power chừng 50 %) cần chừng 705 lần lưu lượng 10 ngày này, power 80 % cần chừng 1.440 lần, tức 48,1 triệu chuyến (ngoại suy thô, chênh thật còn đổi dấu theo seed). Thêm nhiễu giao thoa: trong 157 thiết bị có từ hai khách trở lên, 90 (57 %) có khách ở cả hai nhóm, mà gian lận cấu kết chính là nhiều tài khoản trên một thiết bị.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Trục chi phí chênh trên 1.000 chuyến từ âm 80 nghìn đến dương 80 nghìn VND. Dòng A/A: khoảng 95 phần trăm từ âm 68,3 nghìn đến dương 64,0 nghìn. Dòng A/B v2 so với v1: khoảng tin cậy từ âm 36,3 nghìn đến dương 83,0 nghìn, điểm ước lượng dương 24,5 nghìn. Dòng chênh thật nếu mọi khách dùng v2: âm 2,2 nghìn. Đường nét đứt đánh dấu số 0."><defs></defs><path d="M440 40 L440 200" class="f-none s-muted dash"/><text x="440" y="30" class="t-small" text-anchor="middle">0</text><path d="M100 200 L780 200" class="f-none s-line"/><path d="M100 196 L100 204" class="f-none s-line"/><text x="100" y="220" class="t-small" text-anchor="middle">-80.000</text><path d="M270 196 L270 204" class="f-none s-line"/><text x="270" y="220" class="t-small" text-anchor="middle">-40.000</text><path d="M440 196 L440 204" class="f-none s-line"/><text x="440" y="220" class="t-small" text-anchor="middle">0</text><path d="M610 196 L610 204" class="f-none s-line"/><text x="610" y="220" class="t-small" text-anchor="middle">+40.000</text><path d="M780 196 L780 204" class="f-none s-line"/><text x="780" y="220" class="t-small" text-anchor="middle">+80.000</text><text x="440" y="244" class="t-small" text-anchor="middle">chênh chi phí trên 1.000 chuyến (VND)</text><text x="100" y="58" class="t-small">A/A: cả hai nhóm dùng v1, 95 % trong 100 cách chia</text><path d="M150 70 L712 70" class="f-none s-warn" stroke-width="6"/><text x="100" y="118" class="t-small">A/B v2 so v1: khoảng tin cậy 95 %, điểm +24.478</text><path d="M286 130 L793 130" class="f-none s-pk" stroke-width="6"/><circle cx="544" cy="130" r="6" class="f-pk s-pk"/><text x="100" y="168" class="t-small">chênh thật khi mọi khách dùng v2: -2.247</text><circle cx="430" cy="180" r="6" class="f-phys s-phys"/></svg>
</div>
<figcaption>Hình 2. Chênh chi phí do chia khách ngẫu nhiên (A/A) rộng hơn nhiều so với chênh thật giữa v2 và v1, nên khoảng tin cậy của A/B chứa cả số 0 lẫn dấu sai. Khoảng tin cậy vượt mép phải của trục (+82.970).</figcaption>
</figure>

## Drift: PSI hai phía và những điều nó không nói

**Population Stability Index** so phân phối của một đặc trưng ở dữ liệu huấn luyện với dữ liệu hiện tại: cắt tập tham chiếu thành 10 khoảng theo phân vị, gọi e và a là tỉ lệ dòng của tham chiếu và hiện tại trong từng khoảng, PSI là tổng của (a − e) × ln(a / e). Ngưỡng 0,10 và 0,25 là quy ước kinh nghiệm. Cách cắt khoảng quyết định kết quả với khối giá trị trùng (cờ, số đếm nhiều số 0): lab dùng khoảng đóng bên phải (x ≤ biên) và với mỗi biên e thêm một biên ngay dưới nó, để khối giá trị bằng e có khoảng riêng và PSI thấy lệch về **cả hai phía**; logic này cũng nằm trong `model.py` của lab 57 (`drift.py`, trích):

```python
q = np.unique(np.quantile(ref, np.linspace(0, 1, bins + 1)[1:-1]))
edges = np.unique(np.concatenate([q, np.nextafter(q, -np.inf)]))     # biên phụ ngay dưới mỗi phân vị
e = np.bincount(np.searchsorted(edges, ref, side="left"), minlength=len(edges) + 1) / len(ref)
a = np.bincount(np.searchsorted(edges, cur, side="left"), minlength=len(edges) + 1) / len(cur)
e, a = np.clip(e, EPS, None), np.clip(a, EPS, None)      # khoảng rỗng không làm ln nổ
return float(np.sum((a - e) * np.log(a / e)))
```

Khoảng mở bên phải cho PSI bằng 0 khi cờ hiếm đổi từ 0,1 % lên 50 %; chỉ đóng bên phải thì vẫn mù khi khối giá trị trùng ở đầu trên (cờ 99,9 % số 1 tụt xuống 50 %). Test và dòng này của lab giữ cả hai phía:

```text
PSI hai phía cho cờ hiếm: tham chiếu 99,9 % số 0, hiện tại 50 % số 1: 3.45; tham chiếu 99,9 % số 1, hiện tại 50 % số 0: 3.45
```

PSI «bình thường» lớn cỡ nào? Hai nửa ngẫu nhiên của chính tập tham chiếu cho PSI lớn nhất rất nhỏ. Lab so ba tuần phục vụ (ngày 40 đến 60) với dữ liệu huấn luyện của v1, kèm PR-AUC của v1 trên nhãn cuối của từng tuần và độ quan trọng của đặc trưng:

```text
nhiễu nền: PSI lớn nhất giữa hai nửa ngẫu nhiên 23.000 dòng của chính tập tham chiếu = 0.0019
   cửa sổ (ngày)  chuyến  PR-AUC v1  cờ/1000  >=0,10  >=0,25  ba đặc trưng PSI lớn nhất
       40 đến 47  23,303      0.878     19.7       2       1  cust_prior_trips 0.653, cust_promo_share_30d 0.116, dev_newuser_7d 0.039
       47 đến 54  23,496      0.903     19.9       2       1  cust_prior_trips 0.834, cust_promo_share_30d 0.112, dev_newuser_7d 0.050
       54 đến 60  19,894      0.864     13.9       2       1  cust_prior_trips 0.992, cust_promo_share_30d 0.109, dev_newuser_7d 0.061
cust_prior_trips đếm số chuyến trước đó của khách từ đầu log (không có cửa sổ): trung bình 4.4 ở tập tham chiếu, 12.6, 14.4, 16.6 ở ba tuần
độ quan trọng của cust_prior_trips trong v1: 15.3% tổng mean|SHAP| (hạng 1/30), nhưng chỉ 3.2% gain; ba đặc trưng đầu theo mean|SHAP|: cust_prior_trips 15.3%, cust_promo_share_30d 14.0%, route_ratio 8.1%
tỉ lệ chuyến đầu tiên của khách (cust_prior_trips = 0): 30.9% ở tập tham chiếu, 8.6%, 6.9%, 5.9% ở ba tuần
```

`cust_prior_trips` là bộ đếm tích lũy từ đầu log nên chỉ tăng khi log dài ra (PSI 0,65, 0,83, 0,99); tỉ lệ chuyến đầu tiên của khách giảm từ 30,9 % xuống 5,9 % vì quần thể khách gần như cố định, một giả định của bộ sinh. Độ quan trọng cần đo bằng thứ phản ánh tác động lên điểm: theo gain đặc trưng này chỉ 3,2 %, theo mean|SHAP| nó đứng **đầu** (15,3 %; hạng 1 ở cả ba seed, 15,3 đến 20,4 %), nên drift của nó đáng điều tra: mô hình dựa vào một bộ đếm tự trôi. PR-AUC của v1 dao động 0,878, 0,903, 0,864 giữa các tuần (cùng mô hình, khác mẫu), nên một tuần PR-AUC không đủ để nói «tụt» hay «không tụt». `cust_promo_share_30d` lệch nhẹ (khoảng 0,11); lab chưa tách nó khỏi tạo tác dưới đây.

Bộ sinh còn hai tạo tác ảnh hưởng mọi phép đo drift và mọi valid của phần 07:

```text
hai tạo tác của bộ sinh dữ liệu (không phải hiện tượng thật):
  (a) tỉ lệ chuyến SẠCH dùng NEWUSER50: 4.5% ở tập tham chiếu, 1.3%, 1.1%, 0.9% ở ba tuần (quần thể khách gần như cố định nên khách mới cạn dần)
  (b) cust_promo_share_30d > 0 ở cửa sổ ngày 50+: 38.3% chuyến sạch, 3.4% chuyến gian lận (khách gian lận không dùng mã nào ngoài NEWUSER50 của promo_abuse)
```

Tạo tác (a) làm phân phối khuyến mãi tự đổi theo thời gian; tạo tác (b) cho mô hình một lối tắt không có ngoài đời (bài 61 thấy nó trong đóng góp SHAP). Kịch bản: từ ngày 50 nguồn GPS đổi và `start_pickup_gap_km` lớn gấp 3 ở mọi chuyến (chỉ cột này, để PSI chỉ đúng nó). Chi phí theo quy tắc chi phí kỳ vọng; hai dòng cuối đo xem huấn luyện lại có cứu được không:

```text
kịch bản (đặt vào dữ liệu, không quan sát): từ ngày 50 nguồn GPS đổi, start_pickup_gap_km của mọi chuyến lớn gấp 3
                PSI start_pickup_gap_km  đặc trưng >=0,25  PR-AUC v1  cờ v1   chi phí v1   chi phí v2
nguồn cũ                          0.000                 1      0.886    547    8,024,430    7,949,355
nguồn đổi (x3)                    2.453                 2      0.713  2,493   23,367,214   17,373,594
huấn luyện lại thì sao? vx học đến ngày 50 (nhãn cuối) trên dữ liệu có lỗi nguồn từ ngày 30; chi phí (triệu VND) trên cửa sổ ngày 50+, quy tắc chi phí kỳ vọng:
  v1 nguồn cũ 8.02, v1 nguồn đổi 23.37, v2 nguồn đổi 17.37, vx nguồn đổi 7.99
(v2 học trước lúc nguồn đổi nên không cứu được; vx học luôn dữ liệu lỗi nên số đo hồi phục, nhưng nó đã học một nguồn sai: vẫn phải tìm và sửa nguồn)
```

PSI của cột này vọt lên 2,45 trong khi nó chỉ chiếm 3,8 % mean|SHAP| (mục sau): nếu chỉ báo động khi đặc trưng quan trọng thì lỗi này bị bỏ lọt, nên lab báo động theo PSI ở **mọi** đặc trưng và dùng độ quan trọng chỉ để xếp thứ tự. Về huấn luyện lại: v2, học trước lúc nguồn đổi, không cứu được (17,37 so với 8,02 triệu VND). Bản vx, học đến ngày 50 trên dữ liệu có lỗi nguồn từ ngày 30 và nhãn cuối, **hồi phục** số đo (7,99), vì nó học luôn dữ liệu lỗi; ở ba seed là 7,99, 9,97, 8,41 so với 8,02, 10,09, 8,89 của v1 trên nguồn cũ. Số đo đẹp lại không có nghĩa nguồn ổn: mô hình đã học một nguồn sai, nên vẫn phải tìm và sửa nguồn.

```text
 seed  PSI cust_prior_trips  hạng mean|SHAP|  tác động  PSI lớn nhất còn lại  PSI GPS x3  cờ v1 x lần  chi phí v1 x lần
   42                 0.948                1     15.3%                 0.055       2.453         4.56              2.91
    7                 0.962                1     20.4%                 0.051       2.435         2.64              1.77
  123                 0.967                1     16.3%                 0.051       2.457         3.25              2.23

 seed  PSI điểm v2  PSI điểm v3  v1 nguồn cũ  v1 nguồn đổi  v2 nguồn đổi  vx nguồn đổi
   42        0.374        0.194         8.02         23.37         17.37          7.99
    7        0.457        0.330        10.09         17.89         18.06          9.97
  123        0.074        0.679         8.89         19.87         20.25          8.41
```

## Khi nào huấn luyện lại, và rollback

Từ các phép đo trên, ta gói chính sách thành một hàm trả về đúng một hành động và mọi lý do. Mọi ngưỡng là giả định của lab, không phải chuẩn (`drift.py`, trích):

```python
@dataclass(frozen=True)
class RetrainPolicy:
    max_age_days: float = 30.0                    # tuổi tối đa tính từ ngày cuối của dữ liệu huấn luyện
    max_pr_auc_drop: float = 0.03                 # tụt PR-AUC trên nhãn đã chín
    flag_rate_band: tuple[float, float] = (0.6, 1.6)   # tỉ lệ cờ so với mức chuẩn, không cần nhãn
    rollback_window_days: float = 7.0
# PSI >= 0,25 ở bất kỳ đặc trưng nào: investigate; từ 0,10: monitor; độ quan trọng chỉ để xếp thứ tự trong lý do
```

Chính sách chạy trên sáu tình huống:

```text
  ngày 54, tuần bình thường (v1 24 ngày tuổi): investigate
      - cust_prior_trips: PSI 0.834; mô hình dựa vào nó 15.3% tổng mean|SHAP| (hạng 1)
      - cust_promo_share_30d: PSI 0.112; mô hình dựa vào nó 14.0% tổng mean|SHAP| (hạng 2)
  ngày 60, đúng hạn lịch (v1 30 ngày tuổi): retrain
      - cust_prior_trips: PSI 0.834; mô hình dựa vào nó 15.3% tổng mean|SHAP| (hạng 1)
      - cust_promo_share_30d: PSI 0.112; mô hình dựa vào nó 14.0% tổng mean|SHAP| (hạng 2)
      - mô hình 30 ngày tuổi, quá hạn lịch 30 ngày
  ngày 55, nguồn GPS đổi (kịch bản mục 6): investigate
      - start_pickup_gap_km: PSI 2.453; mô hình dựa vào nó 3.8% tổng mean|SHAP| (hạng 9)
      - cust_prior_trips: PSI 0.948; mô hình dựa vào nó 15.3% tổng mean|SHAP| (hạng 1)
      - cust_promo_share_30d: PSI 0.111; mô hình dựa vào nó 14.0% tổng mean|SHAP| (hạng 2)
  v3 vừa promote 3 ngày trước: rollback
      - tỉ lệ cờ bằng 0.54 lần mức chuẩn, ngoài dải (0.6, 1.6); vừa promote 3 ngày trước
  v1 mất 0,05 PR-AUC trên nhãn đã chín: retrain
      - PR-AUC trên nhãn đã chín tụt 0.050
  chỉ cust_promo_share_30d lệch nhẹ (PSI của tuần 47 đến 54): monitor
      - cust_promo_share_30d: PSI 0.112; mô hình dựa vào nó 14.0% tổng mean|SHAP| (hạng 2)
```

Nhãn phải chín thì hai hành động retrain mới đáng tin: nhãn mới nhất làm PR-AUC tụt giả (bài 54) và dạy mô hình sai. Tỉ lệ cờ đo được ngay nên là tín hiệu sớm, nhưng cũng đổi khi lượng gian lận thật đổi, và dải 0,6 đến 1,6 là giả định. Kịch bản promote nhầm: promote v2 (qua cổng), rồi một người vội promote v3 bằng `force=True` kèm lý do. Sau ba ngày, tỉ lệ cờ của v3 được so với mức chuẩn **cùng ba ngày** nhờ giữ champion cũ chạy shadow, không so với tuần trước vì dữ liệu cũng biến động theo ngày:

```text
promote v2 (qua cổng) -> current = v2
promote v3 không ép: fraud-lgbm-6fec401c không qua cổng: holdout chồng lên dữ liệu huấn luyện, số đo không đáng tin
promote v3 với force=True và lý do -> current = v3
tỉ lệ cờ ngày 57-60: v3 phục vụ 4.4/1000; v2 chấm cùng ba ngày đó (shadow) 11.7/1000; tỉ lệ 0.38
so với v2 ngày 50-57 (21.9/1000) tỉ lệ là 0.20: mức chuẩn khác ngày lẫn thêm biến động của chính dữ liệu
chính sách: rollback (tỉ lệ cờ bằng 0.38 lần mức chuẩn, ngoài dải (0.6, 1.6); vừa promote 3 ngày trước)
rollback -> current = v2; đổi con trỏ và nạp lại mất 123.1 ms; điểm của bản đang phục vụ giống hệt điểm v2 trước đó: True
v3 vẫn còn trong registry để điều tra: True; số bản: 3
```

Rollback chỉ đổi con trỏ, kiểm bản đích và nạp lại từ đĩa (vài chục mili giây): không huấn luyện lại, không xóa v3, và điểm sau rollback giống hệt từng bit điểm v2 trước đó. Lịch sử cho thấy ai làm gì và vì sao:

```text
  promoted    ...f81991ad <- - bởi oncall-ml: bản đầu tiên, cùng công thức mô hình bài 57
  promoted    ...4b482d65 <- f81991ad bởi oncall-ml: holdout 40-50 không kém v1, shadow 50-60 cùng thứ bậc, A/B chưa phân biệt được
  promoted    ...6fec401c <- 4b482d65 bởi dev-in-a-hurry [ÉP QUA CỔNG]: muốn dùng dữ liệu mới nhất
  rolled_back ...6fec401c -> 4b482d65 bởi oncall-ml: tỉ lệ cờ giảm hơn một nửa sau promote v3; nghi nhãn chưa chín trong huấn luyện
```

(Trích.) Rollback là ngăn xếp: hai lần liên tiếp đi lùi hai bản, và pytest của lab kiểm cả promote lại sau rollback.

## MLflow thì sao

`mlflow_demo.py` đăng ký hai bản của lab vào MLflow Model Registry (SQLite cục bộ, không cần server), gán alias `champion`, và rollback bằng cách đưa alias về bản cũ. Kiểm ngày 07/11/2026: MLflow 3.18.0 chạy được trên Python 3.14.7 nhưng kéo theo nhiều phụ thuộc, nên lab đặt nó vào nhóm `--group mlflow`:

```text
mlflow 3.18.0, Python 3.14.7
đã log và đăng ký fraud-lgbm-f81991ad thành fraud-scoring phiên bản 1: models:/m-6034b7cc7c2e4772a69a75f72ef22617
đã log và đăng ký fraud-lgbm-4b482d65 thành fraud-scoring phiên bản 2: models:/m-6c2f37ce252b41bb912dfd968e2075d2
log hai mô hình mất 28.9 giây
alias champion -> phiên bản 1; nạp qua models:/fraud-scoring@champion cho điểm giống hệt bản trong registry thư mục: True
chuyển alias sang phiên bản 2: điểm giống hệt bản 2 = True
rollback = đưa alias về phiên bản 1: champion -> phiên bản 1
skops: 1143 KiB (joblib 345 KiB); nạp không khai báo bị từ chối = True; 5 kiểu cần liệt kê: collections.OrderedDict, lightgbm.basic.Booster, lightgbm.sklearn.LGBMClassifier, sklearn.calibration._CalibratedClassifier, sklearn.calibration._SigmoidCalibration; nạp với danh sách đó cho điểm giống hệt = True
thư mục out/mlflow: 1.7 MB
Kiểm chứng MLflow: OK 5/5
```

Điểm giống: phiên bản bất biến, alias thay con trỏ. Điểm khác: MLflow lưu mô hình scikit-learn bằng cloudpickle (nó cảnh báo như ở trên và gợi ý skops). skops không chạy mã khi nạp nhưng bắt liệt kê kiểu được tin, gồm cả `lightgbm.basic.Booster`, và bản lưu lớn gấp ba. Cổng promote, lý do và lịch sử «ai ép» vẫn do bạn viết.

## Chạy và kiểm chứng

Không cần Docker. Cần uv và Python 3.14; macOS cần `brew install libomp`; lần đầu cần mạng, rồi lab dựng đặc trưng từ event và sinh lại dữ liệu cho seed 7 và 123 (lưu vào `out/`). Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/60-model-registry-ab-drift examples/toi-noi-python/60-model-registry-ab-drift/main.py
uv run --project examples/toi-noi-python/60-model-registry-ab-drift examples/toi-noi-python/60-model-registry-ab-drift/main.py --verify
uv run --project examples/toi-noi-python/60-model-registry-ab-drift pytest examples/toi-noi-python/60-model-registry-ab-drift -q
uv run --group mlflow --project examples/toi-noi-python/60-model-registry-ab-drift examples/toi-noi-python/60-model-registry-ab-drift/mlflow_demo.py --verify
```

Dòng cuối của `--verify` (lần chạy nguội này mất 2 phút 11 giây, load average 1 phút từ 89.53 đến 154.97 trên 10 lõi) và của `pytest`:

```text
Kiểm chứng: OK 88/88
26 passed in 11.88s
```

`--verify` kiểm 88 bất biến, không ghim số tuyệt đối hay tốc độ: shadow lỗi không đổi quyết định, cổng promote từ chối đúng các ca (kể cả con trỏ lệch lịch sử), PSI hai phía, kịch bản GPS bị PSI bắt, rollback cho điểm giống hệt, và ở cả ba seed v3 gắn cờ ít hơn 0,75 lần v1 còn v3f thì không.

| Hiện tượng | Kiểm tra trước |
|---|---|
| PSI của một đặc trưng vọt lên nhưng PR-AUC trên nhãn đã chín không đổi | Đặc trưng có phải bộ đếm tích lũy hoặc phụ thuộc lịch sử khách không; đừng bỏ qua vì gain thấp, xem trung bình trị tuyệt đối SHAP |
| PR-AUC của bản mới bằng hoặc cao hơn nhưng số cờ lệch hẳn | So tỉ lệ cờ và p99 với bản đang chạy cùng khoảng thời gian; kiểm nhãn lúc huấn luyện đã chín chưa |
| Nhóm A/B treatment đắt hơn dù mô hình mới tốt hơn trên shadow | Chạy A/A trước để biết nhiễu chia khách; kiểm khoảng tin cậy bootstrap theo khách |
| `RegistryError` «con trỏ lệch lịch sử» | Tiến trình chết giữa hai bước ghi; xem `history.jsonl`, sửa tay, rồi mới promote hay rollback |
| Sau huấn luyện lại số đo đẹp trở lại nhưng PSI nguồn vẫn cao | Mô hình đã học dữ liệu lỗi; tìm và sửa nguồn rồi huấn luyện lại bằng dữ liệu sạch |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dữ liệu là tổng hợp (200.000 chuyến, 60 ngày) và có hai tạo tác nêu ở mục drift, nên số đo không đại diện cho dữ liệu thật; «v3 hỏng» và hai kịch bản là do lab dựng. Nhãn đánh giá là nhãn cuối; chỉ v3 huấn luyện bằng nhãn biết tại ngày 50. Mô hình chi phí có hai giả định: gắn cờ chặn hết thiệt hại (người duyệt không sai, duyệt trễ vẫn chặn đủ), và <code>amount_vnd</code> đại diện cho thiệt hại (với promo_abuse thiệt hại thật là phần giảm). Registry là một thư mục trên một máy, không kiểm soát truy cập (việc của hạ tầng), <code>fcntl.flock</code> chỉ có trên Unix. A/B ở đây là mô phỏng nhiễu chia khách, không đo phản ứng người duyệt. Ngưỡng chính sách là lựa chọn của lab. Đã chạy với Python 3.14.7, lightgbm 4.7.0, scikit-learn 1.9.1, numpy 2.5.4, polars 2.1.0, joblib 1.6.0, mlflow 3.18.0.</p>
</div>

**Bài tập:** (1) Một A/B chia 10 % khách vào treatment. Sau một tuần, control có 900 chuyến gian lận và treatment có 120. Nếu hai nhóm giống nhau, treatment kỳ vọng có bao nhiêu, và vì sao chưa thể kết luận treatment tệ hơn? (2) Lịch sử registry: promote v1, promote v2, rollback, promote v3. Con trỏ trỏ vào bản nào, và rollback thêm một lần nữa thì trỏ vào đâu?

<details><summary>Xem lời giải</summary><p>(1) Tỉ lệ khách 10 : 90 nên kỳ vọng 900 × 10 / 90 = <b>100</b> chuyến; 120 cao hơn 20 %, nhưng với chừng 100 sự kiện độ lệch chuẩn kiểu Poisson đã là 10, nên 120 chỉ lệch khoảng hai độ lệch chuẩn, và chuyến của cùng khách hay thiết bị không độc lập nên nhiễu thật lớn hơn. Cần so tỉ lệ kèm khoảng tin cậy bootstrap theo khách, và chạy A/A trước. (2) Ngăn xếp: [v1], [v1, v2], rollback cho [v1], promote v3 cho [v1, v3]. Con trỏ trỏ vào <b>v3</b>; rollback thêm một lần về <b>v1</b>; lần nữa thì lỗi vì không còn bản trước.</p></details>

Bài trước: [bài 59](/articles/python-real-time-scoring-feature-store-redis-latency). Bài tiếp: [bài 61](/articles/python-shap-explainability-audit-human-review).

## Tài liệu gốc

- [MLflow: Model Registry](https://mlflow.org/docs/latest/model-registry.html).
- [scikit-learn: Model persistence](https://scikit-learn.org/stable/modules/model_persistence.html).
- [skops: Persistence](https://skops.readthedocs.io/en/stable/persistence.html).
- [Python: os.replace](https://docs.python.org/3/library/os.html#os.replace).
- [Python: fcntl.flock](https://docs.python.org/3/library/fcntl.html).
- [Python: hashlib](https://docs.python.org/3/library/hashlib.html).
