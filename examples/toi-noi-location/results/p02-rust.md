# P02 (phần Rust): kết quả lab, LOC-15 … LOC-18

Nguồn duy nhất cho các con số của bốn bài Rust trong P02: Kalman, HMM + Viterbi, bám đường online, kho vị trí mới nhất. Mọi khối `text` bên dưới là output thật của lệnh ghi
ngay trên nó, không chỉnh sửa (không cắt dòng, không đổi số; đường dẫn đồ thị đã được lab in tương đối so với `examples/toi-noi-location`), trừ những chỗ có ghi rõ là «chép từ màn hình» hoặc «đã lọc». Chạy từ gốc repo.
Quy tắc đọc số: `examples/toi-noi-location/README.md`, mục «Quy tắc số liệu». Kiểm tra toàn phần: `bash examples/toi-noi-location/verify-p02-rust.sh` (xem cuối file).

## Đọc cái này trước khi trích số

**1. Mọi độ chính xác là trên nhiễu tổng hợp do chính lab sinh ra.** Không có GPS người thật trong loạt này. `map_match::trace` cho xe chạy trên lộ trình thật của đồ thị HCM (tăng tốc 1,8 m/s², phanh 2,5 m/s²,
chậm ở khúc cua, dừng ở khoảng 8% nút giao 8–35 s) rồi làm hỏng điểm bằng: sai số Gauss mỗi trục `σ` **tương quan theo thời gian** (AR(1), hằng số 8 s, vì sai số GPS thật không trắng),
accuracy báo cáo = 1,5·σ nhân hệ số lognormal (độ lệch chuẩn 0,25) và **vắng ở 10% điểm**, 0,5% điểm nhảy xa 25–70 m (đa đường), 1% điểm vắng ngẫu nhiên, speed và heading có nhiễu,
và với ca khó có thêm vùng **lệch ổn định** 12 m một hướng ngẫu nhiên (hẻm cao tầng) quanh điểm khó, nhiễu ×1,5 trong vùng đó, accuracy báo cáo chỉ ×1,2 (điện thoại báo thiếu). Kết quả cho thấy thuật toán chạy đúng
**với mô hình nhiễu này**; mô hình nhiễu của GPS thật (phân phối đuôi dày, phụ thuộc thiết bị, độ chính xác báo sai khác) chưa được kiểm.

**2. Tách chọn tham số khỏi báo cáo.** Tham số chọn bằng seed `1106` (tập chuyến khác hẳn) và, với ca khó, chỉ số chẵn của danh mục; báo cáo dùng seed `20261106` và chỉ số lẻ.
Mỗi lab in bảng chọn tham số và dòng «chọn: …; mặc định trong …::default(): …». Giá trị mặc định trong mã (`KalmanConfig::default`: `accel_sigma = 4`, cổng NIS 9,21; `MatchConfig::default`: `beta_m = 1`, `beta_frac = 0`, `accuracy_to_sigma = 0,67`)
đã được đặt bằng đúng giá trị mà bảng chọn tham số trên tập tune trả về. Tham số của ca khó (bán kính 45 m, lệch 12 m, ngưỡng tìm cầu vượt/song song) đặt trước khi xem kết quả, không chỉnh theo kết quả.
Hai điều được làm **sau khi xem kết quả** và phải nói rõ: (a) thêm thước «cạnh nới nút» (xem dưới) vì thước chặt bị trần thấp do cạnh HCM rất ngắn; (b) sửa lỗi phạt đi lùi trên cùng cạnh (mục LOC-16).
Dòng «độ nhạy» ở mục hầm của LOC-17 (`gap_break_s = 300`) chỉ để báo độ nhạy, không dùng chọn tham số.

**3. Thước đo.** «cạnh chặt»: cạnh được gán trùng cạnh thật hoặc `twin` của nó (bỏ qua chiều). «cạnh nới nút»: như trên hoặc cạnh kề chung nút mà điểm chiếu cách vị trí thật ≤ 12 m (cạnh đồ thị HCM nhiều khi chỉ 5–40 m,
điểm thật cách nút vài mét bị gán sang cạnh kề là hợp lý). «chiều nới»: như «nới» nhưng phải đúng chiều xe chạy (cạnh có hướng trùng hoặc nối tiếp). «recall» và «RMF» là chỉ số lộ trình kiểu Newson–Krumm:
recall là tỉ lệ độ dài cạnh thật mà lộ trình suy ra có; `RMF = (độ dài thiếu + độ dài thừa) / độ dài thật` (0 là hoàn hảo, trên 1 là lộ trình sai nhiều hơn đúng). Cạnh `twin` coi là một. Điểm không có ứng viên tính là sai.
«snap» là baseline ngây thơ: mỗi điểm về cạnh gần nhất đi được bằng ô tô, lộ trình là dãy cạnh không lặp liên tiếp (không nối khoảng trống).

**4. Giới hạn của mô hình bám đường.** Chỉ profile ô tô. Bỏ qua cấm rẽ và chi phí rẽ (dùng Dijkstra của `map_match::route`, giống `road_graph::sp`). Một chuyến chạy 2–4,5 km trên HCM có nhiều đoạn qua hẻm vì profile của đồ thị cho ô tô vào hẻm không có thẻ cấm.
Số hạng hướng đi (heading) là tùy chọn, mặc định tắt. Không dùng speed báo cáo ở HMM.

## Cấu hình đo và điều kiện đo

| Mục | Giá trị |
|---|---|
| Máy | Apple M1 Max, 10 lõi, 64 GB RAM, macOS 26.6.2, aarch64 |
| Toolchain | Rust 1.98.1 (rustc 2026-09-01), edition 2024, resolver 3; `--release` (`lto = "thin"`, `codegen-units = 4`, `debug = 1`) |
| Thư viện mới (ghim trong `rust/Cargo.lock`) | `rdkafka` 0.39.0 (`rdkafka-sys` 4.10.0 = librdkafka 2.12.1, build bằng `make` + `cc`, **không cần cmake**), `prost` 0.14.4, `prost-build` 0.14.4, `protox` 0.10.0 (biên dịch `.proto` thuần Rust, **không cần `protoc`**; `build.rs` của `location-store` gọi `protox::compile` rồi `prost_build::Config::compile_fds`), `redis` 1.8.1, `proptest` 1.12.0, `h3o` 0.11.1 |
| Đồ thị | `data/hcm-graph.bin`: 126.699 nút, 319.074 cạnh có hướng (OSM, © OpenStreetMap contributors, ODbL). Fixture: `fixtures/khu-mau.json`, 234 nút, 840 cạnh |
| Seed | báo cáo `20261106`, chọn tham số `1106` |
| Hạ tầng | Redis 7.4.12 (`redis:7`, `127.0.0.1:6391`), Kafka `apache/kafka:4.1.0` (`127.0.0.1:9096`), Docker 29.7.2; broker Kafka chạy sẵn và dùng chung với phiên Go (đã thấy container ở trạng thái Paused trong lúc làm việc, xem mục Test); lab chỉ tạo và xóa topic riêng `location.updates.v1.lab18-<pid>`, test dùng `location.updates.v1.test-<pid>-<nanos>`, khóa Redis riêng `tn:loc18:<pid>`, `tn:test:<pid>` |
| Mã nguồn | worktree `location-series`, base commit `d7abfcc`, các thay đổi của phần này chưa commit |

**Điều kiện đo: máy chịu tải thay đổi lớn.** Mỗi lab in `uptime` lúc bắt đầu, lúc kết thúc và trước phép đo CPU. Tải trung bình 1 phút trong các lần chạy chính thức từ 28 tới 170 trên 10 lõi.
Số **không phụ thuộc thời gian** (RMSE, tỉ lệ đúng cạnh, recall, RMF, số lần gãy chuỗi, số tài xế, số bản ghi) tái lập từng chữ số với cùng seed. Số **thời gian** (µs, ms, cập nhật/s, truy vấn/s) nhiễu mạnh,
đặc biệt `tối đa` và p99; ưu tiên p50 và tỉ lệ so sánh trong cùng một lần chạy; ở mục CPU của LOC-17 và mục đồng thời của LOC-18 có ghi nhiều lần chạy cạnh nhau để thấy mức nhiễu.

## Kết quả theo bài

### LOC-15: Kalman vận tốc không đổi, lọc nhiễu GPS

Lệnh (từ gốc repo):

```bash
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc15_kalman
```

Cấu hình: 40 chuyến 2–4,5 km mỗi bộ (tune: 29.734 điểm thật, báo cáo: 26.392 điểm), 1 Hz, `σ` mặc định 6 m với tương quan 8 s, seed báo cáo 20261106, seed chọn tham số 1106. Không cần Redis hay mạng.

```text
== LOC-15: Kalman vận tốc không đổi ==
đồ thị: data/hcm-graph.bin: 126699 nút, 319074 cạnh có hướng; 40 chuyến mỗi bộ (dài 2–4,5 km trên HCM, 0,4–1,5 km trên fixture), lấy mẫu 1 Hz; seed báo cáo 20261106, seed chọn tham số 1106
uptime lúc bắt đầu: 23:24  up 35 days, 15:07, 1 user, load averages: 69.33 68.27 73.02
lưu ý: nhiễu GPS do chính lab sinh ra (có seed); mọi độ chính xác dưới đây là trên nhiễu tổng hợp này, không phải GPS thật.

chuyến chọn tham số: 29734 điểm thật; chuyến báo cáo: 26392 điểm thật

-- 1. chọn gia tốc nhiễu quá trình trên bộ chọn tham số (σ 6 m, tương quan 8 s, có điểm nhảy ngẫu nhiên) --
 accel_sigma   gate_nis       RMSE (m)        p95 (m)
        0.25        tắt         11.452         21.064
         0.5        tắt          9.892         17.346
           1        tắt          9.275         15.954
           2        tắt          9.075         15.433
           4        tắt          9.031         15.229
           8        tắt          9.045         15.119
        0.25       9.21         13.160         24.146
         0.5       9.21          9.834         17.194
           1       9.21          9.053         15.654
           2       9.21          8.820         15.200
           4       9.21          8.810         15.109
           8       9.21          8.975         15.118
chọn: accel_sigma = 4, gate_nis = Some(9.21) (RMSE tune 8.810 m); mặc định trong KalmanConfig::default(): accel_sigma = 4, gate_nis = Some(9.21)

-- 2. RMSE so với vị trí thật, theo độ lệch chuẩn nhiễu mỗi trục (bộ báo cáo, tham số đã chọn ở mục 1) --
     σ (m) tương quan   RMSE thô  RMSE Kalman       giảm      p95 thô p95 Kalman
         2      8 s       4.52         3.43      24.2%       5.01       5.17
         4      8 s       6.68         6.08       9.0%      10.03      10.23
         6      8 s       9.22         8.87       3.7%      15.04      15.35
        10      8 s      14.62        14.59       0.2%      25.05      25.41
        20      8 s      28.59        29.23      -2.2%      49.78      51.07
         6    trắng       9.20         7.07      23.2%      15.00      12.08
        10    trắng      14.58        10.77      26.1%      24.98      18.68

-- 3. sai số theo pha chuyển động: lọc không làm trễ giờ, nhưng kết quả chạy lệch so với xe khi vận tốc đổi --
nhiễu σ = 6 m, tương quan 8 s:
pha         số điểm   RMSE thô  RMSE Kalman dọc hướng (Kalman)  dọc hướng (thô)
Accel          1242       9.58         8.95            -1.09 m           0.03 m
Brake           543       8.60         8.42             0.98 m           0.41 m
Cruise        16428       9.24         8.91             0.08 m           0.15 m
Stopped        5686       9.29         8.83            -0.28 m          -0.44 m
Turning        2089       8.76         8.76            -0.65 m          -0.09 m
nhiễu σ = 2 m, trắng:
pha         số điểm   RMSE thô  RMSE Kalman dọc hướng (Kalman)  dọc hướng (thô)
Accel          1242       5.38         3.36            -0.54 m           0.02 m
Brake           543       3.98         3.07             0.58 m           0.14 m
Cruise        16428       4.45         3.01            -0.01 m           0.00 m
Stopped        5686       4.73         3.25             0.07 m           0.01 m
Turning        2089       4.27         3.24            -0.29 m           0.03 m

-- 4. làm mượt RTS có trễ cố định: biết thêm L điểm sau thì kết quả cho điểm hiện tại tốt hơn, nhưng phải chờ L giây --
     L     RMSE (m) so với lọc
     0        8.870       0.0%
     1        8.214      -7.4%
     2        8.041      -9.3%
     3        8.046      -9.3%
     5        8.064      -9.1%
     8        8.057      -9.2%
    12        8.059      -9.2%

-- 5. khi nào Kalman không giúp hoặc hại (RMSE m; thô → Kalman) --
tình huống                                                       RMSE thô     Kalman  thay đổi
GPS tốt: σ = 2 m, nhiễu trắng                                        4.53       3.10    -31.6%
σ = 6 m, tương quan 8 s (mặc định)                                   9.22       8.87     -3.7%
σ = 20 m, tương quan 8 s, có cổng NIS                               28.59      29.23      2.2%
σ = 20 m, tương quan 8 s, không cổng NIS                            28.59      29.24      2.2%
3% điểm nhảy xa 25–70 m, không cổng NIS                             12.05      10.98     -8.9%
3% điểm nhảy xa 25–70 m, cổng NIS 9,21                              12.05       9.99    -17.1%
app không báo accuracy (dùng 8 m mặc định)                           9.22       8.85     -4.0%
accel_sigma = 0,25 (quá mượt) so với gia tốc thật của xe             9.22      13.19     43.1%
accel_sigma = 8 (bám sát đo)                                         9.22       9.04     -1.9%
lấy mẫu thưa 5 s (cùng tham số)                                      9.42       9.40     -0.2%
lệch ổn định 15 m suốt 40% chuyến (hẻm cao tầng), accuracy không đổi      13.23      12.99     -1.8%

-- 6. trích: khúc cua gắt nhất trong chuyến báo cáo số 0 (cột: giây, thật lat,lng; thô lat,lng; Kalman lat,lng) --
t_s,truth_lat,truth_lng,raw_lat,raw_lng,kalman_lat,kalman_lng,truth_speed_mps
70,10.7665384,106.6688862,10.7664500,106.6689270,10.7664490,106.6689187,3.8
71,10.7665049,106.6688915,10.7664623,106.6689061,10.7664457,106.6689089,3.8
72,10.7664714,106.6688968,10.7663842,106.6689157,10.7663983,106.6689115,3.8
73,10.7664379,106.6689021,10.7662881,106.6689140,10.7663206,106.6689118,3.8
74,10.7664228,106.6688810,10.7662676,106.6688826,10.7662671,106.6688899,3.8
75,10.7664166,106.6688471,10.7662972,106.6688130,10.7662477,106.6688512,3.8
76,10.7664105,106.6688131,10.7663254,106.6687664,10.7662872,106.6687861,3.8
77,10.7664044,106.6687792,10.7663375,106.6686989,10.7663241,106.6687103,3.8
78,10.7663982,106.6687452,10.7663509,106.6687023,10.7663484,106.6686881,3.8
79,10.7663921,106.6687113,10.7663740,106.6686811,10.7663720,106.6686657,3.8
80,10.7663859,106.6686774,10.7663546,106.6686533,10.7663612,106.6686504,3.8
81,10.7663856,106.6686473,10.7663055,106.6686569,10.7663201,106.6686500,3.8
82,10.7664187,106.6686399,10.7662920,106.6686434,10.7662934,106.6686428,3.8
83,10.7664518,106.6686325,10.7663607,106.6686101,10.7663311,106.6686181,3.8
84,10.7664849,106.6686251,10.7664607,106.6685759,10.7664028,106.6685878,3.8

uptime lúc kết thúc: 23:24  up 35 days, 15:07, 1 user, load averages: 69.33 68.27 73.02
```

Điều nó cho thấy:

- Kalman **giúp ít hơn người ta tưởng** vì sai số GPS tương quan theo thời gian. Cùng σ = 6 m: nhiễu trắng giảm RMSE 23,2% (9,20 → 7,07 m), nhưng với tương quan 8 s chỉ giảm 3,7% (9,22 → 8,87 m); σ = 10 m tương quan: 0,2%; σ = 20 m: **tệ hơn** 2,2% (28,59 → 29,23). Lọc trung bình hóa nhiễu nhanh, không xóa được độ lệch chậm.
- p95 **xấu đi** ở mọi mức tương quan dù RMSE tốt hơn (σ = 4 m: 10,03 → 10,23; σ = 6 m: 15,04 → 15,35), tức đuôi sai số không được cải thiện.
- Độ trễ phải trả nằm ở sai số dọc hướng đi khi vận tốc đổi: −1,09 m lúc tăng tốc, +0,98 m lúc phanh, −0,65 m ở khúc cua (σ = 6 m); khi GPS tốt (σ = 2 m trắng) là −0,54 m, +0,58 m, −0,29 m. Nhỏ so với nhiễu, nhưng là sai số có hướng, không phải ngẫu nhiên. Làm mượt RTS biết thêm L điểm sau thì RMSE giảm 7,4% (L = 1), 9,3% (L = 2) rồi dừng (L = 3: 9,3%, L = 12: 9,2%): chờ 2 giây là đủ, chờ thêm không lợi. Chỗ phẳng này suy được từ tham số, không cần tới sai số tương quan: với `accel_sigma` = 4, `R` = 36, 1 Hz, tính tay từ công thức trọng số của điểm đo sau 1, 2, 3 bước chỉ còn cỡ 0,22, 0,07, 0,02, dù nhiễu thật thế nào. Phần RMSE còn lại chưa tách được bao nhiêu là do tương quan.
- Khi **hại hoặc vô ích**: `accel_sigma = 0,25` (mô hình quá mượt so với gia tốc thật của xe) làm RMSE tệ hơn 43,1% (9,22 → 13,19); lệch ổn định 15 m suốt 40% chuyến chỉ giảm 1,8% (13,23 → 12,99), Kalman không sửa được độ lệch; lấy mẫu thưa 5 s giảm 0,2%. Cổng NIS giúp rõ khi có điểm nhảy xa: 3% điểm nhảy làm RMSE thô 12,05 m, Kalman không cổng 10,98 m, có cổng 9,99 m.
- Bất ngờ nhỏ: vắng accuracy (dùng 8 m mặc định) gần như không khác báo cáo (8,85 so với 8,87 m), vì accuracy trong mô hình nhiễu của lab hiệu chuẩn tốt (lệch lognormal 0,25). Với điện thoại báo accuracy sai hẳn thì kết luận này không đúng.
- Chọn tham số: bảng 1 gần như hòa giữa `accel_sigma` 2 và 4 (8,820 và 8,810 m); lab chọn 4.

Dữ liệu cho hình SVG (khúc cua, mục 6 ở output trên): cột `truth_*`, `raw_*`, `kalman_*` là vĩ độ, kinh độ; 15 điểm quanh khúc cua gắt nhất của chuyến báo cáo số 0 (xe chạy 3,8 m/s, σ = 6 m, tương quan 8 s).

### LOC-16: HMM + Viterbi (Newson–Krumm) so với snap về cạnh gần nhất

Lệnh:

```bash
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc16_hmm
```

Cấu hình: 40 chuyến báo cáo mỗi mức nhiễu (cùng tập chuyến, nhiễu khác nhau), 1 Hz trừ mục 3, `MatchConfig::default()` (bán kính 50 m, tối đa 8 cạnh vật lý = tới 16 trạng thái có hướng, `beta_m = 1`, `accuracy_to_sigma = 0,67`, vận tốc tối đa 38 m/s). Bảng chọn tham số dùng 40 chuyến tune khác hẳn (ba mức σ ở 1 Hz và một mức ở 5 s/điểm).

```text
== LOC-16: HMM + Viterbi ==
đồ thị: data/hcm-graph.bin: 126699 nút, 319074 cạnh có hướng; 40 chuyến mỗi mức nhiễu (dài 2–4,5 km trên HCM, 0,4–1,5 km trên fixture); seed báo cáo 20261106, seed chọn tham số 1106; xe ô tô
uptime lúc bắt đầu: 23:24  up 35 days, 15:07, 1 user, load averages: 69.33 68.27 73.02
lưu ý: nhiễu GPS do chính lab sinh ra (có seed); mọi độ chính xác dưới đây là trên nhiễu tổng hợp này, không phải GPS thật.

-- 1. chọn tham số trên bộ chọn tham số (40 chuyến: 10 mỗi mức σ = 4, 8, 15 m ở 1 Hz, và 10 chuyến σ = 8 m ở 5 s/điểm; không dùng bộ báo cáo) --
  beta_m  beta_frac        acc→σ   cạnh chặt    cạnh nới      RMF      gãy
       1          0          0.5      72.95%      88.89%    0.080       39
       1          0         0.67      72.98%      88.93%    0.081       39
       1          0            1      72.35%      88.24%    0.090       39
       1          0          1.5      72.24%      88.13%    0.090       39
       1        0.1          0.5      73.07%      88.65%    0.084       38
       1        0.1         0.67      73.10%      88.91%    0.081       38
       1        0.1            1      72.64%      88.40%    0.089       38
       1        0.1          1.5      72.26%      88.04%    0.092       38
       1        0.3          0.5      72.82%      88.21%    0.092       38
       1        0.3         0.67      72.95%      88.65%    0.086       38
       1        0.3            1      72.87%      88.75%    0.084       38
       1        0.3          1.5      72.33%      88.19%    0.093       38
       2          0          0.5      72.79%      88.60%    0.083       38
       2          0         0.67      72.79%      88.78%    0.081       38
       2          0            1      72.69%      88.43%    0.088       38
       2          0          1.5      72.29%      88.12%    0.090       38
       2        0.1          0.5      72.85%      88.62%    0.085       38
       2        0.1         0.67      72.62%      88.67%    0.082       38
       2        0.1            1      73.03%      88.85%    0.081       38
       2        0.1          1.5      72.41%      88.23%    0.090       38
       2        0.3          0.5      72.54%      88.27%    0.095       38
       2        0.3         0.67      72.53%      88.61%    0.087       38
       2        0.3            1      72.70%      88.76%    0.085       38
       2        0.3          1.5      72.42%      88.37%    0.092       38
       3          0          0.5      72.62%      88.54%    0.083       38
       3          0         0.67      72.44%      88.64%    0.082       38
       3          0            1      72.82%      88.73%    0.081       38
       3          0          1.5      72.66%      88.33%    0.089       38
       3        0.1          0.5      72.69%      88.43%    0.087       38
       3        0.1         0.67      72.53%      88.66%    0.085       38
       3        0.1            1      73.01%      88.88%    0.082       38
       3        0.1          1.5      72.65%      88.37%    0.092       38
       3        0.3          0.5      72.48%      88.13%    0.097       38
       3        0.3         0.67      72.43%      88.48%    0.088       38
       3        0.3            1      72.73%      88.86%    0.087       38
       3        0.3          1.5      71.96%      88.31%    0.093       38
       6          0          0.5      72.29%      88.22%    0.091       38
       6          0         0.67      72.48%      88.64%    0.085       38
       6          0            1      72.64%      88.83%    0.082       38
       6          0          1.5      72.06%      88.43%    0.091       38
       6        0.1          0.5      72.25%      88.18%    0.096       38
       6        0.1         0.67      72.39%      88.63%    0.089       38
       6        0.1            1      72.61%      88.89%    0.085       38
       6        0.1          1.5      71.89%      88.36%    0.093       38
       6        0.3          0.5      72.03%      87.90%    0.103       38
       6        0.3         0.67      72.38%      88.40%    0.093       38
       6        0.3            1      72.21%      88.71%    0.088       38
       6        0.3          1.5      71.33%      88.13%    0.096       38
      10          0          0.5      71.65%      87.82%    0.103       38
      10          0         0.67      72.04%      88.25%    0.093       38
      10          0            1      71.89%      88.77%    0.088       38
      10          0          1.5      71.50%      88.13%    0.094       38
      10        0.1          0.5      71.69%      87.74%    0.105       38
      10        0.1         0.67      71.86%      88.25%    0.094       38
      10        0.1            1      71.86%      88.73%    0.089       38
      10        0.1          1.5      71.39%      88.03%    0.096       38
      10        0.3          0.5      71.35%      87.44%    0.112       38
      10        0.3         0.67      71.65%      88.07%    0.098       38
      10        0.3            1      71.82%      88.60%    0.091       38
      10        0.3          1.5      71.35%      88.00%    0.097       38
chọn (tối đa cạnh nới% − 10·RMF): beta_m = 1, beta_frac = 0, accuracy_to_sigma = 0.67; mặc định trong MatchConfig::default(): 1, 0, 0.67

-- 2. độ chính xác theo độ lệch chuẩn nhiễu (bộ báo cáo, lấy mẫu 1 Hz, tương quan 8 s, MatchConfig::default()) --
                                        cạnh chặt  cạnh nới  chiều nới  recall    RMF  sai số vị trí (m)
σ = 3 m: 26146 điểm, RMSE thô 5.53 m
  snap về cạnh gần nhất                   75.11%    96.96%    87.00%    98.89%   0.635      3.04
  HMM + Viterbi offline                   83.65%    99.15%    98.08%    99.97%   0.028      2.58  (gãy chuỗi 27)
σ = 6 m: 26146 điểm, RMSE thô 9.22 m
  snap về cạnh gần nhất                   62.51%    85.58%    74.03%    96.37%   0.991      6.32
  HMM + Viterbi offline                   76.99%    96.44%    93.61%    99.61%   0.053      5.00  (gãy chuỗi 29)
σ = 10 m: 26146 điểm, RMSE thô 14.62 m
  snap về cạnh gần nhất                   49.85%    66.06%    55.16%    93.47%   1.310     10.99
  HMM + Viterbi offline                   69.09%    86.87%    82.96%    98.68%   0.098      8.33  (gãy chuỗi 38)
σ = 15 m: 26146 điểm, RMSE thô 21.57 m
  snap về cạnh gần nhất                   38.71%    48.23%    39.09%    89.47%   1.642     17.34
  HMM + Viterbi offline                   57.33%    71.28%    66.73%    95.67%   0.236     13.17  (gãy chuỗi 92)
σ = 25 m: 26146 điểm, RMSE thô 35.64 m
  snap về cạnh gần nhất                   25.24%    29.75%    23.09%    83.78%   2.229     30.10
  HMM + Viterbi offline                   35.80%    44.55%    40.15%    89.65%   0.742     25.86  (gãy chuỗi 445)

-- 3. lấy mẫu thưa (σ = 6 m): app tiết kiệm pin gửi 5 s hoặc 10 s một điểm --
                                        cạnh chặt  cạnh nới  chiều nới  recall    RMF  sai số vị trí (m)
chu kỳ 1 s: 26146 điểm
  snap về cạnh gần nhất                   62.51%    85.58%    74.03%    96.37%   0.991      6.32
  HMM + Viterbi offline                   76.99%    96.44%    93.61%    99.61%   0.053      5.00  (gãy chuỗi 29)
chu kỳ 5 s: 5247 điểm
  snap về cạnh gần nhất                   62.53%    85.69%    73.89%    84.92%   0.639      6.37
  HMM + Viterbi offline                   76.54%    96.34%    94.13%    99.39%   0.028      5.00  (gãy chuỗi 4)
chu kỳ 10 s: 2636 điểm
  snap về cạnh gần nhất                   63.62%    85.96%    75.38%    70.12%   0.595      6.23
  HMM + Viterbi offline                   75.80%    95.56%    94.23%    99.12%   0.023      5.07  (gãy chuỗi 1)

-- 4. thay đổi từng thành phần của mô hình (σ = 10 m, 1 Hz) --
                                        cạnh chặt  cạnh nới  chiều nới  recall    RMF  sai số vị trí (m)
snap về cạnh gần nhất                     49.85%    66.06%    55.16%    93.47%   1.310     10.99
HMM mặc định                              69.09%    86.87%    82.96%    98.68%   0.098      8.33
HMM trên điểm đã lọc Kalman (LOC-15)      70.21%    87.26%    83.95%    98.42%   0.079      8.22
HMM bỏ accuracy (σ cố định 8 m)           69.26%    87.07%    83.12%    98.87%   0.095      8.31
HMM thêm số hạng hướng đi (σ_h 30°)       71.71%    87.91%    84.56%    99.00%   0.087      8.20
HMM chỉ 3 ứng viên mỗi điểm               61.52%    78.49%    73.72%    95.39%   0.361      9.05
HMM bán kính tìm 25 m (nhỏ hơn nhiễu)     66.86%    84.46%    80.15%    97.89%   0.164      8.55
HMM beta_m = 30 (quá lỏng)                65.54%    84.03%    78.74%    98.15%   0.159      8.68
HMM không phạt đi lùi trên cùng cạnh      69.40%    87.06%    83.30%    98.94%   0.091      8.31

-- 4b. điểm nhảy xa 25–70 m (đa đường) và gãy chuỗi (σ = 10 m, 1 Hz) --
                                        cạnh chặt  cạnh nới  chiều nới  recall    RMF  sai số vị trí (m)
HMM, 0% điểm nhảy xa                      70.55%    87.42%    84.19%    98.17%   0.084      8.10  (gãy chuỗi 15)
snap, 0% điểm nhảy xa                     50.14%    66.16%    55.44%    92.40%   1.297     10.86
HMM, 0.5% điểm nhảy xa                    69.09%    86.87%    82.96%    98.68%   0.098      8.33  (gãy chuỗi 38)
snap, 0.5% điểm nhảy xa                   49.85%    66.06%    55.16%    93.47%   1.310     10.99
HMM, 3% điểm nhảy xa                      63.87%    82.60%    77.89%    96.97%   0.232      9.20  (gãy chuỗi 183)
snap, 3% điểm nhảy xa                     49.81%    65.92%    55.03%    93.14%   1.465     11.83

-- 5. chi phí CPU của HMM offline (1 luồng, σ = 10 m; máy chịu tải: xem uptime, chỉ so sánh tương đối) --
uptime: 23:24  up 35 days, 15:07, 1 user, load averages: 59.03 66.05 72.14
8 ứng viên (mặc định)         9.1 µs/điểm;   11.5 ứng viên/điểm;   6.1 lần Dijkstra/điểm;    24.8 nút duyệt/điểm
3 ứng viên                    7.2 µs/điểm;    4.8 ứng viên/điểm;   2.9 lần Dijkstra/điểm;    11.6 nút duyệt/điểm
snap về cạnh gần nhất: 1.20 µs/điểm (26146 điểm)

-- 6. trích: đoạn trên đường từ tertiary trở lên mà snap gán sai cạnh còn HMM đúng (σ = 10 m; cột: giây, cạnh thật, cạnh snap, cạnh HMM, tọa độ thô, tọa độ HMM) --
t_s,truth_edge,snap_edge,hmm_edge,raw_lat,raw_lng,hmm_lat,hmm_lng,truth_street
391,86296,86294,86296,10.7757321,106.6639107,10.7757211,106.6638813,Thành Thái
392,86293,86291,86293,10.7758715,106.6638406,10.7758657,106.6638252,Thành Thái
393,86290,86292,86290,10.7759670,106.6638219,10.7759549,106.6637904,Thành Thái
394,86290,86287,86290,10.7760339,106.6637626,10.7760327,106.6637595,Thành Thái
395,86290,86287,86290,10.7761090,106.6637192,10.7761123,106.6637279,Thành Thái
396,86289,260123,86289,10.7761557,106.6636489,10.7761763,106.6637025,Thành Thái
397,86289,260123,86289,10.7762078,106.6635608,10.7762507,106.6636730,Thành Thái

uptime lúc kết thúc: 23:24  up 35 days, 15:07, 1 user, load averages: 59.03 66.05 72.14
```

Biến thể chạy trên **fixture** (đồ thị nhỏ, chuyến 0,4–1,5 km), để so với HCM:

```bash
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc16_hmm -- --fixture --trips 20
```

(chỉ chép đầu output, dòng chọn tham số và mục 2; các mục còn lại cùng lệnh, không chép. Mục 2 dùng `MatchConfig::default()` đã chọn trên HCM, không dùng giá trị mà dòng «chọn» của fixture trả về)

```text
== LOC-16: HMM + Viterbi ==
đồ thị: fixtures/khu-mau.json (đồ thị tổng hợp): 234 nút, 840 cạnh có hướng; 20 chuyến mỗi mức nhiễu (dài 2–4,5 km trên HCM, 0,4–1,5 km trên fixture); seed báo cáo 20261106, seed chọn tham số 1106; xe ô tô
uptime lúc bắt đầu: 23:24  up 35 days, 15:07, 1 user, load averages: 57.91 65.70 71.98
...  (mục 1: bảng chọn tham số, không chép)
chọn (tối đa cạnh nới% − 10·RMF): beta_m = 6, beta_frac = 0.3, accuracy_to_sigma = 1.5; mặc định trong MatchConfig::default(): 1, 0, 0.67

-- 2. độ chính xác theo độ lệch chuẩn nhiễu (bộ báo cáo, lấy mẫu 1 Hz, tương quan 8 s, MatchConfig::default()) --
                                        cạnh chặt  cạnh nới  chiều nới  recall    RMF  sai số vị trí (m)
σ = 3 m: 3710 điểm, RMSE thô 5.67 m
  snap về cạnh gần nhất                   83.85%    99.57%    91.40%   100.00%   0.721      2.80
  HMM + Viterbi offline                   89.70%    99.16%    99.03%   100.00%   0.109      2.56  (gãy chuỗi 11)
σ = 6 m: 3710 điểm, RMSE thô 9.26 m
  snap về cạnh gần nhất                   78.41%    95.39%    85.55%    99.81%   0.900      5.59
  HMM + Viterbi offline                   87.04%    98.27%    98.06%   100.00%   0.147      4.82  (gãy chuỗi 11)
σ = 10 m: 3710 điểm, RMSE thô 14.59 m
  snap về cạnh gần nhất                   71.59%    83.91%    71.91%    99.62%   1.057      9.71
  HMM + Viterbi offline                   84.69%    94.12%    93.72%   100.00%   0.136      7.80  (gãy chuỗi 10)
σ = 15 m: 3710 điểm, RMSE thô 21.47 m
  snap về cạnh gần nhất                   63.13%    70.43%    57.25%    99.42%   1.180     15.39
  HMM + Viterbi offline                   81.37%    89.33%    88.76%   100.00%   0.150     11.47  (gãy chuỗi 11)
σ = 25 m: 3710 điểm, RMSE thô 35.43 m
  snap về cạnh gần nhất                   48.25%    51.21%    37.60%    99.07%   1.470     27.91
  HMM + Viterbi offline                   60.19%    66.79%    64.50%    96.22%   0.465     23.18  (gãy chuỗi 61)
```

Điều nó cho thấy:

- HMM thắng snap ở **mọi** mức nhiễu trên HCM, và khoảng cách rộng ra khi nhiễu tăng. σ = 10 m: đúng cạnh (nới nút) 86,87% so với 66,06%; (chặt) 69,09% so với 49,85%; recall lộ trình 98,68% so với 93,47%; **RMF 0,098 so với 1,310** (lộ trình của snap sai nhiều hơn độ dài đúng). σ = 25 m cả hai đều tệ (44,55% so với 29,75%; RMF 0,742 so với 2,229), HMM không cứu được nhiễu quá lớn.
- Thước chặt thấp ngay cả ở nhiễu nhỏ (HMM σ = 3 m: 83,65% chặt nhưng 99,15% nới), vì cạnh HCM ngắn và điểm thật nằm gần nút; vì vậy bài nên báo «cạnh nới nút» và «RMF», không báo «cạnh chặt» đứng một mình.
- **Một chỗ snap không thua:** trên fixture ở σ = 3 m, snap có cạnh nới 99,57% so với HMM 99,16% (HMM vẫn hơn ở thước chặt, chiều và RMF: 89,70% / 99,03% / 0,109 so với 83,85% / 91,40% / 0,721). Khi nhiễu nhỏ hơn khoảng cách giữa các đường thì snap đủ tốt cho việc «đang ở đường nào», chỉ thua ở **chiều đi** và **lộ trình**.
- Tham số **không mang sang đồ thị khác**: trên HCM bảng chọn tham số chọn `beta_m = 1`, trên fixture chọn `beta_m = 6, beta_frac = 0,3, accuracy_to_sigma = 1,5` (dòng «chọn» ở khối fixture). Mặc định trong mã là bản chọn trên HCM.
- Lấy mẫu thưa là chỗ snap hỏng nhất: ở 10 s/điểm snap chỉ có 70,12% recall lộ trình (bỏ sót cạnh vì không nối khoảng trống), RMF 0,595; HMM 99,12% và 0,023.
- Bỏ từng thành phần (σ = 10 m, cạnh nới): chỉ 3 ứng viên mất 8,4 điểm (78,49%); bán kính 25 m nhỏ hơn nhiễu mất 2,4; `beta_m = 30` mất 2,8; thêm số hạng hướng đi **được** 1,0 điểm (87,91%); chạy trên điểm đã lọc Kalman được 0,4 điểm (87,26%); bỏ accuracy gần như không đổi (87,07%).
- Một lỗi đã gặp và sửa: bản đầu cho phép «đi lùi dưới 12 m trên cùng cạnh» với quãng đường 0, nên đi ngược chiều cạnh rẻ gần bằng đổi sang cạnh `twin`. Sau khi sửa (phạt `1/backward_beta_m` nat mỗi mét lùi) hàng «không phạt đi lùi» cho 87,06% so với 86,87%: hiệu ứng nhỏ trên tập này; lỗi phát hiện nhờ soi một chỗ gãy chuỗi ở σ = 3 m, không nhờ số tổng.
- Điểm nhảy xa là điểm yếu thật: mỗi điểm nhảy làm gãy chuỗi khoảng hai lần (gãy 15 → 38 → 183 khi tỉ lệ nhảy 0 → 0,5% → 3%) và kéo cạnh nới từ 87,42% xuống 86,87% và 82,60%; snap gần như không đổi (66,16% → 65,92%) vì vốn đã sai nhiều. Chưa có cơ chế «bỏ điểm nhảy rồi nối tiếp» (xem mục chưa làm).
- Chi phí: 9,1 µs mỗi điểm cho HMM offline (11,5 ứng viên, 6,1 lần Dijkstra, 24,8 nút duyệt mỗi điểm) so với 1,20 µs cho snap; 3 ứng viên còn 7,2 µs. Đo khi tải 59 (xem `uptime` trong output).

Dữ liệu cho hình SVG: mục 6 ở output HCM là 7 điểm liên tiếp trên đường Thành Thái (σ = 10 m): cạnh thật, cạnh snap, cạnh HMM, tọa độ thô và tọa độ đã bám; snap gán sai cạnh ở cả 7 điểm (thước chặt), HMM đúng cả 7. Mã cạnh tra được trong `data/hcm-graph.bin`.

### LOC-17: bám đường online, ca khó, mất tín hiệu, ngân sách CPU

Lệnh:

```bash
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc17_online -- --reps 5
```

Cấu hình: 40 chuyến thường (σ = 8 m, 26.146 điểm) cho mục 1; mục 2: tối đa 12 ca khó mỗi loại ở chỉ số lẻ của danh mục × 5 lần nhiễu (σ = 6 m + lệch 12 m quanh điểm khó), chấm các điểm cách điểm khó ≤ 45 m
và lộ trình cả vết; mục 3: 400 tài xế xen kẽ từng điểm, 1 luồng; `MatchConfig::default()`. Chưa dùng ca chỉ số chẵn cho bất cứ việc gì (dành cho chọn tham số về sau).

```text
== LOC-17: bám đường online ==
đồ thị: data/hcm-graph.bin: 126699 nút, 319074 cạnh có hướng; 40 chuyến thường, tối đa 12 ca khó mỗi loại × 5 lần nhiễu; seed báo cáo 20261106; tham số HMM = MatchConfig::default() (đã chọn ở LOC-16)
uptime lúc bắt đầu: 23:24  up 35 days, 15:07, 1 user, load averages: 57.91 65.70 71.98
lưu ý: nhiễu GPS do chính lab sinh ra (có seed); mọi độ chính xác dưới đây là trên nhiễu tổng hợp này, không phải GPS thật.

-- 0. ca khó tìm được trên đồ thị (quét hình học và thẻ, 0.5 s) --
cầu vượt cắt đường ô tô bên dưới: 729; cặp đường song song (6–35 m, ≥100 m, cùng hạng): 2162; cặp đường chính + đường gom: 131; hầm cho ô tô (≥80 m): 22
  cầu vượt: cạnh 13534 (Cầu Kênh Ngang số 3) trên cạnh 17433 (Mễ Cốc) tại 10.725457,106.631132
  cầu vượt: cạnh 37771 (Cầu Phú Mỹ) trên cạnh 53317 (Nguyễn Văn Linh) tại 10.733679,106.724702
  cầu vượt: cạnh 58826 (Cầu Chánh Hưng) trên cạnh 54085 (Đường Phạm Hùng) tại 10.745554,106.668964
  cầu vượt: cạnh 107171 (Cầu vượt Lăng Cha Cả) trên cạnh 61278 (Cộng Hòa) tại 10.800745,106.660900
  cầu vượt: cạnh 261756 (Cầu số 19) trên cạnh 98248 (Mai Chí Thọ) tại 10.770973,106.713767
  Parallel: cạnh 0 (Đường Lê Đức Anh) và cạnh 71552 (Đường Lê Đức Anh), chạy chung 300 m, cách trung bình 7.5 m, ngược chiều
  Parallel: cạnh 43048 (Hẻm 199 Lê Quang Định) và cạnh 198456 (Hẻm 199/24 Lê Quang Định), chạy chung 108 m, cách trung bình 26.4 m, cùng chiều
  Parallel: cạnh 78623 (Hẻm 377 Cách mạng tháng 8) và cạnh 78628 ((không tên)), chạy chung 108 m, cách trung bình 27.0 m, cùng chiều
  Parallel: cạnh 126233 (Trần Bạch Đằng) và cạnh 261706 (Trần Bạch Đằng), chạy chung 144 m, cách trung bình 11.4 m, cùng chiều
  Parallel: cạnh 200322 (Đường Lê Đức Anh) và cạnh 200329 (Đường Lê Đức Anh), chạy chung 108 m, cách trung bình 27.6 m, cùng chiều

-- 1. độ trễ quyết định (chuyến thường, σ = 8 m, 1 Hz, 26146 điểm): chờ thêm L điểm thì chốt chắc hơn --
                               cạnh chặt  cạnh nới  chiều nới  recall    RMF   số điểm   %sửa  %đầu-sai  trễ p50/p95 (s)
snap về cạnh gần nhất             55.76%    75.73%    64.25%    94.85%   1.152    26146
online, trễ 0 điểm                55.76%    75.73%    64.25%    99.74%   2.683    26146    0.0%     0.0%    0.0 /  0.0
online, trễ 1 điểm                61.17%    77.23%    69.97%    97.16%   0.572    26146    7.2%     5.7%    1.0 /  1.0
online, trễ 2 điểm                64.69%    81.71%    73.62%    97.66%   0.504    26146   11.2%     9.3%    2.0 /  2.0
online, trễ 3 điểm                66.40%    84.19%    76.05%    97.46%   0.454    26146   13.7%    11.5%    3.0 /  3.0
online, trễ 5 điểm                69.18%    86.85%    79.77%    98.30%   0.290    26146   16.9%    14.0%    5.0 /  5.0
online, trễ 8 điểm                70.09%    88.68%    81.69%    98.37%   0.219    26146   20.1%    16.3%    8.0 /  9.0
online, trễ 12 điểm               70.79%    89.87%    83.58%    98.38%   0.195    26146   23.4%    19.0%   12.0 / 13.0
online, trễ 20 điểm               71.51%    90.87%    85.67%    98.63%   0.177    26146   26.7%    21.4%   20.0 / 21.0
offline (nhìn cả chuyến)          73.47%    92.53%    88.84%    99.20%   0.073    26146
(%sửa: điểm có ước lượng tạm đổi cạnh ít nhất một lần trước khi chốt; %đầu-sai: cạnh gán lần đầu khác cạnh chốt)

-- 2. ca khó (tập báo cáo: chỉ số lẻ trong danh mục; tập chẵn dành cho chọn tham số và chưa dùng; mỗi ca 5 lần nhiễu σ = 6 m + vùng lệch 12 m quanh điểm khó; chấm các điểm trong 45 m quanh điểm khó, lộ trình chấm cả vết) --
                               cạnh chặt  cạnh nới  chiều nới  recall    RMF   số điểm
chạy trên cầu vượt (12 ca × 5 nhiễu):
  snap về cạnh gần nhất           25.44%    26.60%    26.02%    97.79%   1.702      688
  online, trễ 0 (tham lam)        25.44%    26.60%    26.02%    99.77%   5.864      688
  online, trễ 3 điểm              57.41%    58.58%    57.85%    91.59%   1.019      688
  online, trễ 8 điểm              58.28%    60.47%    59.01%    92.15%   0.760      688
  offline                         75.87%    76.60%    76.60%    90.67%   0.293      688  (gãy chuỗi 27)
chạy dưới cầu vượt (12 ca × 5 nhiễu):
  snap về cạnh gần nhất           24.86%    35.77%    26.45%    90.19%   2.148     1384
  online, trễ 0 (tham lam)        24.78%    35.77%    26.37%    97.92%   7.427     1384
  online, trễ 3 điểm              39.09%    54.55%    49.13%    86.53%   1.933     1384
  online, trễ 8 điểm              43.50%    57.23%    53.25%    90.52%   1.351     1384
  offline                         46.53%    61.27%    57.73%    90.05%   0.342     1384  (gãy chuỗi 41)
hai đường song song (12 ca × 5 nhiễu):
  snap về cạnh gần nhất           53.90%    61.29%    52.64%    97.06%   1.164      948
  online, trễ 0 (tham lam)        53.90%    61.29%    52.64%    99.86%   3.922      948
  online, trễ 3 điểm              73.84%    81.65%    79.01%    94.46%   0.580      948
  online, trễ 8 điểm              79.64%    86.81%    84.28%    97.40%   0.312      948
  offline                         85.02%    92.30%    89.87%    99.91%   0.112      948  (gãy chuỗi 17)
đường chính, có đường gom bên cạnh (12 ca × 5 nhiễu):
  snap về cạnh gần nhất           44.26%    55.14%    49.55%    98.34%   1.421      662
  online, trễ 0 (tham lam)        44.11%    55.14%    49.55%    99.77%   3.953      662
  online, trễ 3 điểm              72.81%    81.57%    79.15%    94.79%   0.793      662
  online, trễ 8 điểm              68.13%    76.89%    76.59%    95.60%   0.549      662
  offline                         66.31%    76.28%    75.83%    94.64%   0.285      662  (gãy chuỗi 8)
đường gom bên cạnh đường chính (12 ca × 5 nhiễu):
  snap về cạnh gần nhất           59.00%    65.21%    63.84%    97.79%   1.183     1239
  online, trễ 0 (tham lam)        58.92%    65.13%    63.76%    99.65%   3.855     1239
  online, trễ 3 điểm              68.28%    78.69%    72.07%    94.64%   0.406     1239
  online, trễ 8 điểm              76.43%    83.86%    81.03%    97.94%   0.215     1239
  offline                         74.90%    81.28%    80.15%    97.45%   0.159     1239  (gãy chuỗi 14)
hầm (mất tín hiệu) (11 ca × 5 nhiễu):
  snap về cạnh gần nhất           36.16%    59.82%    50.00%    68.61%   1.242      224
  online, trễ 0 (tham lam)        35.27%    59.82%    50.00%    87.36%   3.958      224
  online, trễ 3 điểm              54.91%    66.96%    65.62%    76.55%   0.630      224
  online, trễ 8 điểm              46.43%    67.86%    64.73%    81.31%   0.490      224
  offline                         58.93%    78.12%    78.12%    71.55%   0.437      224  (gãy chuỗi 18)
  offline, gap_break_s = 300      58.93%    78.12%    78.12%    76.70%   0.386      224  (gãy chuỗi 8; độ nhạy, không dùng chọn tham số)
mất tín hiệu 25 giây (12 ca × 5 nhiễu):
  snap về cạnh gần nhất           66.67%    89.34%    77.54%    95.00%   0.861    38855
  online, trễ 0 (tham lam)        66.67%    89.34%    77.54%    99.84%   1.757    38855
  online, trễ 3 điểm              73.10%    91.58%    82.62%    98.79%   0.181    38855
  online, trễ 8 điểm              75.69%    94.21%    86.99%    99.03%   0.139    38855
  offline                         78.38%    96.22%    94.21%    98.81%   0.053    38855  (gãy chuỗi 38)

-- 2b. nội suy trong khoảng mất tín hiệu (≥ 6 s không có điểm), kết quả online trễ 8 điểm --
loại                         số khoảng    số điểm sai số nội suy theo đường p50/p95/max (m) sai số nối thẳng p50/p95/max (m)
hầm (mất tín hiệu)                38       2161        7.8 / 153.7 / 252.4       32.6 / 202.2 / 234.0
mất tín hiệu 25 giây              59       1477          6.4 / 47.1 / 97.2         10.0 / 40.5 / 85.3

-- 3. ngân sách CPU theo tài xế (online, 400 tài xế xen kẽ từng điểm, σ = 8 m, 1 luồng) --
uptime: 23:24  up 35 days, 15:08, 1 user, load averages: 67.87 67.34 72.42
     trễ  max_settled      µs/điểm   p50 (µs)   p99 (µs) tối đa (µs)   cạnh nới    chạm trần bộ nhớ/xe (TB, tối đa)
       0         4000        35.01       7.12      53.96     36573.9     76.75%            0       504 B, 504 B
       3         4000        53.96      10.83     131.50     75052.5     83.24%            0      2785 B, 3072 B
       8         4000        39.87      12.25      98.58     41500.9     88.25%            0      7850 B, 8696 B
      20         4000        21.31      15.29      96.46     45994.1     90.58%            0     18420 B, 20760 B
       3            4        18.10      10.04      65.08     56617.2     83.25%       651362      2785 B, 3072 B
       3            8        13.28      10.46      38.08     15908.7     83.24%       108392      2785 B, 3072 B
       3           16        34.80      10.92      85.96     34330.6     83.24%          734      2785 B, 3072 B
       3           50        15.46      10.42      38.96     37907.0     83.24%            0      2785 B, 3072 B
       3          200        13.77      10.08      30.50     29118.1     83.24%            0      2785 B, 3072 B
tải tham chiếu của Tới Nơi: 25.000 tài xế, mỗi 4 giây một điểm = 6.250 điểm/giây (kiến trúc §14, LOC-09).
  trễ 0 điểm, max_settled 4000: 6.250 điểm/s × 35.01 µs = 0.2188 lõi; 25.000 tài xế × 504 B = 12.6 MB
  trễ 3 điểm, max_settled 4000: 6.250 điểm/s × 53.96 µs = 0.3372 lõi; 25.000 tài xế × 2785 B = 69.6 MB
  trễ 8 điểm, max_settled 4000: 6.250 điểm/s × 39.87 µs = 0.2492 lõi; 25.000 tài xế × 7850 B = 196.2 MB
  trễ 20 điểm, max_settled 4000: 6.250 điểm/s × 21.31 µs = 0.1332 lõi; 25.000 tài xế × 18420 B = 460.5 MB

-- 4. một cầu vượt cụ thể: chạy trên cầu vượt bằng ô tô, 20 lần nhiễu σ = 6 m + lệch 12 m quanh giao điểm (minh họa một ca, không phải thống kê) --
số giao cắt có tên cầu bắt đầu bằng «Cầu vượt»: 246
cạnh 28226 (Cầu vượt An Sương) trên cạnh 44669 ((không tên))
cầu vượt: 10.840876,106.613380 10.842339,106.614989 10.844280,106.617178 10.844444,106.617362
đường dưới: 10.843400,106.615892 10.843328,106.615894 10.843256,106.615921 10.843194,106.615977 10.843178,106.616047 10.843180,106.616120
giao điểm: 10.843216,106.615978
                               cạnh chặt  cạnh nới  chiều nới  recall    RMF   số điểm
snap về cạnh gần nhất             32.35%    32.35%    32.35%    98.35%   1.655      204
online, trễ 3 điểm                86.27%    86.27%    86.27%    91.62%   0.534      204
online, trễ 8 điểm                90.20%    90.20%    90.20%    93.12%   0.430      204
offline                          100.00%   100.00%   100.00%    98.41%   0.131      204
lần nhiễu đầu tiên; cạnh cầu = 28226, cạnh đường dưới = 44669
t_s,truth_edge,snap_edge,online3_edge,offline_edge,raw_lat,raw_lng,online3_lat,online3_lng,truth_lat,truth_lng
75,28226,266030,28226,28226,10.8425464,106.6153786,10.8426223,106.6153088,10.8426020,106.6152860
76,28226,161685,28226,28226,10.8426250,106.6154466,10.8426908,106.6153861,10.8426574,106.6153484
77,28226,161685,28226,28226,10.8426968,106.6154822,10.8427404,106.6154420,10.8427127,106.6154108
78,28226,28226,28226,28226,10.8428019,106.6155039,10.8427983,106.6155072,10.8427681,106.6154731
79,28226,28226,28226,28226,10.8428532,106.6155785,10.8428578,106.6155743,10.8428234,106.6155355
80,28226,28226,28226,28226,10.8429111,106.6156214,10.8429048,106.6156272,10.8428787,106.6155979
81,28226,28226,28226,28226,10.8429562,106.6156602,10.8429440,106.6156715,10.8429341,106.6156603
82,28226,75529,28226,28226,10.8430256,106.6156009,10.8429462,106.6156739,10.8429894,106.6157227
83,28226,75529,28226,28226,10.8431223,106.6157051,10.8430406,106.6157803,10.8430448,106.6157851
84,28226,75529,28226,28226,10.8432297,106.6157826,10.8431267,106.6158773,10.8431001,106.6158474
86,28226,44671,28226,28226,10.8433379,106.6158635,10.8432148,106.6159767,10.8432108,106.6159722
87,28226,44669,28226,28226,10.8433864,106.6159028,10.8432558,106.6160229,10.8432662,106.6160346
88,28226,44666,28226,28226,10.8434556,106.6160008,10.8433348,106.6161119,10.8433215,106.6160970
89,28226,75529,28226,28226,10.8434595,106.6160671,10.8433689,106.6161504,10.8433768,106.6161594
90,28226,75529,28226,28226,10.8435178,106.6161127,10.8434174,106.6162051,10.8434322,106.6162217
91,28226,44666,28226,28226,10.8435713,106.6161366,10.8434531,106.6162453,10.8434875,106.6162841
92,28226,75529,28226,28226,10.8435808,106.6162786,10.8435268,106.6163283,10.8435429,106.6163465
93,28226,75529,28226,28226,10.8436332,106.6163442,10.8435823,106.6163910,10.8435982,106.6164089
94,28226,28226,28226,28226,10.8436606,106.6164081,10.8436259,106.6164401,10.8436536,106.6164713
95,28226,28226,28226,28226,10.8437254,106.6165293,10.8437142,106.6165396,10.8437089,106.6165337
96,28226,28226,28226,28226,10.8437809,106.6165997,10.8437735,106.6166065,10.8437643,106.6165960

uptime lúc kết thúc: 23:26  up 35 days, 15:09, 1 user, load averages: 115.83 83.25 78.10
```

Chạy lại riêng mục 3 hai lần nữa (`--trips 5 --per-kind 2 --reps 1`; mục 3 không phụ thuộc ba tham số này) để thấy mức nhiễu do tải máy; dòng `uptime` ở cuối mỗi khối là của lệnh `uptime` chạy ngay sau lab.

Lần 2:

```text
-- 3. ngân sách CPU theo tài xế (online, 400 tài xế xen kẽ từng điểm, σ = 8 m, 1 luồng) --
uptime: 23:31  up 35 days, 15:15, 1 user, load averages: 166.28 113.78 93.15
     trễ  max_settled      µs/điểm   p50 (µs)   p99 (µs) tối đa (µs)   cạnh nới    chạm trần bộ nhớ/xe (TB, tối đa)
       0         4000        36.07       7.08      64.08     87063.7     76.75%            0       504 B, 504 B
       3         4000        33.00      11.00     109.79    102582.3     83.24%            0      2785 B, 3072 B
       8         4000        41.51      12.88     194.08     90691.1     88.25%            0      7850 B, 8696 B
      20         4000        26.21      15.25      78.38     30871.3     90.58%            0     18420 B, 20760 B
       3            4        13.22      10.75      68.08      3772.4     83.25%       651362      2785 B, 3072 B
       3            8        11.80      10.42      35.12      3371.5     83.24%       108392      2785 B, 3072 B
       3           16        15.87      11.38     110.08      6218.9     83.24%          734      2785 B, 3072 B
       3           50        13.80      11.04      67.92      3579.8     83.24%            0      2785 B, 3072 B
       3          200        10.91       9.88      29.29       892.6     83.24%            0      2785 B, 3072 B
tải tham chiếu của Tới Nơi: 25.000 tài xế, mỗi 4 giây một điểm = 6.250 điểm/giây (kiến trúc §14, LOC-09).
  trễ 0 điểm, max_settled 4000: 6.250 điểm/s × 36.07 µs = 0.2255 lõi; 25.000 tài xế × 504 B = 12.6 MB
  trễ 3 điểm, max_settled 4000: 6.250 điểm/s × 33.00 µs = 0.2063 lõi; 25.000 tài xế × 2785 B = 69.6 MB
  trễ 8 điểm, max_settled 4000: 6.250 điểm/s × 41.51 µs = 0.2595 lõi; 25.000 tài xế × 7850 B = 196.2 MB
  trễ 20 điểm, max_settled 4000: 6.250 điểm/s × 26.21 µs = 0.1638 lõi; 25.000 tài xế × 18420 B = 460.5 MB

23:32  up 35 days, 15:16, 1 user, load averages: 180.93 128.52 100.03
```

Lần 3:

```text
-- 3. ngân sách CPU theo tài xế (online, 400 tài xế xen kẽ từng điểm, σ = 8 m, 1 luồng) --
uptime: 23:32  up 35 days, 15:16, 1 user, load averages: 169.09 126.93 99.63
     trễ  max_settled      µs/điểm   p50 (µs)   p99 (µs) tối đa (µs)   cạnh nới    chạm trần bộ nhớ/xe (TB, tối đa)
       0         4000         7.28       6.25      20.08     19055.6     76.75%            0       504 B, 504 B
       3         4000        11.50      10.25      27.67      5008.0     83.24%            0      2785 B, 3072 B
       8         4000        12.35      11.38      27.42     28612.2     88.25%            0      7850 B, 8696 B
      20         4000        15.42      13.96      33.38     18429.5     90.58%            0     18420 B, 20760 B
       3            4        11.80       9.67      28.92     12073.3     83.25%       651362      2785 B, 3072 B
       3            8        17.91      10.33      43.17     54404.6     83.24%       108392      2785 B, 3072 B
       3           16        12.35      10.17      30.38     16987.3     83.24%          734      2785 B, 3072 B
       3           50        11.42       9.88      27.38      7612.0     83.24%            0      2785 B, 3072 B
       3          200        19.36      10.54      46.25     36469.5     83.24%            0      2785 B, 3072 B
tải tham chiếu của Tới Nơi: 25.000 tài xế, mỗi 4 giây một điểm = 6.250 điểm/giây (kiến trúc §14, LOC-09).
  trễ 0 điểm, max_settled 4000: 6.250 điểm/s × 7.28 µs = 0.0455 lõi; 25.000 tài xế × 504 B = 12.6 MB
  trễ 3 điểm, max_settled 4000: 6.250 điểm/s × 11.50 µs = 0.0719 lõi; 25.000 tài xế × 2785 B = 69.6 MB
  trễ 8 điểm, max_settled 4000: 6.250 điểm/s × 12.35 µs = 0.0772 lõi; 25.000 tài xế × 7850 B = 196.2 MB
  trễ 20 điểm, max_settled 4000: 6.250 điểm/s × 15.42 µs = 0.0964 lõi; 25.000 tài xế × 18420 B = 460.5 MB

23:33  up 35 days, 15:16, 1 user, load averages: 117.28 118.26 97.43
```

Và một lần chạy lúc phát triển (lệnh như lần chính, `uptime` của lab: tải 39,78 lúc 23:14), không có file, chép từ màn hình:

```text
     trễ  max_settled      µs/điểm   p50 (µs)   p99 (µs) tối đa (µs)   cạnh nới    chạm trần bộ nhớ/xe (TB, tối đa)
       0         4000         7.59       6.29      24.00      4088.9     76.75%            0       504 B, 504 B
       3         4000        11.37       9.50      29.62     11602.0     83.24%            0      2785 B, 3072 B
       8         4000        11.54      10.42      28.12     12149.1     88.25%            0      7850 B, 8696 B
      20         4000        13.71      12.58      31.83      5516.8     90.58%            0     18420 B, 20760 B
```

Điều nó cho thấy:

- **Độ trễ quyết định đổi lấy độ chính xác và lộ trình sạch.** Trễ 0 trong mã thực chất là snap: lớp vừa dựng được chốt ngay nên cửa sổ rỗng, `push` gọi `extend` với `prev = None` và chỉ còn điểm số phát; vì vậy cạnh gán y hệt snap (75,73% nới, 55,76% chặt). **Lộ trình tệ hơn snap** (RMF 2,683 so với 1,152) chỉ vì cách dựng: `reconstruct_route` nối hai điểm liên tiếp bằng đường ngắn nhất, còn snap không lấp khoảng trống. Bản tham lam thật (nối tiếp từ trạng thái vừa chốt) chưa đo. Trễ 3 giây: 84,19% nới, RMF 0,454; trễ 8 giây: 88,68%, 0,219; trễ 20 giây: 90,87%, 0,177; offline 92,53%, 0,073. Phần lớn lợi ích có trong 5–8 giây đầu; sau đó tăng chậm.
- Ước lượng tạm bị sửa: ở trễ 3 điểm, 13,7% điểm bị đổi cạnh ít nhất một lần trước khi chốt, và 11,5% có cạnh gán lần đầu khác cạnh chốt. Với app, đó là lý do hiển thị vị trí bám đường cho khách phải dùng điểm đã chốt hoặc chấp nhận nhảy lại.
- Ca khó, kết quả «cạnh nới» (snap → online trễ 3 → online trễ 8 → offline): chạy trên cầu vượt 26,60% → 58,58% → 60,47% → 76,60%; hai đường song song 61,29% → 81,65% → 86,81% → 92,30%; đường gom bên đường chính 65,21% → 78,69% → 83,86% → 81,28%.
- **Không phải ca nào offline cũng hơn online:** đường chính có đường gom bên cạnh: offline 76,28% **thấp hơn** online trễ 3 (81,57%); hầm: online trễ 3 chặt 54,91% so với trễ 8 46,43%. Offline tối ưu xác suất của mô hình, không tối ưu độ chính xác; khi lệch ổn định 12 m đẩy cả chuỗi về phía đường kia thì đường đi «xác suất nhất» là đường sai.
- Ca khó nhất là **chạy dưới cầu vượt** (không phải trên): offline chỉ đạt 61,27% nới (snap 35,77%). Tôi chưa kiểm nguyên nhân; hai giả thuyết: điểm lệch nằm gần cầu hơn gần đường dưới, và xe còn dừng ở đèn dưới chân cầu (khoảng 22% số điểm của các chuyến thường ở LOC-15 là lúc xe dừng: 5.686 trên 25.988 điểm đã phân pha). Bài phải nói rõ cầu vượt khó theo chiều nào và không khẳng định nguyên nhân.
- Mất tín hiệu: hầm nằm trong 22 hầm của đồ thị HCM (≥ 80 m, ô tô); chuỗi gãy khi khoảng trống dài hơn `gap_break_s = 90` s, nên recall lộ trình offline 71,55%; với `gap_break_s = 300` (chỉ để biết độ nhạy) là 76,70%, gãy 18 → 8. Nội suy dọc đường đi giữa hai điểm đã bám sai số p50 7,8 m (hầm) và 6,4 m (25 giây) so với 32,6 m và 10,0 m khi nối thẳng hai điểm thô; p95 153,7 m so với 202,2 m ở hầm, nghĩa là hầm dài vẫn lệch cả trăm mét. Mất tín hiệu 25 giây trên chuyến thường: online trễ 8 đạt 94,21% nới, RMF 0,139.
- Ngân sách CPU: p50 của một lần `push` ổn định 9,5–11,0 µs ở trễ 3 qua các lần chạy; trung bình 7,3–54 µs tùy tải máy (`tối đa` tới hơn 100 ms là nhiễu hệ điều hành lúc tải 166, không phải thuật toán). Ở lần chạy nhanh nhất 11,37 µs/điểm cho trễ 3 ⇒ 6.250 điểm/s × 11,37 µs = **0,071 lõi** cho 25.000 tài xế; bộ nhớ **2.785 B mỗi tài xế** (3.072 B tối đa) ⇒ 69,6 MB; trễ 20 điểm là 18.420 B ⇒ 460,5 MB. Độ trễ quyết định tính bằng điểm trả bằng bộ nhớ gần tuyến tính.
- Trần `max_settled` **không phải nút thắt** ở mức nhiễu này: ngay cả 4 nút mỗi lần Dijkstra (chạm trần 651.362 lần) cho cạnh nới 83,25% so với 83,24% ở 4000. Chưa thử nhiễu lớn hơn hay đồ thị thưa hơn.
- Cầu vượt An Sương (mục 4, một ca, không phải thống kê): snap 32,35% (cạnh nới), online trễ 3 là 86,27%, trễ 8 là 90,20%, offline 100,00%.

Dữ liệu cho hình SVG (mục 4): đường dẫn hình học của cầu vượt (`cầu vượt:`) và đường dưới (`đường dưới:`) là các cặp `vĩ độ,kinh độ` cách nhau bằng dấu cách; giao điểm `10.843216,106.615978`; 21 điểm GPS của lần nhiễu đầu với cạnh snap, cạnh online trễ 3, cạnh offline, tọa độ thô, tọa độ online và tọa độ thật. Cạnh cầu là 28226, cạnh đường dưới là 44669; snap gán sang cạnh khác cạnh thật ở 14 trong 21 điểm.

### LOC-18: kho vị trí mới nhất (H3 trong bộ nhớ, TTL, Redis, Kafka)

Lệnh (có Redis và Kafka thật; bỏ `--redis --kafka` để chỉ chạy mục 1–5 trong bộ nhớ, không cần mạng):

```bash
cargo run -q --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p labs --bin loc18_kho_vi_tri -- --redis --kafka
```

Cấu hình: 25.000 tài xế tổng hợp (6.250 điểm/giây, tải tham chiếu của Tới Nơi), H3 độ phân giải 9, TTL 30 s, 64 mảnh khóa, seed 20261106; Redis 7.4.12 và Kafka 3 partition ở cổng lab (xem bảng cấu hình).

```text
== LOC-18: kho vị trí mới nhất ==
25000 tài xế tổng hợp (60% quanh trung tâm HCM, 40% rải khắp bbox), mỗi 4 giây một điểm = 6250 điểm/giây; H3 độ phân giải 9, TTL 30 s; seed 20261106
lưu ý: dữ liệu do lab sinh; thời gian đo khi máy đang chịu tải nặng, chỉ so sánh tương đối trong cùng một lần chạy.
uptime lúc bắt đầu: 23:27  up 35 days, 15:10, 1 user, load averages: 90.25 85.49 79.63

-- 1. cập nhật một luồng (apply) --
chèn 25000 tài xế mới: 0.60 µs/lần (1668613/s), 2868 ô H3 có tài xế
cập nhật 200000 lần (kể cả sinh vị trí mới): 0.73 µs/lần; đổi ô H3 21563 lần = 10.8% số cập nhật; bỏ vì cũ 0
điểm cũ đến sau bị bỏ: 0.30 µs/lần

-- 2. truy vấn gần nhất (k = 5, bán kính tối đa 2 km, một luồng) --
20000 truy vấn quanh trung tâm: p50 5.25 µs, p95 6.21 µs, p99 7.75 µs; trung bình 5.00 tài xế/truy vấn
đối chiếu với vét cạn trên 25000 vị trí: 0/200 truy vấn khác kết quả

-- 3. đọc nhiều ghi nhiều: mỗi phép đo 2 s, 25000 tài xế; ghi là cập nhật theo vòng, đọc là truy vấn k = 5 liên tục --
uptime trước phép đo đồng thời: 23:27  up 35 days, 15:10, 1 user, load averages: 90.25 85.49 79.63
mảnh khóa luồng ghi luồng đọc ghi (cập nhật/s) đọc (truy vấn/s)   đọc p50 µs   đọc p99 µs
       1        1        0          1613576                0            -            -
      64        1        0          1555612                0            -            -
       1        4        0           723217                0            -            -
      64        4        0          1967088                0            -            -
       1        1        4            73304           138899          5.7        107.9
      64        1        4           221557           154898          5.8         93.8
       1        4        4           174080           188216          5.8        282.2
      64        4        4           679723            93112          6.0        276.2

-- 4. TTL cho tài xế im lặng (TTL 30 s) --
25000 tài xế, 10% còn gửi: trước khi dọn, truy vấn bán kính 30 km tại t = 41 s thấy 2500 (đã bỏ qua tài xế quá TTL); sweep xóa 22500 tài xế trong 1.9 ms; còn trong bộ nhớ 2500

-- 5. giảm mẫu sang lịch sử (chính sách như history-writer của Go: 60 m, rẽ 30° sau 4 s, nhịp 60 s; điểm có cờ luôn giữ) --
60 chuyến, 10101 điểm (4 s/điểm, nhiễu σ 6 m): giữ 3400 = 33.7% (đầu 60, di chuyển 2483, rẽ 848, nhịp 9, cờ 0)

-- 6. Redis làm bản phụ (redis://127.0.0.1:6391/, tiền tố khóa tn:loc18:26623) --
uptime trước phép đo Redis: 23:27  up 35 days, 15:11, 1 user, load averages: 93.53 86.44 80.10
ghi 25000 tài xế qua MirrorWriter (gộp theo tài xế, lô 500): 1.25 s = 20011 tài xế/s; 50 lô, 25000 đã ghi, 0 bị bỏ vì hàng đợi đầy, 0 lỗi
Redis có 25000 thành viên trong sorted set GEO
gộp theo tài xế: 5.000 cập nhật cho 1.000 tài xế trong một lần xả chỉ ghi 1000 lệnh
khởi động lại, nạp từ Redis: đọc 25000 bản ghi, nạp 25000, quá hạn TTL 0, hỏng 0: 46.0 ms; kho có 25000 tài xế, bất biến đúng
bản sao khớp GEOSEARCH của Redis trên 100 truy vấn k = 5: 100/100 cùng tập tài xế

-- 7. Kafka (127.0.0.1:9096, topic lab location.updates.v1.lab18-26623, 3 partition, key driver_id) --
uptime trước phép đo Kafka: 23:27  up 35 days, 15:11, 1 user, load averages: 93.53 86.44 80.10
phát 200000 bản ghi (8 vòng trải từ now−34 s tới now−2 s): 0.67 s = 299984 bản ghi/s
(a) khởi động nóng từ Kafka (SinceMs now−34 s): kết nối + offsets_for_times 0.08 s; phát lại 200000 bản ghi / 3 partition trong 0.65 s = 307478 bản ghi/s; kho có 25000 tài xế (còn trong TTL ngay sau khi phát lại: 25000); hết hạn chờ: false
(b) khởi động nóng từ Redis (đo ở mục 6): 46.0 ms cho 25000 tài xế
(c) trực tiếp 10 s, 25000 tài xế gửi mỗi 4 s, kho đã khởi động nóng: nhận 62452 bản ghi (62452 áp dụng, 0 bỏ vì cũ), độ trễ gửi→kho p50 27 ms, p95 214 ms, p99 303 ms; lịch sử giữ 37575 (60.2%); kho 25000 → 25000 tài xế
    số tài xế nhìn thấy được (điểm trong TTL 30 s) theo thời gian: t=0s: 25000 (100%); t=1s: 25000 (100%); t=4s: 25000 (100%); t=7s: 25000 (100%); t=10s: 25000 (100%)
(d) trực tiếp 10 s, 25000 tài xế gửi mỗi 4 s, kho khởi động nguội (Latest): nhận 62415 bản ghi (62415 áp dụng, 0 bỏ vì cũ), độ trễ gửi→kho p50 9 ms, p95 56 ms, p99 83 ms; lịch sử giữ 37560 (60.2%); kho 0 → 25000 tài xế
    số tài xế nhìn thấy được (điểm trong TTL 30 s) theo thời gian: t=0s: 0 (0%); t=1s: 6207 (25%); t=2s: 12197 (49%); t=3s: 18720 (75%); t=4s: 24924 (100%); t=5s: 25000 (100%); t=6s: 25000 (100%); t=7s: 25000 (100%); t=8s: 25000 (100%); t=9s: 25000 (100%); t=10s: 25000 (100%)
đã xóa topic lab location.updates.v1.lab18-26623
uptime lúc kết thúc: 23:28  up 35 days, 15:11, 1 user, load averages: 94.36 87.27 80.59
```

Lần chạy trước, **ít tải hơn** (`uptime` 23:15, tải 28,56), không có file, chép từ màn hình (mục 3, mục 6 và mục 7 rút gọn):

```text
mảnh khóa luồng ghi luồng đọc ghi (cập nhật/s) đọc (truy vấn/s)   đọc p50 µs   đọc p99 µs
       1        1        0          1591019                0            -            -
      64        1        0          1732805                0            -            -
       1        4        0           870252                0            -            -
      64        4        0          3698259                0            -            -
       1        1        4            24845           364158          6.5         67.1
      64        1        4           684690           515988          6.4         20.3
       1        4        4            12440           391786          6.4         65.9
      64        4        4          1663963           304737          7.0         45.5
ghi 25000 tài xế qua MirrorWriter (gộp theo tài xế, lô 500): 0.19 s = 130653 tài xế/s
khởi động lại, nạp từ Redis: đọc 25000 bản ghi, nạp 25000, quá hạn TTL 0, hỏng 0: 81.0 ms
(a) khởi động nóng từ Kafka: kết nối + offsets_for_times 0.07 s; phát lại 200000 bản ghi / 3 partition trong 0.43 s = 468776 bản ghi/s
(c) ... độ trễ gửi→kho p50 5 ms, p95 11 ms, p99 26 ms
```

Một lần chạy khác lúc tải cao (`uptime` 23:26, tải 125,58), cũng chép từ màn hình: (a) phát lại 200.000 bản ghi mất **3,05 s** (65.540 bản ghi/s), nạp từ Redis **60,2 ms**, độ trễ gửi→kho khởi động nóng p50 5 ms, p95 26 ms, p99 42 ms.

(Lần chạy ít tải ở trên dùng lịch sử có giờ chạm tới `now + 2 s`, nên mục (c) có 239 bản ghi bị bỏ vì cũ; lỗi của lab, đã sửa trước lần chính: lịch sử giờ trải từ `now − 34 s` tới `now − 2 s`, và lần chính có 0 bản ghi bị bỏ.)

Điều nó cho thấy:

- **Đường ghi và đọc trong bộ nhớ rất rẻ:** một cập nhật `apply` 0,73 µs (đổi ô H3 ở 10,8% số cập nhật), điểm cũ đến sau bị bỏ trong 0,30 µs, truy vấn gần nhất k = 5 (bán kính ≤ 2 km) p50 5,25 µs trên 25.000 tài xế (lần chính, tải khoảng 90); kết quả khớp vét cạn với 0/200 truy vấn khác. Tải tham chiếu 6.250 cập nhật/giây chỉ dùng một phần nhỏ của một lõi.
- **Chia mảnh khóa quyết định khi vừa đọc vừa ghi.** Với một khóa toàn cục (1 mảnh) và 4 luồng đọc, tốc độ ghi rơi từ 0,72–1,6 triệu/s (không có đọc) xuống 12.440–174.080 cập nhật/s, còn với 64 mảnh vẫn 679.723 (lần chính) và 1.663.963 (lần ít tải) ở 4 luồng ghi + 4 luồng đọc. Chiều ngược lại có thật: ở 4 ghi + 4 đọc lần chính, một mảnh cho đọc nhiều hơn (188.216 so với 93.112 truy vấn/s), vì luồng ghi bị chặn. Cơ chế (luồng đọc giữ `RwLock` gần như liên tục nên luồng ghi chờ) là suy đoán hợp lý, **chưa kiểm riêng**; con số thì đo thật.
- TTL: tài xế im lặng quá 30 s bị truy vấn bỏ qua **ngay**, chưa cần dọn (truy vấn bán kính 30 km thấy 2.500 trong số 25.000); `sweep` xóa 22.500 tài xế trong 1,9 ms.
- Giảm mẫu sang lịch sử, chính sách như `history-writer` của Go (60 m, rẽ 30° sau 4 s, nhịp 60 s): trên 60 chuyến HCM sinh có seed, giữ **33,7%** (3.400/10.101): di chuyển 2.483, rẽ 848, điểm đầu 60, nhịp 9. Con số 60,2% ở mục 7 là của luồng đi ngẫu nhiên do lab sinh (đổi hướng nhiều), **không đại diện** cho xe chạy trên đường.
- Redis là bản phụ: ghi 25.000 tài xế qua `MirrorWriter` (gộp theo tài xế, lô 500) mất 1,25 s lần chính và 0,19 s lần ít tải, không bị bỏ cập nhật nào; 5.000 cập nhật cho 1.000 tài xế trong một lần xả chỉ ghi 1.000 lệnh; hàng đợi có giới hạn nên Redis chậm thì bỏ và đếm, không chặn luồng Kafka (kiểm bằng test: hàng đợi 4 chỗ, 50.000 `upsert` không chặn và có bản ghi bị bỏ). `GEOSEARCH` của Redis trả cùng tập tài xế với kho trên 100/100 truy vấn.
- **Khởi động lại thì nạp từ đâu** (cùng 25.000 tài xế): từ Redis (`HGETALL`) 46,0 ms lần chính, 81,0 ms lần ít tải; từ Kafka phát lại 34 giây cuối bằng `offsets_for_times`: 0,65 s lần chính cộng 0,08 s kết nối (200.000 bản ghi, 3 partition); nguội hoàn toàn (`Latest`, không nạp gì) thì kho đầy lại sau **4 giây**, đúng một chu kỳ báo cáo (25% sau 1 s, 49–50% sau 2 s, 75% sau 3 s, 99–100% sau 4 s). Tức Kafka không cần Redis để khởi động nóng, vì TTL chỉ 30 s nên phần phát lại ngắn; Redis có ích khi muốn khởi động dưới 0,1 s hoặc khi Kafka chậm.
- Độ trễ gửi→kho khi chạy trực tiếp qua Kafka: p50 5–27 ms, p95 11–214 ms, p99 26–303 ms tùy tải máy (lần chính p99 303 ms ở khởi động nóng, 83 ms ở nguội); SLO tham chiếu là độ tươi vị trí p95 dưới 5–10 giây, tức dư rất xa trong lab này (một broker, cùng máy, không có mạng thật).

### Test, và những gì test chứng minh

Lệnh chạy test mặc định (offline, không cần mạng; hai test tích hợp Kafka/Redis tự in «bỏ qua» rồi qua; output đã lọc còn các dòng `Running`, `test`, `test result`):

```bash
cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p map-match -p location-store
```

```text
     Running unittests src/lib.rs 
test mirror::tests::unpack_rejects_wrong_length_and_version ... ok
test history::tests::keeps_first_move_flag_turn_heartbeat_and_drops_the_rest ... ok
test model::tests::proto_round_trip_keeps_absent_distinct_from_zero ... ok
test store::tests::insert_update_stale_and_cell_move ... ok
test ingest::tests::sweep_removes_silent_driver_and_counts ... ok
test model::tests::rejects_missing_point_or_driver ... ok
test mirror::tests::pack_round_trip_all_presence_combinations ... ok
test store::tests::nearest_respects_radius_and_k ... ok
test store::tests::ttl_hides_silent_drivers_and_sweep_removes_them ... ok
test ingest::tests::stale_points_reach_neither_history_nor_store ... ok
test store::tests::concurrent_writers_and_readers_keep_invariants ... ok
test store::tests::nearest_matches_brute_force ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/kafka.rs 
test consume_apply_replay_ttl_against_real_kafka ... ok
test unreachable_kafka_fails_with_clear_message ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.02s
     Running tests/redis.rs 
test mirror_round_trip_expiry_geosearch_and_backpressure ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs 
test kalman::tests::missing_accuracy_differs_from_zero ... ok
test kalman::tests::ignores_out_of_order_and_resets_after_gap ... ok
test kalman::tests::follows_constant_velocity_without_bias ... ok
test kalman::tests::smoother_with_lag_is_no_worse_than_filter_on_noisy_line ... ok
test scenario::tests::split_is_disjoint ... ok
test route::tests::limit_and_budget_cut_the_search ... ok
test route::tests::matches_reference_dijkstra_and_path_is_contiguous ... ok
test trace::tests::same_seed_same_trace_and_truth_follows_route ... ok
test trace::tests::noise_has_roughly_the_requested_scale ... ok
test trace::tests::dropout_removes_points_and_keeps_truth ... ok
test scenario::tests::fixture_has_overpass_and_divided_avenue_pair ... ok
test scenario::tests::routes_are_contiguous_and_forced_through_via ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/hard_cases.rs 
test matching_is_deterministic_for_a_fixed_seed ... ok
test hmm_beats_nearest_snap_on_fixture_hard_cases ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.23s
     Running tests/properties.rs 
test kalman_follows_noise_free_straight_line ... ok
test offline_route_is_contiguous_when_chain_never_breaks ... ok
test online_decides_every_point_once_in_order ... ok
test clean_trace_is_matched_to_the_true_road ... ok
test online_with_unbounded_lag_equals_offline ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.13s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Test «HMM thắng snap» trên ca khó của fixture, in số đo (release):

```bash
cargo test --release --manifest-path examples/toi-noi-location/rust/Cargo.toml -p map-match --test hard_cases -- --nocapture
```

```text
test matching_is_deterministic_for_a_fixed_seed ... ok
snap 58.6%/65.4%  offline 85.5%/93.9%  online(5) 87.0%/94.6%  RMF 0.97 / 0.14 / 0.19  (654 điểm)
test hmm_beats_nearest_snap_on_fixture_hard_cases ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

(`snap`, `offline`, `online(5)` là «cạnh chặt/cạnh nới» trên 654 điểm của cầu vượt, chạy dưới cầu và hai chiều đại lộ có dải phân cách; ngưỡng của test là offline ≥ snap + 5 điểm, online ≥ snap + 3 điểm, RMF offline < RMF snap, đặt rộng hơn số đo để không dễ vỡ. Ngưỡng đặt sau khi xem một lần chạy.)

Test tích hợp **chạy thật với Kafka và Redis** (đặt biến môi trường, hạ tầng của `infra/` phải chạy):

```bash
TOI_NOI_KAFKA=1 TOI_NOI_REDIS=1 cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p location-store --test kafka --test redis
```

```text
     Running tests/kafka.rs 
test consume_apply_replay_ttl_against_real_kafka ... ok
test unreachable_kafka_fails_with_clear_message ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.04s
     Running tests/redis.rs 
test mirror_round_trip_expiry_geosearch_and_backpressure ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
```

Cùng test Kafka khi **Kafka không tới được** (cổng 1): test **thất bại với hướng dẫn bật hạ tầng**, không bỏ qua âm thầm (có thử lại 3 lần × 20 giây nên tốn 60 giây):

```bash
TOI_NOI_KAFKA=1 TOI_NOI_KAFKA_BROKERS=127.0.0.1:1 cargo test --manifest-path examples/toi-noi-location/rust/Cargo.toml -p location-store --test kafka consume_apply
```

```text
     Running tests/kafka.rs 
test consume_apply_replay_ttl_against_real_kafka has been running for over 60 seconds
test consume_apply_replay_ttl_against_real_kafka ... FAILED
thread 'consume_apply_replay_ttl_against_real_kafka' (61546755) panicked at crates/location-store/tests/kafka.rs:61:29:
TOI_NOI_KAFKA=1 nhưng không dùng được Kafka: không kết nối được Kafka ở 127.0.0.1:1: Admin operation error: OperationTimedOut (Local: Timed out). Bật hạ tầng lab: docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait kafka
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 60.32s
```

Test Kafka kiểm: tiêu thụ 300 bản ghi hợp lệ cộng 2 bản ghi thêm (một điểm cũ đến sau, một bản ghi hỏng) của 30 tài xế qua 3 partition, điểm cũ bị kho bỏ, bản ghi hỏng được đếm và bỏ qua, `accuracy` vắng vẫn vắng và `0` vẫn là `0` sau khi đi qua Kafka, khởi động nóng theo thời gian chỉ phát lại phần sau mốc, TTL dọn hết sau 60 s. Test Redis kiểm vòng ghi, nạp lại, lọc quá hạn, `GEOSEARCH`, gộp theo tài xế và hàng đợi đầy.

Ghi chú trung thực về độ ổn định: khi đang phát triển, `consume_apply_replay_ttl_against_real_kafka` thất bại 2 trong khoảng 7 lần chạy có đặt `TOI_NOI_KAFKA=1` (`CreateTopics` quá 15 giây). Lúc đó `docker ps` cho thấy container Kafka dùng chung (phiên Go) ở trạng thái **Paused** (thí nghiệm LOC-13); đó là nguyên nhân hợp lý nhưng không được chứng minh riêng. Sau khi thêm thử lại 3 lần cho `ensure_topic`, 6/6 lần chạy liên tiếp qua và các lần chạy ở trên đều qua. Nếu thấy lỗi này, kiểm `docker ps` trước.

### Kiểm tra toàn phần

```bash
bash examples/toi-noi-location/verify-p02-rust.sh
```

Output đã lọc (chỉ các dòng bước, `ok`, phần «bỏ qua» và `PASS`; mã thoát 0):

```text
==> cargo fmt --check (map-match, location-store)
==> rustfmt --check (các lab P02)
==> cargo clippy -D warnings (map-match, location-store, mọi target)
==> cargo clippy -D warnings (labs: lib và loc15…loc18)
==> cargo test (unit, proptest, test ca khó trên fixture, doctest; test Kafka/Redis tự bỏ qua nếu không đặt biến)
==> lab: loc15_kalman --fixture --trips 6
ok
==> lab: loc16_hmm --fixture --trips 6
ok
==> lab: loc17_online --fixture --trips 6 --per-kind 3 --drivers 50
ok
==> lab: loc18_kho_vi_tri --fixture --drivers 3000 --seconds 1
ok
Đã bỏ qua có chủ đích: lab trên đồ thị HCM thật (chỉ chạy --fixture, số chuyến nhỏ); test tích hợp Kafka (TOI_NOI_KAFKA=1) và Redis
(TOI_NOI_REDIS=1); loc18 với --redis --kafka; fmt/clippy/test toàn workspace và crate của việc khác (route-engine, simulator, geo-engine); cargo bench.
Số trong bài lấy từ results/p02-rust.md, không lấy từ lần chạy này.
PASS
```

## API các phần sau dựa vào

`map-match` (`examples/toi-noi-location/rust/crates/map-match`):

- `Obs { t_ms, pos, accuracy_m, speed_mps, heading_deg }` (`Option` cho ba trường cuối, vắng khác 0), `Obs::from_wire(...)` đúng đơn vị proto; `NoiseModel::sigma_m`.
- `kalman::{KalmanFilter::update, KalmanConfig, Estimate, Step, rts_fixed_lag}`.
- `hmm::{MatchCtx::new(&g, &index, MatchConfig), match_offline(&[Obs]) -> MatchResult, extend, path_between, reconstruct_route, interpolate_gap, MatchedPoint}`. `MatchedPoint.edge` là **cạnh có hướng** nên hướng xe chạy và phía đường (LOC-34) đọc thẳng từ đó; `offset_m` là mét từ nút đầu cạnh.
- `online::{OnlineMatcher::new(Lag::Points(n)), push(&mut ctx, Obs) -> Vec<Decision>, flush, provisional, mem_bytes}`: `Decision` có `point`, `first_guess`, `revisions`, `lag_ms`. Phát hiện lệch tuyến (LOC-48) so cạnh đã chốt với lộ trình dự kiến; ước lượng tạm (`provisional`) chỉ để hiển thị.
- `route::Router`, `trace::{drive, corrupt, DriveConfig, NoiseConfig, Canyon, zone_around}`, `scenario::{Catalog, InEdges, build, realize, extend_route, random_trip, Kind, Split}`, `eval::{score_trace, nearest_snap, Score, canon}`.

`location-store` (`examples/toi-noi-location/rust/crates/location-store`):

- `LatestStore::{new(StoreConfig), apply(&Update) -> Applied, get(id, now_ms), nearest(q, k, max_radius_m, now_ms), sweep(now_ms), snapshot, len, stats}`; `Update::from_proto` / `to_proto`; `Position` (accuracy, speed, heading là `Option`). Điểm điều phối (LOC-36) đọc vị trí tài xế qua `get` và `nearest`.
- `ingest::Ingest::{new, handle, sweep, run}`, `kafka::{KafkaConsumer::connect(brokers, topic, Start), poll, catch_up}`, `mirror::{RedisMirror, MirrorWriter}`, `history::{Sampler, Downsample, HistorySink}`.

## Chưa làm, hoặc làm khác dự kiến

- Chưa có GPS thật; mô hình nhiễu là của lab. Không có kết luận nào về độ chính xác ngoài đời.
- Chỉ profile ô tô; chưa bám đường cho xe máy (hẻm hẹp là chỗ xe máy khác ô tô). Chưa tính cấm rẽ.
- Chưa có «bỏ điểm nhảy rồi nối tiếp» cho điểm đa đường: mỗi điểm nhảy làm gãy chuỗi (LOC-16 mục 4b).
- Số hạng hướng đi (`heading_sigma_deg`) có sẵn nhưng tắt mặc định, vì chưa kiểm với heading thật.
- Ca khó lấy từ thẻ OSM và hình học của đồ thị HCM: danh mục «cầu vượt» có 729 giao cắt gồm cả cầu qua kênh và có thể có lỗi vẽ bản đồ; danh mục «song song» có 2.162 cặp, phần lớn là hẻm. Chưa duyệt từng ca bằng mắt.
- Redis chỉ thử với một nút. Chưa thử Redis chết giữa lúc chạy (chỉ có test hàng đợi đầy), chưa thử Kafka rebalance, chưa commit offset (cố ý: mỗi bản kho đọc cả topic bằng `assign`, xem `kafka.rs`).
- Ghi lịch sử ra Parquet là việc của Go (LOC-20); phần Rust chỉ quyết định điểm nào đi sang lịch sử (`history::Sampler`).
- Khác dự kiến ban đầu: thêm thước «cạnh nới nút» sau khi thấy thước chặt thấp ở nhiễu nhỏ; tách `location-store` thành crate riêng thay vì mở rộng `driver-store`; protobuf biên dịch bằng `protox` trong `build.rs`.
