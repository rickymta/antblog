# Kết quả P02, phần Go (LOC-11, 12, 13, 14, 19, 20)

Nguồn duy nhất cho số trong các bài Go của P02. Mọi khối `text` bên dưới là output thật, chép nguyên từ lần chạy; output đầy đủ (nhiều dòng hơn phần trích) nằm ở [`p02-go-raw/`](p02-go-raw/), tên file ghi ở mỗi mục. Chỗ nào phần trích đã lọc dòng, lệnh lọc được ghi ngay trên khối đó.

## Máy, cấu hình, seed

| Mục | Giá trị |
|---|---|
| Máy | Apple M1 Max, 10 lõi, 64 GB RAM, macOS (Darwin 25.6.0) |
| Go | 1.27.1 darwin/arm64, `GOMAXPROCS=10`, `ulimit -n` = 1.048.576 |
| Build đo | `go build` thường (không `-race`); `-race` chỉ dùng cho test và LOC-11 |
| Kafka | `apache/kafka:4.1.0` một broker trong Docker, topic `location.updates.v1` 6 partition, key `driver_id` |
| Cách chạy | `location-ingest`, `tracking-hub`, `history-writer`, `loadgen` và Kafka **cùng một máy**, nối qua loopback. Số đo là của cả cụm này, chia sẻ CPU với nhau, không phải của riêng ingest |
| Seed | đội xe `--seed 1` (mặc định) cho mọi lần chạy loadgen, LOC-13 và LOC-20; pipeline LOC-11 dùng seed 11. Quỹ đạo và thành phố của từng tài xế tái lập được. **Thời gian gửi thì không**: đo bằng đồng hồ thật nên con số dao động giữa các lần chạy |
| Mỗi số là một lần chạy | chưa lặp để ước lượng phương sai. Đừng đọc chênh lệch vài phần trăm là có thật |
| Dữ liệu | toàn bộ do simulator sinh (`internal/sim`): tâm 6 thành phố, tài xế đi ngẫu nhiên 0 đến 14 m/s, accuracy 3 đến 25 m. Không có GPS người thật |
| Phân vị | `internal/stat.Histogram` đếm theo bậc logarit rộng khoảng 2,5%, nên p50/p95/p99 là cận trên của bậc và có thể lớn hơn `max` đúng một bậc |

Chuẩn bị (đã chạy):

```bash
docker compose -f examples/toi-noi-location/infra/docker-compose.yml up -d --wait
go -C examples/toi-noi-location/go build -o bin/ ./cmd/...
```

Ghi chú hạ tầng: lần dựng đầu, lệnh `up -d --wait` chạy hơn 3 phút và container Kafka hiển thị `unhealthy`, vì healthcheck gọi `kafka-topics.sh` vào `127.0.0.1:9092` trong khi broker quảng bá `127.0.0.1:9096`, địa chỉ này không truy cập được từ bên trong container (log health ghi `Connection to node 1 (/127.0.0.1:9096) could not be established`). Đã sửa trong `infra/docker-compose.yml`: healthcheck dùng listener `INTERNAL` ở `127.0.0.1:29092`. Kiểm lại lúc điều tra ở dưới: `up -d --wait kafka` trả về với container `Up 7 seconds (healthy)`. Các phép đo ở phần này chạy trước khi sửa, nhưng Kafka vẫn dùng được từ máy chủ nên số đo không bị ảnh hưởng.

Cách đọc cột của loadgen (dòng `STEP`):

```text
target_pts_s achieved_pts_s msgs_s p50_ms p95_ms p99_ms max_ms errs ingest_cpu_pct ingest_heap_mb ingest_rss_mb
ingest_goroutines q_max_depth q_dropped q_coalesced loadgen_cpu_pct loadgen_rss_mb
```

- `achieved_pts_s` là số điểm được ack trung bình cả bước, kể cả vài giây đầu khi các tài xế đổi nhịp. Số ở dòng giữa chừng `t=20s` là tốc độ ổn định.
- `p50..max_ms` là độ trễ từ lúc app gửi lô tới lúc nhận `BatchAck`, đo ở phía client.
- `ingest_cpu_pct` là (CPU user + sys của tiến trình ingest) chia thời gian thực, tính trên **một lõi** (100 = một lõi đầy; sys gồm cả phần kernel xử lý socket). `ingest_rss_mb` là đỉnh RSS từ lúc tiến trình khởi động.
- `ingest_heap_mb` là `HeapAlloc` tại thời điểm chụp, chưa trừ rác chưa thu gom, nên dao động.
- `q_dropped` (gồm điểm bị đẩy ra) và `q_coalesced` là số tăng thêm trong bước đó; `q_max_depth` là đỉnh độ sâu hàng đợi của ingest từ lúc khởi động.

Chạy lại các thí nghiệm có 25.000 kết nối: dải cổng tạm thời của macOS (`net.inet.ip.portrange` 49152 đến 65535) chỉ có 16.384 cổng cho mỗi địa chỉ nguồn, nên ingest lắng nghe cả `127.0.0.1` lẫn `[::1]` và loadgen chia kết nối đều cho hai địa chỉ; mỗi lần đóng 25.000 kết nối để lại ~25.000 socket `TIME_WAIT` trong ~30 giây, chờ hết (`netstat -an | grep -c TIME_WAIT` dưới vài trăm) rồi mới chạy lần sau, nếu không sẽ gặp `can't assign requested address`. Nguồn của ghi chú này là các lần chạy thử đầu tiên (output không giữ lại): hai cổng trên cùng `127.0.0.1` chỉ dựng được 16.270 trên 25.000 kết nối; thêm `[::1]` nhưng chạy ngay sau lần trước còn 20.077; sau khi chờ hết `TIME_WAIT` đủ 25.000.


### LOC-11

Lệnh (từ gốc repo). `race-bad` chạy không `-race` để thấy tổng bị mất; có `-race` thì detector báo lỗi:

```bash
go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon pipeline
go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon timeout
go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon fail
go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon race-bad
go -C examples/toi-noi-location/go run -race ./cmd/loc11_nhap_mon race-fixed
go -C examples/toi-noi-location/go run -race ./cmd/loc11_nhap_mon race-bad 2>&1 | head -32
```

Output (`p02-go-raw/loc11.txt`):

```text
$ go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon pipeline
sinh=10000 kiểm=10000 gom=10000 thành phố=map[CT:400 DN:1150 HCM:4750 HN:2750 HP:500 NT:450]
hoàn tất, không lỗi
(exit=0)

$ go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon timeout
sinh=1242 kiểm=1242 gom=1242 thành phố=map[CT:48 DN:146 HCM:590 HN:339 HP:63 NT:56]
kết thúc sớm vì context hết hạn: context deadline exceeded
(exit=0)

$ go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon fail
sinh=729 kiểm=704 gom=701 thành phố=map[CT:27 DN:83 HCM:333 HN:188 HP:38 NT:32]
kết thúc vì lỗi, errgroup huỷ các stage còn lại: điểm hỏng ở stage kiểm tra
(exit=0)

$ go -C examples/toi-noi-location/go run ./cmd/loc11_nhap_mon race-bad
race-bad: đếm được 270093, đúng phải là 800000 (mất 529907)
(exit=0)

$ go -C examples/toi-noi-location/go run -race ./cmd/loc11_nhap_mon race-fixed
mutex:   đếm được 800000, muốn 800000
atomic:  đếm được 800000, muốn 800000
channel: đếm được 800000, muốn 800000
(exit=0)

$ go -C examples/toi-noi-location/go run -race ./cmd/loc11_nhap_mon pipeline
sinh=10000 kiểm=10000 gom=10000 thành phố=map[CT:400 DN:1150 HCM:4750 HN:2750 HP:500 NT:450]
hoàn tất, không lỗi
(exit=0)
```

Race detector (`p02-go-raw/loc11_race.txt`, 32 dòng đầu; bản đầy đủ ở `loc11_race_full.txt`):

```text
$ go -C examples/toi-noi-location/go run -race ./cmd/loc11_nhap_mon race-bad 2>&1 | head -32
==================
WARNING: DATA RACE
Read at 0x00c0001140b8 by goroutine 10:
  main.raceBad.func1()
      /Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi-noi-location/go/cmd/loc11_nhap_mon/race.go:25 +0x88

Previous write at 0x00c0001140b8 by goroutine 14:
  main.raceBad.func1()
      /Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi-noi-location/go/cmd/loc11_nhap_mon/race.go:25 +0x98

Goroutine 10 (running) created at:
  main.raceBad()
      /Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi-noi-location/go/cmd/loc11_nhap_mon/race.go:22 +0x70
  main.main()
      /Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi-noi-location/go/cmd/loc11_nhap_mon/main.go:28 +0x1d0
  runtime.main()
      /usr/local/go/src/runtime/proc.go:302 +0x37c

Goroutine 14 (running) created at:
  main.raceBad()
      /Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi-noi-location/go/cmd/loc11_nhap_mon/race.go:22 +0x70
  main.main()
      /Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi-noi-location/go/cmd/loc11_nhap_mon/main.go:28 +0x1d0
  runtime.main()
      /usr/local/go/src/runtime/proc.go:302 +0x37c
==================
==================
WARNING: DATA RACE
Write at 0x00c0001140b8 by goroutine 10:
  main.raceBad.func1()
      /Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi-noi-location/go/cmd/loc11_nhap_mon/race.go:25 +0x98
```

Một lần chạy `-race` thứ hai với cùng lệnh, bỏ stdout và giữ toàn bộ stderr (`loc11_race_meta.txt`: mã thoát của `go run`, số khối báo cáo, hai dòng cuối; goroutine id khác lần trên vì mỗi lần chạy khác nhau):

```text
exit=1
so khối WARNING: DATA RACE: 2
Found 2 data race(s)
exit status 66
```

Cấu hình: 8 goroutine, mỗi goroutine cộng 100.000 lần vào cùng một `int`. Pipeline: 200 tài xế × 50 điểm, seed 11, 4 worker kiểm tra, một goroutine gom theo thành phố.

Số cho thấy: bản sai mất hơn một nửa số đếm (270.093 trên 800.000 trong lần chạy này, con số thay đổi mỗi lần) mà không báo lỗi nào; `-race` chỉ ra hai khối tranh chấp (đọc/ghi và ghi/ghi) cùng dòng `race.go:25` và kết thúc với `exit status 66`. Cả ba cách sửa (mutex, atomic, channel với một goroutine sở hữu) ra đúng 800.000. Bất ngờ nhỏ: lần `fail` ở trên có `sinh=729 kiểm=704 gom=701`, tức các stage không dừng cùng lúc; số điểm đã sinh ra nhưng chưa tới stage cuối khi `errgroup` huỷ context là số không xác định, nên bài không được viết số này như một hằng số.


### LOC-12

Kết nối: mỗi tài xế một WebSocket binary frame, mỗi frame một `GpsBatch` (3 điểm), trả `BatchAck`. 25.000 tài xế × một điểm mỗi 4 giây = 6.250 điểm/giây (số trong `du-an-mau-toi-noi`). Ingest chạy `--sink kafka` (Kafka thật) trừ khi ghi khác. Mỗi lần chạy khởi động ingest mới (đã tắt tiến trình cũ), chờ hết `TIME_WAIT`.

#### A. Chịu 6.250 điểm/giây, lô 3 điểm, Kafka thật

```bash
examples/toi-noi-location/go/bin/location-ingest --sink kafka --listen '127.0.0.1:8081,[::1]:8083'
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --rate 6250 --batch 3 --duration 60s --report-every 10s
```

Output (`p02-go-raw/loc12_a.txt`; dòng `STEP` là tổng kết, các dòng `t=` là cửa sổ 10 giây):

```text
# loadgen drivers=25000 seed=1 batch=3 urls=2 viewers=0 slow_viewers=0
# kết nối: 25000/25000 thành công, 0 lỗi, mất 2.6s
# khởi động 14s ở 6250 điểm/s (không tính)
STEP	target_pts_s	achieved_pts_s	msgs_s	p50_ms	p95_ms	p99_ms	max_ms	errs	ingest_cpu_pct	ingest_heap_mb	ingest_rss_mb	ingest_goroutines	q_max_depth	q_dropped	q_coalesced	loadgen_cpu_pct	loadgen_rss_mb
  t= 10s target=6250 acked_pts_s=6261 win_p50=0.2ms win_p99=8.8ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=74958 heap_mb=496
  t= 20s target=6250 acked_pts_s=6203 win_p50=0.2ms win_p99=7.3ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=136980 heap_mb=597
  t= 30s target=6250 acked_pts_s=6281 win_p50=0.4ms win_p99=13.1ms errs=0 q_depth=9 q_coalesced=0 q_dropped=0 kafka_delivered=199806 heap_mb=431
  t= 40s target=6250 acked_pts_s=6262 win_p50=0.3ms win_p99=8.6ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=262437 heap_mb=532
  t= 50s target=6250 acked_pts_s=6243 win_p50=0.4ms win_p99=11.6ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=324837 heap_mb=632
  t= 60s target=6250 acked_pts_s=6251 win_p50=0.4ms win_p99=9.1ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=387357 heap_mb=733
STEP	6250	6250	2083	0.28	6.25	10.25	51.38	0	19	733.0	1020.1	25012	108	0	0	24	684.4
```

Số đo chính của lần chạy này:

| Chỉ số | Giá trị |
|---|---|
| Tốc độ đạt được | 6.250 điểm/s (2.083 lô/s), lỗi 0, hàng đợi bỏ 0, gộp 0 |
| Độ trễ ack (client) | p50 0,28 ms, p95 6,25 ms, p99 10,25 ms, max 51,38 ms |
| CPU ingest | 19% của một lõi (user + sys) |
| CPU loadgen | 24% của một lõi |
| Bộ nhớ ingest | RSS đỉnh 1.020 MB cho 25.012 goroutine, khoảng 41 KB mỗi kết nối; `HeapAlloc` dao động 430 đến 733 MB giữa các lần GC |
| Hàng đợi | độ sâu đỉnh 108 điểm |

#### B. Cùng tải, nhưng lô 1 điểm (không batching)

```bash
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --rate 6250 --batch 1 --duration 60s --report-every 10s
```

```text
STEP	target_pts_s	achieved_pts_s	msgs_s	p50_ms	p95_ms	p99_ms	max_ms	errs	ingest_cpu_pct	ingest_heap_mb	ingest_rss_mb	ingest_goroutines	q_max_depth	q_dropped	q_coalesced	loadgen_cpu_pct	loadgen_rss_mb
STEP	6250	6250	6250	0.75	11.04	30.38	109.22	0	29	587.2	1085.4	25012	304	0	0	33	752.0
```

Cùng 6.250 điểm/s nhưng 6.250 lô/s thay vì 2.083: CPU ingest 29% so với 19%, p99 ack 30,38 ms so với 10,25 ms. Batching phía app giảm khoảng một phần ba CPU của ingest và độ trễ đuôi; cái giá là điểm cũ nhất trong lô đã chờ ở app tối đa 8 giây (xem LOC-19).

#### C. Tìm điểm không theo kịp: tăng dần tốc độ, sink `discard` (bỏ Kafka ra)

```bash
examples/toi-noi-location/go/bin/location-ingest --sink discard --listen '127.0.0.1:8081,[::1]:8083'
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --batch 3 --sweep 6250,12500,25000,50000,100000 --step 20s --report-every 10s
```

Các dòng `STEP` (`p02-go-raw/loc12_c.txt`):

```text
STEP	target_pts_s	achieved_pts_s	msgs_s	p50_ms	p95_ms	p99_ms	max_ms	errs	ingest_cpu_pct	ingest_heap_mb	ingest_rss_mb	ingest_goroutines	q_max_depth	q_dropped	q_coalesced	loadgen_cpu_pct	loadgen_rss_mb
STEP	6250	6248	2083	1.01	10.51	23.15	123.61	0	12	538.2	803.7	25008	183	0	0	19	634.0
STEP	12500	11883	3961	0.37	7.43	16.39	113.26	0	23	452.9	1040.3	25008	384	0	0	32	699.3
STEP	25000	24371	8124	0.85	12.80	31.92	175.90	0	34	656.8	1071.1	25008	1113	0	0	47	727.2
STEP	50000	49368	16456	1.86	79.58	219.02	433.33	0	60	603.1	1147.8	25008	5989	0	2247	70	788.1
STEP	100000	59606	20954	519.78	3394.95	6140.54	14454.70	11	80	497.1	1292.1	25008	11966	0	20075	91	985.3
```

#### D. Cùng sweep với Kafka thật

Cùng lệnh loadgen, ingest `--sink kafka` (`p02-go-raw/loc12_d.txt`). Ở bước 100.000, `ingest_*` là `-1` vì loadgen không lấy được `/debug/stats` trong 5 giây: ingest không còn đáp ứng kịp.

```text
STEP	target_pts_s	achieved_pts_s	msgs_s	p50_ms	p95_ms	p99_ms	max_ms	errs	ingest_cpu_pct	ingest_heap_mb	ingest_rss_mb	ingest_goroutines	q_max_depth	q_dropped	q_coalesced	loadgen_cpu_pct	loadgen_rss_mb
STEP	6250	6247	2082	1.03	11.89	24.32	74.59	0	15	589.6	864.9	25012	222	0	0	18	642.1
STEP	12500	11881	3962	0.75	9.29	18.54	134.66	0	25	571.4	1062.5	25010	443	0	0	30	691.2
STEP	25000	24367	8120	1.77	22.04	49.78	288.53	0	36	483.1	1127.2	25010	1517	0	0	41	695.5
STEP	50000	49338	16794	7.62	280.36	396.15	1130.07	0	62	688.1	1177.8	25015	6038	0	6508	68	807.8
STEP	100000	69594	25930	752.80	4679.98	9115.67	15701.66	50766	-1	-1.0	-1.0	-1	-1	-1	-1	142	1360.4
```

Các cửa sổ ổn định (dòng `t=20s`) của bước 50.000 và 100.000, vì dòng `STEP` còn tính cả vài giây đầu đổi nhịp:

```text
  t= 10s target=50000 acked_pts_s=47938 win_p50=4.2ms win_p99=367.9ms errs=0 q_depth=9 q_coalesced=0 q_dropped=0 kafka_delivered=1345588 heap_mb=787
  t= 20s target=50000 acked_pts_s=49840 win_p50=13.1ms win_p99=396.1ms errs=0 q_depth=965 q_coalesced=6508 q_dropped=0 kafka_delivered=1833614 heap_mb=657
  t= 10s target=100000 acked_pts_s=36622 win_p50=682.0ms win_p99=10313.5ms errs=0 q_depth=13349 q_coalesced=274279 q_dropped=0 kafka_delivered=1853491 heap_mb=784
  t= 20s target=100000 acked_pts_s=52913 win_p50=873.0ms win_p99=7121.1ms errs=202 q_depth=21900 q_coalesced=372182 q_dropped=0 kafka_delivered=2045824 heap_mb=572
```

Tiêu chí «theo kịp» dùng ở đây: tốc độ đạt được ≥ 98% mục tiêu, không lỗi ack, p99 dưới 1 giây. Kết quả trên máy này:

| Mục tiêu (điểm/s) | Sink discard: đạt / p99 | Kafka: đạt / p99 | Theo kịp? |
|---|---|---|---|
| 6.250 (đỉnh Tới Nơi) | 6.248 / 23 ms | 6.247 / 24 ms | có |
| 12.500 | 11.883 / 16 ms (cửa sổ ổn định 12.512) | 11.881 / 19 ms (12.493) | có |
| 25.000 | 24.371 / 32 ms (24.987) | 24.367 / 50 ms (24.987) | có |
| 50.000 | 49.368 / 219 ms | 49.338 / 396 ms | có, p99 đã gấp 9 đến 16 lần so với 6.250 |
| 100.000 | 59.606 / 6.141 ms, 11 lỗi | 69.594 / 9.116 ms, 50.766 lỗi ack | **không** |

Điểm gãy nằm giữa 50.000 và 100.000 điểm/giây (8 đến 16 lần đỉnh của Tới Nơi) trên máy này, khi loadgen cũng chạy ở máy này và đã dùng 70 đến 90% một lõi.

#### E. Vì sao gãy: profile ở 100.000 điểm/s

```bash
examples/toi-noi-location/go/bin/location-ingest --sink discard --listen '127.0.0.1:8081,[::1]:8083' --profile-contention
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --batch 3 --rate 100000 --duration 30s --report-every 10s
curl -o ingest_cpu.prof 'http://127.0.0.1:8081/debug/pprof/profile?seconds=15'     # bắt đầu 12 giây sau loadgen
curl 'http://127.0.0.1:8081/debug/pprof/mutex?debug=1'
go tool pprof -top -nodecount=25 examples/toi-noi-location/go/bin/location-ingest ingest_cpu.prof
```

Dòng `STEP` của lần chạy này (`p02-go-raw/loc12_p.txt`), và 25 dòng đầu của `pprof -top` (`loc12_pprof.txt`):

```text
STEP	target_pts_s	achieved_pts_s	msgs_s	p50_ms	p95_ms	p99_ms	max_ms	errs	ingest_cpu_pct	ingest_heap_mb	ingest_rss_mb	ingest_goroutines	q_max_depth	q_dropped	q_coalesced	loadgen_cpu_pct	loadgen_rss_mb
STEP	100000	57343	19522	894.84	3566.82	6140.54	14986.64	49	86	806.2	1366.0	25007	14142	0	19906	89	1117.8
```

```text
$ go tool pprof -top -nodecount=25 ingest_cpu.prof
File: location-ingest
Type: cpu
Time: 2026-11-06 20:43:45 +07
Duration: 15.15s, Total samples = 7.91s (52.21%)
Showing nodes accounting for 7.39s, 93.43% of 7.91s total
Dropped 178 nodes (cum <= 0.04s)
Showing top 25 nodes out of 129
      flat  flat%   sum%        cum   cum%
     4.89s 61.82% 61.82%      4.89s 61.82%  syscall.rawsyscalln
     1.21s 15.30% 77.12%      1.21s 15.30%  runtime.pthread_cond_wait
     0.43s  5.44% 82.55%      0.43s  5.44%  runtime.pthread_cond_signal
     0.37s  4.68% 87.23%      0.37s  4.68%  runtime.kevent
     0.12s  1.52% 88.75%      0.12s  1.52%  runtime.usleep
     0.05s  0.63% 89.38%      0.05s  0.63%  runtime.madvise
     0.04s  0.51% 89.89%      0.04s  0.51%  runtime.(*mspan).moveInlineMarks
     0.04s  0.51% 90.39%      0.04s  0.51%  runtime.nanotime1
     0.04s  0.51% 90.90%      0.07s  0.88%  runtime.tryDeferToSpanScan
     0.03s  0.38% 91.28%      0.07s  0.88%  runtime.scanblock
     0.02s  0.25% 91.53%      0.04s  0.51%  context.(*cancelCtx).Done
```

Mutex profile, mục đầu (`p02-go-raw/ingest_mutex.txt`):

```text
--- mutex:
cycles/second=999981391
sampling period=5
16838362484250 276925 @ 0x104f1e9ad 0x104f1f7dc 0x104f1df00 0x104f1d8e0 0x104f1d6f8 0x1050b2088 0x104e322c8 0x104e339d4 0x104e3a0f0 0x104e30b20 0x104e
#	0x104f1e9ac	sync.(*Mutex).Unlock+0x1c					/usr/local/go/src/sync/mutex.go:65
#	0x104f1f7db	toinoi/location/internal/backpressure.(*Queue[...]).Offer+0x4cb	/Users/quandh/Other/github/antblog/.claude/worktrees/location-series/exa
#	0x104f1deff	toinoi/location/internal/ingest.(*Server).process+0x46f		/Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/t
#	0x104f1d8df	toinoi/location/internal/ingest.(*Server).serve+0x17f		/Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi
#	0x104f1d6f7	toinoi/location/internal/ingest.(*Server).HandleWS+0x2f7	/Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/t
```

Đọc kết quả: trong cửa sổ 15 giây, ingest chỉ dùng 7,91 giây CPU (52% của một lõi, dù máy có 10 lõi), trong đó 62% trong `syscall.rawsyscalln` (đọc/ghi 25.000 socket) và 15% trong `pthread_cond_wait`. Mutex profile cho thấy khoá của hàng đợi (`Queue.Offer`) là điểm có nhiều goroutine phải chờ nhất (hơn 276 nghìn sự kiện tranh chấp ở mục đầu; mutex profile chỉ lấy mẫu một phần). Hai điều này khớp với việc ingest dừng ở dưới một lõi: goroutine ngủ chờ khoá hoặc chờ socket chứ không bận tính. Đây là cách **đọc** của người viết, chưa phải bằng chứng nhân quả; thí nghiệm sau thử bỏ khoá chung.

#### F. Thử chia hàng đợi làm 8 phần (`--shards 8`), sink `discard`

```bash
examples/toi-noi-location/go/bin/location-ingest --sink discard --listen '127.0.0.1:8081,[::1]:8083' --shards 1     # rồi --shards 8
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --batch 3 --sweep 50000,100000,150000 --step 20s --report-every 10s
```

Cột: `target achieved msgs_s p50 p95 p99 max errs ingest_cpu% heap rss goroutines q_max q_dropped q_coalesced loadgen_cpu% loadgen_rss`. (`achieved` có thể vượt `target` khi tài xế bù các nhịp bị trễ.)

`--shards 1`:

```text
STEP	50000	53565	17178	4.32	235.86	649.13	1233.53	0	60	666.1	1098.0	25008	5023	0	0	71	738.6
STEP	100000	41012	14234	716.52	4796.98	6947.46	13960.80	142	77	550.9	1309.4	24990	5134	0	250	74	1193.7
STEP	150000	41726	17036	894.84	6612.69	10313.54	15304.20	68479	68	638.2	1468.0	21135	20977	0	30198	85	1288.5
```

`--shards 8`:

```text
STEP	50000	50147	16710	4.01	143.94	406.05	658.39	0	55	538.9	1117.9	25022	5048	0	326	66	740.0
STEP	100000	49967	17428	1063.68	3841.08	5844.66	14289.28	4	74	571.0	1371.6	25021	6362	0	501	74	1009.5
STEP	150000	77380	25201	830.95	2786.40	4239.83	12465.44	49	95	548.6	1469.8	25021	10357	0	1243	87	1040.2
```

`--shards 8` cộng Kafka thật, sweep 50.000 và 100.000 (`loc12_s8k.txt`):

```text
STEP	50000	50301	16769	7.62	301.92	617.86	1726.02	0	63	549.6	1144.7	25027	9564	0	23877	70	738.3
STEP	100000	53590	18889	649.13	3747.39	5990.77	15050.44	2499	77	939.8	1495.4	24362	28304	0	419977	82	1129.8
```

Chia khoá **không** dời điểm gãy một cách rõ ràng: bước 100.000 đạt 41.012 (1 phần) và 49.967 (8 phần) điểm/s, p99 6,9 và 5,8 giây, đều là «không theo kịp»; bước 150.000 đạt 41.726 so với 77.380 nhưng chỉ một lần chạy mỗi bên nên chưa kết luận được. Khoá chung là có thật nhưng không phải nút cổ chai duy nhất; phần còn lại (syscall socket, loadgen chung máy, GC với 25.000 kết nối) chưa tách ra được. Các lần chạy C và D dùng bản dựng trước khi thêm `--shards` và `/debug/pprof`; đường `--shards 1` là cùng mã, chỉ thêm một lần kiểm `len`.

#### So sánh WebSocket và gRPC stream (lập luận, **không có số đo gRPC**)

Lab này chỉ đo WebSocket; chưa dựng gRPC stream để đối chứng.

- **Qua proxy và mạng di động.** WebSocket bắt đầu bằng một HTTP/1.1 Upgrade nên đi qua hầu hết CDN, L7 load balancer và proxy doanh nghiệp. gRPC cần HTTP/2 xuyên suốt (kể cả trailers) và thường bị proxy/LB cấu hình sai hoặc hạ cấp. Kiến trúc §7.3 cũng nói không mặc định dùng gRPC trực tiếp từ mobile, gRPC streaming hợp hơn cho service-với-service.
- **Hợp đồng.** gRPC cho schema, codegen client/server, deadline, keepalive có sẵn. Với WebSocket phải tự đặt: ở đây dùng protobuf trong binary frame, `batch_seq` và `BatchAck` do ta định nghĩa; nghĩa là phải tự viết và tự kiểm các chuyện gRPC làm hộ.
- **Flow control.** HTTP/2 có cửa sổ theo stream; WebSocket chỉ có TCP, nên backpressure tầng ứng dụng (ack, độ sâu hàng đợi) phải tự làm. Ở LOC-19 chính vì vậy mà hub cần `TrackingAck` (xem mục đó).
- **Mạng chập chờn.** Điều quyết định ở đây không phải giao thức mà là hợp đồng: app giữ lô cho tới khi nhận `BatchAck` rồi gửi lại sau khi nối lại, server dùng `seq` và giờ để loại điểm trùng/cũ (`validate`). Cả hai giao thức cần phần này. Lab **chưa** chạy kịch bản mất mạng, nối lại và gửi lại; mới có test đơn vị cho điểm đến sau (LOC-14).
- **Chi phí đo được của WebSocket ở đây:** khoảng 41 KB RSS mỗi kết nối (1.020 MB / 25.012), 19% một lõi cho 6.250 điểm/s, và ack p99 10 ms trên loopback. Số này không gồm độ trễ mạng thật.

Hợp đồng protobuf và quy ước số: `examples/toi-noi-location/proto/README.md`.


### LOC-13

#### Thí nghiệm offline: sink chậm, ba chính sách

Tải: 25.000 tài xế, 6.250 điểm/s (một điểm mỗi tài xế mỗi 4 giây), hàng đợi tối đa 30.000 điểm (`unbounded` không giới hạn), seed 1. Sink giả (`SlowSink`) đổi tốc độ theo pha: `normal` 5 giây ở 10.000 điểm/s, `slow` 40 giây ở 1.000 điểm/s, `recover` 25 giây ở 12.000 điểm/s. `Coalesce` giữ FIFO bình thường và chỉ gộp theo tài xế khi độ sâu vượt `capacity/10` (3.000). Mỗi chính sách chạy một tiến trình riêng:

```bash
for v in unbounded drop-newest coalesce; do
  examples/toi-noi-location/go/bin/loc13_backpressure --variant $v
done
```

Tổng kết theo pha. `age` là tuổi của điểm lúc giao cho sink (giờ giao trừ giờ lấy mẫu); `stale` là số điểm đã giao mà khi đó tài xế đó đã có điểm mới hơn (đã tới hàng đợi hoặc đã bị bỏ).

`unbounded` (`p02-go-raw/loc13_unbounded.txt`):

```text
# variant=unbounded drivers=25000 rate=6250 điểm/s cap=30000 seed=1 GOMAXPROCS=10
# tổng kết theo pha (tuổi = lúc giao cho sink trừ lúc lấy mẫu; stale = đã có điểm mới hơn của cùng tài xế khi giao)
S	phase	offered	delivered	lost	stale_delivered	stale_pct	age_p50_ms	age_p99_ms	age_max_ms	max_depth
S	normal	31312	31250	0	0	0.0	8	14	21	0
S	slow	262875	41725	0	37000	88.7	17756	35449	35329	221012
S	recover	166000	294400	0	294400	100.0	25715	35449	35501	216250
# cuối: depth=92612 heap_live_max=97.5 MB; còn trong hàng đợi chưa giao: 92612
```

`drop-newest` (`loc13_drop-newest.txt`):

```text
# variant=drop-newest drivers=25000 rate=6250 điểm/s cap=30000 seed=1 GOMAXPROCS=10
# tổng kết theo pha (tuổi = lúc giao cho sink trừ lúc lấy mẫu; stale = đã có điểm mới hơn của cùng tài xế khi giao)
S	phase	offered	delivered	lost	stale_delivered	stale_pct	age_p50_ms	age_p99_ms	age_max_ms	max_depth
S	normal	31375	31312	0	0	0.0	8	16	18	0
S	slow	253750	40308	183305	35600	88.3	17322	30567	30366	30000
S	recover	157187	186825	437	29000	15.5	8	29094	30352	25400
# cuối: depth=0 heap_live_max=17.3 MB; còn trong hàng đợi chưa giao: 0
```

`coalesce` (`loc13_coalesce.txt`):

```text
# variant=coalesce drivers=25000 rate=6250 điểm/s cap=30000 seed=1 GOMAXPROCS=10
# tổng kết theo pha (tuổi = lúc giao cho sink trừ lúc lấy mẫu; stale = đã có điểm mới hơn của cùng tài xế khi giao)
S	phase	offered	delivered	lost	stale_delivered	stale_pct	age_p50_ms	age_p99_ms	age_max_ms	max_depth
S	normal	31562	31562	0	0	0.0	9	22	32	22
S	slow	253250	40063	190446	1991	5.0	2124	4240	4200	24925
S	recover	157938	171062	9554	0	0.0	8	2124	2244	12941
# cuối: depth=0 heap_live_max=16.0 MB; còn trong hàng đợi chưa giao: 0
```

Chuỗi thời gian từng giây dùng vẽ hình (cột `depth`, `heap_live_mb`, `age_p99_ms`; `heap_live_mb` là bộ nhớ **sống** sau `runtime.GC()`). Mỗi bản đầy đủ 70 dòng nằm ở file raw; bên dưới lọc mỗi 5 giây bằng `awk -F'\t' '$1=="T" && $2%5==0'`, cộng các giây 6 đến 16 của `coalesce` để thấy lúc coalesce bắt đầu:

`unbounded`:

```text
t_s	phase	sink_pps	depth	offered	delivered	coalesced	dropped	evicted	heap_live_mb	age_p50_ms	age_p99_ms
5	normal	10000	0	31312	31250	0	0	0	4.6	8	14
10	slow	1000	26562	62937	36175	0	0	0	15.7	3841	4240
15	slow	1000	53237	94812	41375	0	0	0	27.0	8258	8676
20	slow	1000	80487	127062	46375	0	0	0	38.4	12566	12880
25	slow	1000	107537	159312	51575	0	0	0	49.7	16900	17322
30	slow	1000	134712	191687	56775	0	0	0	61.2	21106	21633
35	slow	1000	162387	224562	61975	0	0	0	72.8	25715	26358
40	slow	1000	190125	257500	67175	0	0	0	84.4	30567	30567
45	slow	1000	221012	294187	72975	0	0	0	97.5	35449	35449
50	recover	12000	195675	329250	133375	0	0	0	86.9	32115	32918
55	recover	12000	169562	361937	192175	0	0	0	75.9	27693	28385
60	recover	12000	143487	395062	251375	0	0	0	65.1	23879	23879
65	recover	12000	118114	427689	309375	0	0	0	54.3	19599	20089
70	recover	12000	92612	460187	367375	0	0	0	43.6	15311	15693
```

`drop-newest`:

```text
t_s	phase	sink_pps	depth	offered	delivered	coalesced	dropped	evicted	heap_live_mb	age_p50_ms	age_p99_ms
5	normal	10000	0	31375	31312	0	0	0	4.6	8	18
10	slow	1000	26455	62875	36220	0	0	0	15.7	3841	4136
15	slow	1000	29800	94562	41420	0	23142	0	17.2	8057	8676
20	slow	1000	30000	126437	46420	0	49817	0	17.3	12566	12880
25	slow	1000	30000	158437	51420	0	76817	0	17.2	16900	17322
30	slow	1000	29862	190187	56620	0	103505	0	17.2	21106	21633
35	slow	1000	29988	221750	61620	0	129942	0	17.2	25715	25715
40	slow	1000	30000	253437	66620	0	156617	0	17.2	29822	30567
45	slow	1000	30000	285125	71620	0	183305	0	17.2	30567	30567
50	recover	12000	5700	317062	127420	0	183742	0	7.0	1362	1787
55	recover	12000	0	348500	164695	0	183742	0	4.6	6	15
60	recover	12000	0	379875	196070	0	183742	0	4.6	7	12
65	recover	12000	18	411250	227445	0	183742	0	4.6	7	28
70	recover	12000	0	442312	258445	0	183742	0	4.7	7	39
```

`coalesce`:

```text
t_s	phase	sink_pps	depth	offered	delivered	coalesced	dropped	evicted	heap_live_mb	age_p50_ms	age_p99_ms
5	normal	10000	0	31562	31562	0	0	0	4.6	9	21
6	slow	1000	5250	37875	32425	0	0	0	7.0	427	772
7	slow	1000	10500	44125	33425	0	0	0	9.4	1264	1619
8	slow	1000	15812	50437	34425	0	0	0	12.1	2124	2524
9	slow	1000	21187	56812	35425	0	0	0	14.3	3001	3395
10	slow	1000	24800	63187	36425	1762	0	0	15.9	3841	4240
11	slow	1000	23800	69562	37425	8137	0	0	15.5	682	1038
12	slow	1000	22800	76000	38425	14575	0	0	15.0	1541	1924
13	slow	1000	21800	82312	39425	20887	0	0	14.6	2403	2786
14	slow	1000	23000	88625	40425	25000	0	0	15.1	3231	3567
15	slow	1000	24400	94937	41425	28912	0	0	15.7	3747	4136
16	slow	1000	23400	101187	42425	35162	0	0	15.3	917	1264
20	slow	1000	24000	126500	46625	55675	0	0	15.6	633	4136
25	slow	1000	23925	158062	51425	82512	0	0	15.5	699	4240
30	slow	1000	23400	189500	56625	109275	0	0	15.4	940	1328
35	slow	1000	23400	221062	61425	136037	0	0	15.3	1064	1396
40	slow	1000	22800	253062	66625	163437	0	0	15.0	1466	1831
45	slow	1000	22541	284812	71625	190446	0	0	14.9	1743	2072
50	recover	12000	0	316500	116437	200000	0	0	5.5	7	10
55	recover	12000	0	348312	148250	200000	0	0	5.5	7	17
60	recover	12000	0	379812	179812	200000	0	0	5.4	7	22
65	recover	12000	0	411187	211125	200000	0	0	5.4	7	14
70	recover	12000	0	442750	242687	200000	0	0	5.5	7	17
```

So sánh trong pha chậm (40 giây, khoảng 253.000 đến 263.000 điểm đến tuỳ lần chạy, sink nhận được khoảng 40.000):

| | unbounded | drop-newest | coalesce |
|---|---|---|---|
| Độ sâu đỉnh | 221.012 | 30.000 (chạm trần) | 24.925 |
| Bộ nhớ sống đỉnh | 97,5 MB | 17,3 MB | 16,0 MB |
| Tuổi điểm giao, p99 | 35.449 ms và còn tăng | 30.567 ms | 4.240 ms |
| Điểm đã giao nhưng đã lỗi thời | 88,7% | 88,3% | 5,0% |
| Điểm mất (có đếm) | 0 | 183.305 (bỏ điểm mới) | 190.446 (gộp điểm cũ) |
| Sau 70 giây | còn 92.612 điểm trong hàng, tuổi vẫn trên 15 giây | hàng rỗng | hàng rỗng, tuổi 7 ms |

Số cho thấy: cả ba cùng giao ~40.000 điểm trong pha chậm vì sink chỉ có 1.000 điểm/s. Khác nhau là **điểm nào** được giao. `unbounded` giữ mọi thứ, bộ nhớ tăng đều (khoảng 2,3 MB mỗi giây) và giao điểm cũ dần tới 35 giây. `drop-newest` chặn được bộ nhớ nhưng giao đúng những điểm cũ nhất (ngược với thứ khách cần). `coalesce` giao điểm mới nhất của tài xế, tuổi bị chặn ở cỡ khoảng cách giữa hai lần báo của chính tài xế đó (4 giây) cộng thời gian chờ ngắn. Bất ngờ: coalesce chưa làm gì trong 4 giây đầu của pha chậm (`coalesced` vẫn 0 tới giây 9) vì phải đợi tài xế báo điểm thứ hai mới có cái để gộp; và coalesce **mất nhiều điểm hơn** drop-newest (190.446 so với 183.305): nó đổi lịch sử đầy đủ lấy độ tươi. Hệ quả trực tiếp cho LOC-20: điểm trung gian bị gộp thì không bao giờ tới Kafka và không có trong lịch sử.

#### Tắt êm: xả hàng đợi trong hạn

Hàng đợi có 20.000 điểm của 20.000 tài xế, sink 2.000 điểm/s, lô 100; ba hạn tắt khác nhau:

```bash
examples/toi-noi-location/go/bin/loc13_backpressure --mode shutdown
```

```text
$ examples/toi-noi-location/go/bin/loc13_backpressure --mode shutdown
# backlog=20000 điểm, sink 2000 điểm/s, lô 100
D	deadline	elapsed_ms	sink_got	dropped_at_shutdown	lost_in_flight	late_offer_result
D	1s	1001	1900	18000	100	Closed
D	5s	5000	9500	10400	100	Closed
D	15s	10434	20000	0	0	Closed
```

Hạn 1 giây xả được 1.900 điểm, bỏ 18.000 (có đếm vào `shutdown_dropped`), cộng 100 điểm của lô đang gửi dở bị huỷ (`lost_in_flight`); tổng bằng 20.000 (hạn 5 giây: 9.500 + 10.400 + 100). Hạn 15 giây vừa đủ xả hết (10,4 giây thực tế). Sau `Shutdown`, `Offer` trả `Closed`. Thứ tự tắt trong `location-ingest`: ngừng nhận kết nối, đóng kết nối WebSocket (`GoingAway`), xả hàng đợi trong hạn, flush Kafka, in bộ đếm cuối.

#### Kafka thật bị tạm dừng 25 giây

Ingest 25.000 kết nối, 6.250 điểm/s, hàng đợi 30.000, hai chính sách. Trong khi loadgen chạy, `docker pause` Kafka 25 giây rồi `docker unpause` (chỉ container `toi-noi-location-kafka` của lab này):

```bash
examples/toi-noi-location/go/bin/location-ingest --sink kafka --listen '127.0.0.1:8081,[::1]:8083' --policy coalesce --queue-cap 30000     # rồi --policy drop-newest
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --rate 6250 --batch 3 --duration 100s --report-every 5s
(sleep 45; docker pause toi-noi-location-kafka; sleep 25; docker unpause toi-noi-location-kafka) &     # chạy song song, 45 giây sau khi bắt đầu loadgen
```

Giờ thực của `docker pause`/`unpause`: coalesce 20:07:08 và 20:07:34 (26 giây), drop-newest 20:25:01 và 20:25:26 (25 giây). Các dòng 5 giây (cột `win_p50` bỏ cho gọn; file raw `loc13k_*.txt` giữ nguyên):

`coalesce`:

```text
  t= 25s acked_pts_s=6232 win_p99=13.1ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=168471 heap_mb=641
  t= 30s acked_pts_s=6272 win_p99=17.6ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=182280 heap_mb=691
  t= 35s acked_pts_s=6277 win_p99=18.1ms errs=0 q_depth=11672 q_coalesced=17344 q_dropped=0 kafka_delivered=182280 heap_mb=730
  t= 40s acked_pts_s=6250 win_p99=31.9ms errs=0 q_depth=22089 q_coalesced=38178 q_dropped=0 kafka_delivered=182280 heap_mb=411
  t= 45s acked_pts_s=6257 win_p99=13.8ms errs=0 q_depth=27000 q_coalesced=64551 q_dropped=0 kafka_delivered=182280 heap_mb=448
  t= 50s acked_pts_s=6227 win_p99=10.8ms errs=0 q_depth=27000 q_coalesced=95685 q_dropped=0 kafka_delivered=182280 heap_mb=485
  t= 55s acked_pts_s=6233 win_p99=9.8ms errs=0 q_depth=7284 q_coalesced=125568 q_dropped=0 kafka_delivered=202841 heap_mb=543
  t= 60s acked_pts_s=6272 win_p99=11.3ms errs=0 q_depth=0 q_coalesced=136126 q_dropped=0 kafka_delivered=231189 heap_mb=591
  t= 65s acked_pts_s=6212 win_p99=8.6ms errs=0 q_depth=0 q_coalesced=136126 q_dropped=0 kafka_delivered=262392 heap_mb=641
  t= 70s acked_pts_s=6293 win_p99=7.4ms errs=0 q_depth=9 q_coalesced=136126 q_dropped=0 kafka_delivered=293919 heap_mb=692
```

Dòng `STEP` và bộ đếm cuối của ingest (`loc13k_coalesce_ingest.json`):

```text
STEP	6250	6253	2084	0.52	7.62	19.96	393.41	0	16	583.3	1097.2	25012	27000	0	136126	22	687.3
queue/kafka cuối: {"queue": {"depth": 0, "max_depth": 27000, "offered": 643902, "coalesced": 136126, "stale_ignored": 0, "dropped": 0, "evicted": 0, "taken": 507776, "shutdown_dropped": 0}, "kafka": {"delivered": 481494, "failed": 20000}, "max_rss_mb": 1098}
```

`drop-newest`:

```text
  t= 25s acked_pts_s=6236 win_p99=16.4ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=164787 heap_mb=642
  t= 30s acked_pts_s=6280 win_p99=7.4ms errs=0 q_depth=15267 q_coalesced=0 q_dropped=0 kafka_delivered=164787 heap_mb=686
  t= 35s acked_pts_s=6270 win_p99=12.8ms errs=0 q_depth=30000 q_coalesced=0 q_dropped=16620 kafka_delivered=164787 heap_mb=724
  t= 40s acked_pts_s=6247 win_p99=14.5ms errs=0 q_depth=30000 q_coalesced=0 q_dropped=47850 kafka_delivered=164787 heap_mb=411
  t= 45s acked_pts_s=6259 win_p99=25.6ms errs=0 q_depth=30000 q_coalesced=0 q_dropped=79146 kafka_delivered=164787 heap_mb=449
  t= 50s acked_pts_s=6227 win_p99=22.0ms errs=0 q_depth=0 q_coalesced=0 q_dropped=108090 kafka_delivered=216894 heap_mb=511
  t= 55s acked_pts_s=6236 win_p99=19.5ms errs=0 q_depth=0 q_coalesced=0 q_dropped=108090 kafka_delivered=248136 heap_mb=561
  t= 60s acked_pts_s=6274 win_p99=21.0ms errs=0 q_depth=0 q_coalesced=0 q_dropped=108090 kafka_delivered=279468 heap_mb=611
  t= 65s acked_pts_s=6220 win_p99=38.9ms errs=0 q_depth=0 q_coalesced=0 q_dropped=108090 kafka_delivered=310551 heap_mb=661
  t= 70s acked_pts_s=6281 win_p99=13.1ms errs=0 q_depth=0 q_coalesced=0 q_dropped=108090 kafka_delivered=341937 heap_mb=712
```

```text
STEP	6250	6250	2083	0.50	8.01	17.21	95.83	0	17	596.3	1100.2	25010	30000	108090	0	23	767.6
queue/kafka cuối: {"queue": {"depth": 0, "max_depth": 30000, "offered": 643818, "coalesced": 0, "stale_ignored": 0, "dropped": 108090, "evicted": 0, "taken": 535728, "shutdown_dropped": 0}, "kafka": {"delivered": 535728, "failed": 0}, "max_rss_mb": 1104}
```

Số cho thấy: trong lúc Kafka đứng, `kafka_delivered` đóng băng (182.280 và 164.787), hàng đợi đầy lên (27.000 điểm ở coalesce, 30.000 ở drop-newest) và gộp/bỏ từ đó (`q_coalesced` lên 136.126, hoặc `q_dropped` lên 108.090), nhưng **độ trễ ack phía app không đổi** (`win_p99` trong các cửa sổ có Kafka đứng nằm trong khoảng 10 đến 32 ms, ngang các cửa sổ khác) và `acked_pts_s` giữ 6.2xx: goroutine đọc WebSocket không bị kẹt, đúng ý đồ. Sau khi Kafka chạy lại, hàng rỗng sau 5 đến 15 giây. Bộ nhớ ingest không phình thêm (đỉnh RSS 1.097 và 1.100 MB so với 1.020 MB khi không có sự cố).

Bất ngờ cần ghi: **bộ đệm nội bộ của client Kafka là hàng đợi thứ hai**, và lần chạy coalesce báo `kafka.failed = 20000`, đúng bằng `MaxBufferedRecords(20000)`, trong khi lần drop-newest báo `failed = 0` với cùng thời gian dừng (26 so với 25 giây) và `RecordDeliveryTimeout` 30 giây. Nguyên nhân của con số đó (và của khoảng chênh `taken − (delivered + failed)` ở hai lần chạy này) được điều tra ở mục «Điều tra `taken − (delivered + failed)` và `kafka.failed`» ngay dưới. Điều chắc chắn: bộ đếm của hàng đợi phía trước không thấy phần mất này, nên ingest phải xuất cả `queue.*` lẫn `kafka.failed` (nay `/debug/stats` còn xuất `kafka.buffered` và `kafka.failed_<nguyên nhân>`). Với coalesce, `drop-newest` ở trên, so sánh tuổi điểm đã giao trong lần Kafka thật **chưa đo** (chỉ có ở thí nghiệm offline).

#### Điều tra `taken − (delivered + failed)` và `kafka.failed`

Hai điều chưa khớp sau các lần chạy trên: (1) ở ba trong bốn bộ đếm cuối của ingest, `queue.taken − (kafka.delivered + kafka.failed)` khác 0; (2) `kafka.failed = 20000` ở một trong hai lần dừng Kafka. Các lần chạy bên dưới dùng ingest đã thêm `kafka.buffered` (`Sink.Buffered()`, bằng `BufferedProduceRecords()` của client) và `kafka.failed_<nguyên nhân>` (nhóm lỗi của promise, lần đầu mỗi nhóm còn ghi log kèm lỗi gốc); `uptime` ghi trong từng file raw.

**Bốn bộ đếm cuối đã có** (đọc từ `loc12_a_ingest.json`, `loc12_b_ingest.json`, `loc13k_coalesce_ingest.json`, `loc13k_drop-newest_ingest.json`; đều là `/debug/stats` chụp ngay sau khi loadgen kết thúc, ingest vẫn đang chạy):

| Lần chạy | `taken` | `delivered` | `failed` | `taken − (delivered + failed)` |
|---|---|---|---|---|
| LOC-12 A (lô 3) | 393.561 | 388.746 | 0 | 4.815 |
| LOC-12 B (lô 1) | 393.642 | 393.169 | 0 | 473 |
| LOC-13 coalesce, dừng 26 giây | 507.776 | 481.494 | 20.000 | 6.282 |
| LOC-13 drop-newest, dừng 25 giây | 535.728 | 535.728 | 0 | 0 |

`taken` tăng trong `Queue.TakeBatch`, trước khi `Sink.Send` đưa bản ghi cho producer; `delivered` và `failed` chỉ tăng trong promise, sau khi broker xác nhận hoặc lỗi. Bản ghi nằm giữa hai mốc đó (đợi linger 5 ms, đang trên đường tới broker) là «đang bay». Ở `loc12_a_ingest.json`, lúc chụp `conns_open` còn 766: loadgen đang đóng kết nối, tức bộ đếm được chụp giữa chừng.

Muốn phân biệt «đang bay» với «mất» hoặc «bộ đếm sai» thì phải chụp lại sau khi mọi thứ lắng xuống. Chạy lại cấu hình A (`p02-go-raw/loc12_final_snapshot.txt`): ngay khi loadgen xong khoảng chênh là 3.672, một giây sau là 0 và giữ nguyên 0 trong 18 giây; sau `SIGTERM` (ngừng nhận kết nối, xả hàng đợi, `Flush` Kafka) bộ đếm cuối là `taken = delivered = 394.002`, `failed = 0`.

```text
# 22:52:39.879  {"queue": {"depth": 0, "offered": 393870, ..., "taken": 393870}, "kafka": {"delivered": 390198, "failed": 0}, "taken_minus_delivered_failed": 3672}
# 22:52:41.012  {"queue": {"depth": 0, "offered": 394002, ..., "taken": 394002}, "kafka": {"delivered": 394002, "failed": 0}, "taken_minus_delivered_failed": 0}
cuối sau SIGTERM: "taken": 394002, "kafka": {"delivered": 394002, "failed": 0}
```

Với `kafka.buffered` thì khoảng chênh trùng đúng số bản ghi đang bay ở năm lần chạy sau (`loc13k_failed_runs.txt`, snapshot đầu tiên sau loadgen). Từng cặp (khoảng chênh / `buffered`): 3.897 / 3.897, 4.965 / 4.968, 4.563 / 4.563 (lần này còn có `failed = 20000`), 5.919 / 5.919, 5.139 / 5.139; lệch 3 ở lần thứ hai vì `taken` và `BufferedProduceRecords()` không được đọc cùng một lúc. Một giây sau cả năm lần đều về 0 và bộ đếm cuối sau `SIGTERM` cũng 0. Ở cả sáu bộ đếm cuối còn có: `offered − coalesced − dropped − evicted − stale_ignored − shutdown_dropped − taken − depth = 0`, tức hàng đợi phía trước không làm rơi điểm nào ngoài những gì nó đếm.

Kết luận cho (1): **không có bản ghi nào mất và không có lỗi ở bộ đếm; đó là bản ghi đang bay lúc chụp**. Bốn bộ đếm cũ không có `buffered` nên không tự chúng chứng minh được điều này; bằng chứng là cơ chế trên, lần chạy lặp lại cùng cấu hình, và các lần chụp lại sau đó. Lần drop-newest ở bảng trên chênh 0, cách hiểu hợp lý là lúc chụp tình cờ không còn bản ghi nào đang bay (không kiểm riêng). Hệ quả cho cách đọc số: đối soát giữa `queue.taken` và `kafka.delivered + kafka.failed` phải làm sau khi flush, hoặc cộng thêm `kafka.buffered`.

**`kafka.failed = 20000`.** Lặp lại thí nghiệm dừng Kafka với ingest mới (`p02-go-raw/loc13k_failed_runs.txt`), cùng lệnh như mục trên, chỉ đổi thời gian dừng:

| Chính sách | Dừng Kafka (giây) | `kafka.failed` | Nguyên nhân (`failed_*`) |
|---|---|---|---|
| coalesce | 26 | 0 | |
| coalesce | 28 | 0 | |
| coalesce | 30 | 20.000 | `record_timeout` = 20.000 |
| drop-newest | 30 | 0 | |
| drop-newest | 34 | 0 | |

Lần coalesce 30 giây tái hiện đúng con số 20.000, và log của ingest ghi lỗi gốc `records have timed out before they were able to be produced`: đó là `kgo.ErrRecordTimeout`, tức `RecordDeliveryTimeout(30s)` hết hạn ở các bản ghi chờ gửi trong lúc broker đứng. Tài liệu của franz-go nói khi một bản ghi hết hạn thì **mọi bản ghi đang đệm cùng partition bị coi là lỗi theo**; bộ đệm lúc đó đầy (20.000) nên toàn bộ 20.000 bản ghi lỗi cùng lúc, và con số bằng `MaxBufferedRecords` là hệ quả của đó, không phải sự trùng hợp. Nhưng hai dòng drop-newest (30 và 34 giây, không ngắn hơn hạn 30 giây) không lỗi. Mỗi ô chỉ một lần chạy nên bảng này chưa đủ để nói chính sách có liên quan hay không; thí nghiệm sau bỏ hẳn hàng đợi để trả lời.

Để cô lập khỏi hàng đợi và 25.000 kết nối, `kafka_lab timeouts` chạy 8 producer cùng cấu hình `NewSink` (linger 5 ms, `MaxBufferedRecords(20000)`, 6.200 bản ghi/giây mỗi producer) chỉ khác `RecordDeliveryTimeout`, cùng chịu một lần `docker pause` (`p02-go-raw/loc13k_timeouts.txt`; cột là hạn 20, 25, 30, 30, 35, 40, 60 giây và không hạn):

```bash
examples/toi-noi-location/go/bin/kafka_lab timeouts 85s 20,25,30,30,35,40,60,0
(sleep 20; docker pause toi-noi-location-kafka; sleep <P>; docker unpause toi-noi-location-kafka) &
```

| Dừng (giây) | Số producer lỗi trên 8 | Producer lỗi (số bản ghi lỗi) |
|---|---|---|
| 26 | 0 | |
| 30 | 2 | hạn 25 giây (19.708), một trong hai producer hạn 30 giây (19.628) |
| 34, lần 1 | 4 | hạn 20 giây (19.717), hạn 25 giây (17), cả hai hạn 30 giây (19.742 và 19.717) |
| 34, lần 2 | 0 | |

Đọc được từ đây: mọi lỗi đều là `record_timeout`; producer hạn 35, 40, 60 giây và không hạn không lỗi ở cả bốn lần chạy; không producer nào lỗi khi dừng 26 giây (kể cả hạn 20 giây); và **hai producer hạn 30 giây giống hệt nhau, cùng một lần dừng 30 giây, một lỗi 19.628 bản ghi còn một không lỗi**; cùng thời gian dừng 34 giây, hai lần chạy cho kết quả khác nhau (4 producer lỗi so với 0). Vậy nguyên nhân là hạn giao `RecordDeliveryTimeout` của client Kafka hết hạn khi broker đứng cỡ 30 giây trở lên, và việc nó xảy ra hay không ở vùng 30 đến 34 giây là một cuộc đua thời gian bên trong client; 8 producer này không có hàng đợi hay chính sách nào phía trước mà vẫn tái hiện, nên chính sách coalesce hay drop-newest không phải là nguyên nhân. Số `failed` ở lần coalesce gốc không còn là bí ẩn về loại lỗi, nhưng giữ hai điều **chưa giải quyết**: (a) thời điểm chính xác franz-go xét hạn (tài liệu: chỉ trước khi ghi một request hoặc sau một phản hồi produce, và chỉ khi an toàn với idempotence) chưa được lần theo, nên chưa giải thích được vì sao cùng hạn, cùng thời gian dừng, hai producer cho kết quả khác nhau; (b) lần coalesce gốc lỗi ở thời gian dừng danh nghĩa 26 giây (hiệu hai mốc giờ ghi theo giây, độ trễ lệnh `docker pause`/`unpause` lúc máy rất nặng không ghi), còn các lần lặp 26 giây ở trên (ingest, 8 producer) đều không lỗi; khả năng cao là thời gian Kafka thực sự đóng băng dài hơn 26 giây, nhưng chưa chứng minh được. Hạn 30 giây là tham số chọn cho lab; tăng nó lên trên thời gian sự cố dự kiến là hướng sửa hiển nhiên, chưa thử trong lab này.

Test liên quan (`-race`): `go test -race -v ./internal/backpressure` (`p02-go-raw/tests_backpressure.txt`) gồm bảo toàn số điểm (vào = ra + mất có đếm) với nhiều producer và consumer ở cả ba chính sách, coalesce chỉ bật khi vượt ngưỡng, tắt êm trong hạn và hết hạn.


### LOC-14

Hai phần: bộ test bảng điểm xấu (không cần hạ tầng) và một lần chạy toàn tuyến có tiêm lỗi vào ingest.

#### Test bảng (16 ca)

```bash
go -C examples/toi-noi-location/go test -race -count=1 -v -run 'TestBangDiemXau|TestSkew|TestChuoiNhay' ./internal/validate
```

```text
$ go -C examples/toi-noi-location/go test -race -count=1 -v -run TestBangDiemXau|TestSkew|TestChuoiNhay ./internal/validate
--- PASS: TestBangDiemXau (0.00s)
    --- PASS: TestBangDiemXau/điểm_tốt (0.00s)
    --- PASS: TestBangDiemXau/toạ_độ_ngoài_miền (0.00s)
    --- PASS: TestBangDiemXau/null_island (0.00s)
    --- PASS: TestBangDiemXau/giờ_ở_tương_lai (0.00s)
    --- PASS: TestBangDiemXau/quá_cũ (0.00s)
    --- PASS: TestBangDiemXau/đến_sau_điểm_mới_hơn (0.00s)
    --- PASS: TestBangDiemXau/trùng_giờ_trùng_seq (0.00s)
    --- PASS: TestBangDiemXau/seq_về_0_nhưng_giờ_mới_(app_khởi_động_lại) (0.00s)
    --- PASS: TestBangDiemXau/đồng_hồ_nhanh_90_s:_hiệu_chỉnh_và_gắn_cờ (0.00s)
    --- PASS: TestBangDiemXau/accuracy_250_m (0.00s)
    --- PASS: TestBangDiemXau/app_khai_báo_giả_lập (0.00s)
    --- PASS: TestBangDiemXau/tốc_độ_khai_báo_80_m/s (0.00s)
    --- PASS: TestBangDiemXau/nhảy_3_km_trong_1_s (0.00s)
    --- PASS: TestBangDiemXau/nhảy_200_m_trong_1_s:_trong_ngưỡng_teleport_nhưng_quá_nhanh (0.00s)
    --- PASS: TestBangDiemXau/nhảy_30_m_trong_1_s_(jitter,_trong_sai_số) (0.00s)
    --- PASS: TestBangDiemXau/khai_báo_đứng_yên_nhưng_đã_đi_100_m_trong_5_s (0.00s)
--- PASS: TestSkewChiHieuChinhKhiVuotNguong (0.00s)
--- PASS: TestChuoiNhayRoiVeNoiGocGanCoCaHaiLan (0.00s)
PASS
ok  	toinoi/location/internal/validate	1.281s
(exit=0)
```

Ngưỡng mặc định (`validate.Default()`): lệch đồng hồ trong 3 giây coi là độ trễ mạng và không hiệu chỉnh; điểm cũ hơn 5 phút hoặc ở tương lai quá 5 giây (sau hiệu chỉnh) bị loại; accuracy trên 100 m gắn `POOR_ACCURACY`; tốc độ trên 42 m/s gắn `SPEED_IMPLAUSIBLE`; tốc độ suy ra trên 100 m/s kèm nhảy từ 500 m gắn `TELEPORT`; khai báo tốc độ lệch hơn 15 m/s so với suy ra gắn `SPEED_MISMATCH` (chỉ khi hai điểm cách nhau không quá 10 giây). Quãng nhảy được trừ sai số accuracy của hai điểm trước khi tính tốc độ suy ra.

Loại điểm (không đi tiếp): toạ độ ngoài miền hoặc (0,0), giờ ở tương lai, quá cũ, đến sau điểm mới hơn, trùng giờ trùng seq. Còn lại đi tiếp, có thể kèm cờ. `seq` về 0 nhưng giờ mới hơn vẫn được nhận (app khởi động lại).

#### Chạy toàn tuyến có tiêm lỗi

Tiêm bằng loadgen: 0,2% điểm bị nhảy 5 km (`--fault-rate`), 2% tài xế có đồng hồ nhanh 90 giây (`--skew-fraction`), 5% lô bị đảo thứ tự điểm (`--reorder-fraction`), 1% điểm tự khai giả lập (`--mock-fraction`). Ingest sink `discard`, 2.000 tài xế, 500 điểm/s, seed 1, 30 giây:

```bash
examples/toi-noi-location/go/bin/location-ingest --sink discard
examples/toi-noi-location/go/bin/loadgen --drivers 2000 --rate 500 --batch 3 --duration 30s --report-every 15s --seed 1 --fault-rate 0.002 --skew-fraction 0.02 --reorder-fraction 0.05 --mock-fraction 0.01
curl -s http://127.0.0.1:8081/debug/stats     # chọn forwarded, flagged, rejected, flag_counts
```

Output (`p02-go-raw/loc14.txt`):

```text
# loadgen drivers=2000 seed=1 batch=3 urls=1 viewers=0 slow_viewers=0
# kết nối: 2000/2000 thành công, 0 lỗi, mất 0.3s
# khởi động 14s ở 500 điểm/s (không tính)
STEP	target_pts_s	achieved_pts_s	msgs_s	p50_ms	p95_ms	p99_ms	max_ms	errs	ingest_cpu_pct	ingest_heap_mb	ingest_rss_mb	ingest_goroutines	q_max_depth	q_dropped	q_coalesced	loadgen_cpu_pct	loadgen_rss_mb
  t= 15s target=500 acked_pts_s=494 win_p50=0.4ms win_p99=9.8ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=8185 heap_mb=36
  t= 30s target=500 acked_pts_s=503 win_p50=0.5ms win_p99=14.8ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=15457 heap_mb=45
STEP	500	499	166	0.40	6.25	14.13	34.66	0	2	45.3	83.4	2007	14	0	0	4	81.8
$ curl -s http://127.0.0.1:8081/debug/stats | python3 -c 'chọn forwarded, flagged, rejected, flag_counts'
points 16572
forwarded 15982
flagged 572
rejected {"bad_coord": 0, "duplicate": 0, "future": 0, "other": 0, "out_of_order": 590, "too_old": 0}
flag_counts {"POINT_FLAG_CLOCK_SKEW": 378, "POINT_FLAG_MOCK_REPORTED": 163, "POINT_FLAG_POOR_ACCURACY": 0, "POINT_FLAG_SPEED_IMPLAUSIBLE": 32, "POINT_FLAG_SPEED_MISMATCH": 36, "POINT_FLAG_TELEPORT": 32}
```

Số cho thấy: 16.572 điểm vào, 15.982 đi tiếp (572 mang cờ) và 590 bị loại do đến sau điểm mới hơn. Số điểm bị loại khớp ước lượng: lô 3 điểm bị đảo thì điểm mới nhất vào trước, hai điểm cũ còn lại bị loại, tức khoảng 2 điểm × 5% × 5.500 lô. 32 cờ `TELEPORT` khớp 0,2% × 16.572 điểm. Cờ giả lập 163 (khoảng 1%), lệch đồng hồ 378 (2% tài xế, mọi điểm của họ). Bất ngờ: `SPEED_MISMATCH` có 36 điểm, nhiều hơn số teleport; một phần do chính các cú nhảy (cú nhảy làm tốc độ suy ra lệch), phần còn lại là điểm hợp lệ của simulator, như ở LOC-20 nơi 32 điểm bị gắn cờ dù không tiêm lỗi (đèn đỏ mô phỏng đưa tốc độ về 0 đột ngột). Tức là ngưỡng này có dương tính giả trên dữ liệu mô phỏng cỡ 0,01%; trên dữ liệu thật phải hiệu chỉnh lại. Validator chỉ gắn cờ; không có đoạn mã nào kết luận gian lận.

Quyền riêng tư: log của ingest và hub không ghi toạ độ; thống kê chỉ có bộ đếm.


### LOC-19

Hub đọc `location.updates.v1` (consumer không nhóm, bắt đầu từ cuối topic), gán chuyến qua `/lab/assign` (thay cho sự kiện của Dispatch), khách nghe qua `/track?trip=...`. Tải: toàn bộ 25.000 tài xế ở 6.250 điểm/s qua ingest và Kafka thật, **200 khách xem** (200 tài xế đầu tiên được gán chuyến), trong đó 10 khách đọc chậm (mỗi khung xử lý 6 giây rồi mới ack). 60 giây đo.

```bash
examples/toi-noi-location/go/bin/location-ingest --sink kafka --listen '127.0.0.1:8081,[::1]:8083'
examples/toi-noi-location/go/bin/tracking-hub
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --rate 6250 --batch 3 --duration 60s --report-every 20s --hub http://127.0.0.1:8082 --viewers 200 --slow-viewers 10 --slow-delay 6s
curl -s http://127.0.0.1:8082/debug/stats
```

Lần thứ hai y hệt nhưng `--batch 1`. Hai độ trễ đầu cuối đều tính phía khách, cùng một máy nên cùng đồng hồ: `send_to_recv` từ lúc app gửi lô tới lúc khách nhận khung; `sample_to_recv` từ lúc lấy mẫu điểm tới lúc khách nhận (gồm cả thời gian điểm chờ trong lô ở app).

#### Lô 3 điểm (`p02-go-raw/loc19_b3.txt`)

```text
# loadgen drivers=25000 seed=1 batch=3 urls=2 viewers=200 slow_viewers=10
# kết nối: 25000/25000 thành công, 0 lỗi, mất 2.9s
# khởi động 14s ở 6250 điểm/s (không tính)
STEP	target_pts_s	achieved_pts_s	msgs_s	p50_ms	p95_ms	p99_ms	max_ms	errs	ingest_cpu_pct	ingest_heap_mb	ingest_rss_mb	ingest_goroutines	q_max_depth	q_dropped	q_coalesced	loadgen_cpu_pct	loadgen_rss_mb
  t= 20s target=6250 acked_pts_s=6263 win_p50=0.6ms win_p99=23.2ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=137352 heap_mb=592
  t= 40s target=6250 acked_pts_s=6258 win_p50=0.4ms win_p99=13.1ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=262515 heap_mb=412
  t= 60s target=6250 acked_pts_s=6247 win_p50=0.5ms win_p99=15.2ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=387450 heap_mb=613
STEP	6250	6256	2084	0.45	7.25	16.39	60.00	0	17	613.4	1053.9	25010	171	0	0	21	684.7
VIEWERS normal=190 frames=1005 skipped_by_hub=2001
VIEWERS e2e_send_to_recv   p50=10ms p95=33ms p99=59ms max=93ms
VIEWERS e2e_sample_to_recv p50=10ms p95=34ms p99=72ms max=8015ms
VIEWERS slow=10 frames=51 skipped_by_hub=102 sample_to_recv p50=8ms p95=28ms p99=8057ms max=8021ms
```

```text
$ curl -s http://127.0.0.1:8082/debug/stats
{
    "coalesced": 2103,
    "denied": 0,
    "dispatched": 3162,
    "frames_sent": 1056,
    "held_back_flagged": 4,
    "ignored_unassigned": 385601,
    "ingest_to_write": {
        "count": 1056,
        "max_ms": 79,
        "p50_ms": 8.007,
        "p95_ms": 28.21,
        "p99_ms": 53.607
    },
    "proc": {
        "cpu_user_s": 1.797018,
        "cpu_sys_s": 0.726422,
        "goroutines": 13,
        "heap_alloc_mb": 6.515556335449219,
        "go_sys_mb": 22.53345489501953,
        "max_rss_mb": 31.296875
    },
    "sample_to_write": {
        "count": 1056,
        "max_ms": 8020,
        "p50_ms": 9.059,
        "p95_ms": 32.715,
        "p99_ms": 65.315
    },
    "slow_kicked": 0,
    "trips_assigned": 200,
    "write_errors": 0
}
```

#### Lô 1 điểm (`p02-go-raw/loc19_b1.txt`)

```text
# loadgen drivers=25000 seed=1 batch=1 urls=2 viewers=200 slow_viewers=10
# kết nối: 25000/25000 thành công, 0 lỗi, mất 1.8s
# khởi động 6s ở 6250 điểm/s (không tính)
STEP	target_pts_s	achieved_pts_s	msgs_s	p50_ms	p95_ms	p99_ms	max_ms	errs	ingest_cpu_pct	ingest_heap_mb	ingest_rss_mb	ingest_goroutines	q_max_depth	q_dropped	q_coalesced	loadgen_cpu_pct	loadgen_rss_mb
  t= 20s target=6250 acked_pts_s=6265 win_p50=0.8ms win_p99=32.7ms errs=0 q_depth=1 q_coalesced=0 q_dropped=0 kafka_delivered=137795 heap_mb=431
  t= 40s target=6250 acked_pts_s=6250 win_p50=0.7ms win_p99=17.6ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=262800 heap_mb=423
  t= 60s target=6250 acked_pts_s=6250 win_p50=0.5ms win_p99=22.0ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=387668 heap_mb=415
STEP	6250	6257	6256	0.66	10.00	24.32	153.13	0	30	415.7	1087.7	25012	223	0	0	35	683.0
VIEWERS normal=190 frames=2959 skipped_by_hub=0
VIEWERS e2e_send_to_recv   p50=10ms p95=34ms p99=62ms max=114ms
VIEWERS e2e_sample_to_recv p50=10ms p95=34ms p99=62ms max=114ms
VIEWERS slow=10 frames=104 skipped_by_hub=30 sample_to_recv p50=6141ms p95=8057ms p99=10062ms max=10042ms
```

```text
$ curl -s http://127.0.0.1:8082/debug/stats
{
    "coalesced": 33,
    "denied": 0,
    "dispatched": 3160,
    "frames_sent": 3072,
    "held_back_flagged": 13,
    "ignored_unassigned": 391069,
    "ingest_to_write": {
        "count": 3073,
        "max_ms": 4039,
        "p50_ms": 8.007,
        "p95_ms": 34.371,
        "p99_ms": 2071.842
    },
    "proc": {
        "cpu_user_s": 1.9754209999999999,
        "cpu_sys_s": 0.880059,
        "goroutines": 13,
        "heap_alloc_mb": 1.926116943359375,
        "go_sys_mb": 23.04907989501953,
        "max_rss_mb": 32.125
    },
    "sample_to_write": {
        "count": 3073,
        "max_ms": 4039,
        "p50_ms": 10,
        "p95_ms": 38.887,
        "p99_ms": 2071.842
    },
    "slow_kicked": 0,
    "trips_assigned": 200,
    "write_errors": 1
}
```

So với SLO kiến trúc §14.1 (lan truyền realtime p95 dưới 2 đến 3 giây; độ tươi vị trí tài xế p95 dưới 5 đến 10 giây):

| Đo | Lô 3 điểm | Lô 1 điểm |
|---|---|---|
| Gửi → nhận, p50 / p95 / p99 | 10 / 33 / 59 ms | 10 / 34 / 62 ms |
| Lấy mẫu → nhận, p95 / p99 / max | 34 / 72 / 8.015 ms | 34 / 62 / 114 ms |
| Hub: ingest nhận → hub ghi, p95 / p99 (gồm cả 10 khách chậm, khung của họ chờ ack) | 28 / 54 ms | 34 / 2.072 ms |
| Khung gửi / điểm hub nhận cho chuyến có gán | 1.056 / 3.162 | 3.072 / 3.160 |

Số cho thấy: độ trễ của riêng đường ống (app gửi → Kafka → hub → khách) có p95 khoảng 34 ms trên loopback, nhỏ hơn mục tiêu hai bậc độ lớn, và gần như không phụ thuộc cách chia lô. Con số này **không** gồm mạng di động thật và không phải bằng chứng cho SLO. Với lô 3 điểm, ba điểm tới hub gần như cùng lúc nên ô chờ chỉ giữ điểm mới nhất: hub gộp 2.103 trong 3.162 điểm, khách thấy ít khung hơn (1.056) nhưng khung nào cũng là điểm mới nhất; điểm cũ nhất trong lô đã chờ ở app tối đa 8 giây, thể hiện ở `max` 8.015 ms của `sample_to_recv`. Bất ngờ: batching phía app cho tracking đáng giá về CPU của ingest (LOC-12) nhưng chỉ làm marker chậm khi điểm bị chọn là điểm đầu lô; hub đã tự chọn điểm cuối.

Khách đọc chậm (10 người, 6 giây mỗi khung): lô 1 điểm, 104 khung, hub gộp 30 điểm, tuổi điểm lúc nhận p50 6.141 ms, p99 10.062 ms: bị chặn ở khoảng hai khung bay (cửa sổ ack) thay vì tăng mãi. Ở lần chạy đầu của chính hub này, **trước khi có ack tầng ứng dụng**, cùng cấu hình cho `sample_to_recv` p50 10.062 ms, p99 20.089 ms và hub gộp 0 điểm: bộ đệm TCP nuốt các khung cũ nên hub không bao giờ thấy client chậm. Output đó không còn trong file (đã bị lần chạy sau ghi đè), con số chép từ log phiên làm việc; test `TestClientChamBiCoalesce` ghi lại hành vi hiện tại (2.001 điểm tới trong lúc client không ack: client nhận 3 khung, hub gộp 1.998).

Kiểm tra «chỉ mở sau khi gán» (test, `p02-go-raw/tests_hub.txt`):

```text
$ go -C examples/toi-noi-location/go test -race -count=1 -v ./internal/hub
--- PASS: TestChiMoSauKhiGan (0.01s)
--- PASS: TestVeKhongPhaiCuaChuyenNaoBiBoQua (0.00s)
--- PASS: TestDiemNghiVanKhongToiKhach (0.00s)
    hub_test.go:174: 2001 điểm tới trong lúc client không ack: client nhận 3 khung, hub gộp bỏ 1998
--- PASS: TestClientChamBiCoalesce (0.12s)
--- PASS: TestKetThucChuyenDongKetNoi (0.00s)
--- PASS: TestAssignChiChoAdmin (0.00s)
--- PASS: TestClientKhongAckBiNgat (0.26s)
PASS
ok  	toinoi/location/internal/hub	1.737s
(exit=0)
```

Chưa gán chuyến, khách khác dùng token hợp lệ của mình, token của chuyến khác (đoán trip id), và sau khi chuyến kết thúc, đều bị từ chối (403 hoặc 401); điểm `TELEPORT` bị giữ lại không tới khách; client không ack bị ngắt. Hợp đồng nội suy và ack: `examples/toi-noi-location/proto/location/v1/tracking-contract.md`.


### LOC-20

#### Giảm mẫu, Parquet theo ngày và thành phố, retention (không cần Kafka)

```bash
examples/toi-noi-location/go/bin/history-writer demo --out examples/toi-noi-location/go/out/history --id-level pseudonym
examples/toi-noi-location/go/bin/history-writer inspect --out examples/toi-noi-location/go/out/history
examples/toi-noi-location/go/bin/history-writer demo --out examples/toi-noi-location/go/out/history-raw --id-level raw --drivers 300 --minutes 5 --retention-span-days 0
examples/toi-noi-location/go/bin/history-writer demo --out examples/toi-noi-location/go/out/history-coarse --id-level coarse --drivers 300 --minutes 5 --retention-span-days 0
examples/toi-noi-location/go/bin/history-writer retention --out examples/toi-noi-location/go/out/history-retention --days 30 --dry-run --now 2026-11-06T12:00:00+07:00
examples/toi-noi-location/go/bin/history-writer retention --out examples/toi-noi-location/go/out/history-retention --days 7 --now 2026-11-06T12:00:00+07:00
```

Cấu hình `demo`: 2.000 tài xế × 30 phút, điểm mỗi 4 giây, seed 1, giờ bắt đầu 2026-11-06 08:00 +07; thí nghiệm retention ghi thêm 45 ngày (50 tài xế × 10 phút mỗi ngày, seed 2). Ngưỡng giảm mẫu: giữ khi di chuyển ≥ 60 m, hoặc đổi hướng ≥ 30° sau ≥ 4 giây, hoặc 60 giây không có điểm, hoặc điểm có cờ. Output đầy đủ ở `p02-go-raw/loc20_demo.txt`:

```text
$ examples/toi-noi-location/go/bin/history-writer demo --out examples/toi-noi-location/go/out/history --id-level pseudonym
mức định danh: pseudonym | 2000 tài xế × 30 phút, điểm mỗi 4 giây, seed 1
đầu vào: 900000 điểm, 53237116 byte protobuf (event)
không giảm mẫu: 900000 dòng, 11385353 byte Parquet (12.7 byte/dòng)
giảm mẫu:       239990 dòng (giữ 26.7%), 4071570 byte Parquet (17.0 byte/dòng), nhỏ hơn 2.8 lần so với không giảm mẫu
lý do giữ: first=2000 heartbeat=1876 move=152655 turn=83459
sai số vẽ lại quỹ đạo từ điểm đã giữ (khoảng cách điểm bị bỏ tới đoạn nối hai điểm giữ): p50=5.0 m p95=15.3 m p99=20.9 m max=44.7 m (655220 điểm bị bỏ)
cây thư mục đầu ra:
  date=2026-11-06/city=CT/part-1793972456393-0001.parquet  329423 byte
  date=2026-11-06/city=DN/part-1793972456467-0002.parquet  412738 byte
  date=2026-11-06/city=HCM/part-1793972456743-0003.parquet  1618533 byte
  date=2026-11-06/city=HN/part-1793972457012-0004.parquet  1236071 byte
  date=2026-11-06/city=HP/part-1793972457084-0005.parquet  280451 byte
  date=2026-11-06/city=NT/part-1793972457112-0006.parquet  194354 byte

retention: đã ghi 45 ngày (50 tài xế × 10 phút mỗi ngày) vào examples/toi-noi-location/go/out/history-retention, 86966 dòng
dry-run, giữ 30 ngày tính tới 2026-11-06: sẽ xoá 15 phân vùng ngày, cũ nhất 2026-09-23, mới nhất bị xoá 2026-10-07
đã xoá 15 phân vùng, 90 file, 801438 byte; còn 30 phân vùng ngày

$ examples/toi-noi-location/go/bin/history-writer inspect --out examples/toi-noi-location/go/out/history
schema (file đầu tiên):
  event_time     NOT NULL TIMESTAMP(isAdjustedToUTC=true,unit=MILLIS)
  received_time  NOT NULL TIMESTAMP(isAdjustedToUTC=true,unit=MILLIS)
  driver_ref     NULLABLE STRING
  id_level       NOT NULL INT(32,true)
  trip_id        NULLABLE STRING
  lat_e7         NOT NULL INT(32,true)
  lng_e7         NOT NULL INT(32,true)
  accuracy_dm    NULLABLE INT(32,true)
  speed_dmps     NULLABLE INT(32,true)
  heading_ddeg   NULLABLE INT(32,true)
  seq            NOT NULL INT(64,true)
  flags          NOT NULL INT(32,true)
  sample_reason  NOT NULL STRING
metadata: toinoi.schema_version=1 toinoi.id_level=1
driver_ref của hai dòng đầu: 015e3d6bec00c070, 015e3d6bec00c070 (id_level=1)
date=2026-11-06/city=CT      rows=19325    bytes=329423
date=2026-11-06/city=DN      rows=24341    bytes=412738
date=2026-11-06/city=HCM     rows=95761    bytes=1618533
date=2026-11-06/city=HN      rows=73077    bytes=1236071
date=2026-11-06/city=HP      rows=16356    bytes=280451
date=2026-11-06/city=NT      rows=11130    bytes=194354
tổng: 239990 dòng trong 6 phân vùng

$ examples/toi-noi-location/go/bin/history-writer demo --out examples/toi-noi-location/go/out/history-raw --id-level raw --drivers 300 --minutes 5 --retention-span-days 0   # chỉ để so sánh mức định danh
mức định danh: raw | 300 tài xế × 5 phút, điểm mỗi 4 giây, seed 1
không giảm mẫu: 22500 dòng, 307777 byte Parquet (13.7 byte/dòng)
giảm mẫu:       6459 dòng (giữ 28.7%), 130226 byte Parquet (20.2 byte/dòng), nhỏ hơn 2.4 lần so với không giảm mẫu
driver_ref của hai dòng đầu: DR-00009, DR-00009 (id_level=0)
tổng: 6459 dòng trong 6 phân vùng

$ examples/toi-noi-location/go/bin/history-writer demo --out examples/toi-noi-location/go/out/history-coarse --id-level coarse --drivers 300 --minutes 5 --retention-span-days 0   # chỉ để so sánh mức định danh
mức định danh: coarse | 300 tài xế × 5 phút, điểm mỗi 4 giây, seed 1
không giảm mẫu: 22500 dòng, 147434 byte Parquet (6.6 byte/dòng)
giảm mẫu:       6459 dòng (giữ 28.7%), 80655 byte Parquet (12.5 byte/dòng), nhỏ hơn 1.8 lần so với không giảm mẫu
driver_ref của hai dòng đầu: NULL, NULL (id_level=2)
tổng: 6459 dòng trong 6 phân vùng

$ examples/toi-noi-location/go/bin/history-writer retention --out examples/toi-noi-location/go/out/history-retention --days 30 --dry-run --now 2026-11-06T12:00:00+07:00   # sau khi demo đã xoá 15 phân vùng, còn lại 30 ngày: không còn gì để xoá
tổng: 0 phân vùng, 0 file, 0 byte (giữ 30 ngày tính tới 2026-11-06)

$ examples/toi-noi-location/go/bin/history-writer retention --out examples/toi-noi-location/go/out/history-retention --days 7 --now 2026-11-06T12:00:00+07:00
ĐÃ XOÁ date=2026-10-08 files=6 bytes=53053
ĐÃ XOÁ date=2026-10-09 files=6 bytes=54318
ĐÃ XOÁ date=2026-10-10 files=6 bytes=54619
ĐÃ XOÁ date=2026-10-11 files=6 bytes=52696
ĐÃ XOÁ date=2026-10-12 files=6 bytes=53282
ĐÃ XOÁ date=2026-10-13 files=6 bytes=51902
ĐÃ XOÁ date=2026-10-14 files=6 bytes=53345
ĐÃ XOÁ date=2026-10-15 files=6 bytes=53380
ĐÃ XOÁ date=2026-10-16 files=6 bytes=54693
ĐÃ XOÁ date=2026-10-17 files=6 bytes=53775
ĐÃ XOÁ date=2026-10-18 files=6 bytes=53961
ĐÃ XOÁ date=2026-10-19 files=6 bytes=54448
ĐÃ XOÁ date=2026-10-20 files=6 bytes=54380
ĐÃ XOÁ date=2026-10-21 files=6 bytes=52978
ĐÃ XOÁ date=2026-10-22 files=6 bytes=54469
ĐÃ XOÁ date=2026-10-23 files=6 bytes=53852
ĐÃ XOÁ date=2026-10-24 files=6 bytes=52854
ĐÃ XOÁ date=2026-10-25 files=6 bytes=55401
ĐÃ XOÁ date=2026-10-26 files=6 bytes=54127
ĐÃ XOÁ date=2026-10-27 files=6 bytes=52559
ĐÃ XOÁ date=2026-10-28 files=6 bytes=51813
ĐÃ XOÁ date=2026-10-29 files=6 bytes=52909
ĐÃ XOÁ date=2026-10-30 files=6 bytes=52820
tổng: 23 phân vùng, 138 file, 1231634 byte (giữ 7 ngày tính tới 2026-11-06)
```

Số cho thấy: giảm mẫu giữ 26,7% số điểm (239.990 trong 900.000) và file Parquet nhỏ đi 2,8 lần (4,07 MB so với 11,39 MB không giảm mẫu); so với 53,2 MB protobuf của chính các sự kiện đó là nhỏ hơn 13 lần. Byte mỗi dòng **tăng** (12,7 lên 17,0) vì điểm giữ lại ít lặp hơn nên nén kém hơn; đừng đọc «byte mỗi dòng» thành chỉ số tiết kiệm. Sai số khi vẽ lại quỹ đạo từ điểm đã giữ: p50 5,0 m, p95 15,3 m, p99 20,9 m, lớn nhất 44,7 m (đo trên quỹ đạo mô phỏng có nhiễu accuracy 3 đến 25 m, nên một phần sai số là nhiễu chứ không phải do bỏ điểm). Lý do giữ điểm: `turn` chiếm 35% số điểm giữ (83.459) vì đường đi mô phỏng đổi hướng ngẫu nhiên mỗi bước; đường thật thẳng hơn nên tỷ lệ giữ sẽ khác, **số này chỉ đúng cho simulator**. Mức `coarse` nhỏ hơn mức `raw` khoảng 38% byte trong lần chạy 300 tài xế (80.655 so với 130.226) và `driver_ref` là NULL. Retention: giữ 30 ngày xoá 15 phân vùng ngày (90 file, 801.438 byte); sau đó `--days 7` xoá thêm 23 phân vùng. Quyết định xoá dựa trên ngày ghi trong tên thư mục, không dựa mtime.

#### Luồng thật qua Kafka

`history-writer run` đọc `location.updates.v1` bằng consumer group riêng (`--from end`, group đặt tên theo thời điểm chạy), commit offset **sau** khi file đã đổi tên xong (giao hàng ít nhất một lần). Loadgen đẩy 25.000 tài xế ở 6.250 điểm/s trong 60 giây qua ingest và Kafka thật (`p02-go-raw/loc20_live.txt`):

```bash
examples/toi-noi-location/go/bin/history-writer run --out examples/toi-noi-location/go/out/history-live --id-level pseudonym --flush-every 10s --group history-lab-1793972459 --from end
examples/toi-noi-location/go/bin/loadgen --url 'ws://127.0.0.1:8081/ws,ws://[::1]:8083/ws' --drivers 25000 --rate 6250 --batch 3 --duration 60s --report-every 20s
```

```text
# loadgen drivers=25000 seed=1 batch=3 urls=2 viewers=0 slow_viewers=0
# kết nối: 25000/25000 thành công, 0 lỗi, mất 3.5s
# khởi động 14s ở 6250 điểm/s (không tính)
  t= 20s target=6250 acked_pts_s=6232 win_p50=0.6ms win_p99=40.9ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=137107 heap_mb=596
  t= 40s target=6250 acked_pts_s=6272 win_p50=0.5ms win_p99=24.3ms errs=0 q_depth=9 q_coalesced=0 q_dropped=0 kafka_delivered=262641 heap_mb=521
  t= 60s target=6250 acked_pts_s=6249 win_p50=0.8ms win_p99=25.6ms errs=0 q_depth=0 q_coalesced=0 q_dropped=0 kafka_delivered=387642 heap_mb=721
STEP	6250	6252	2084	0.57	10.00	23.15	141.45	0	17	721.3	1008.2	25010	561	0	0	21	732.4
ingest kafka delivered 393882
$ examples/toi-noi-location/go/bin/history-writer run (log)
flush: in=0 kept=0 rows=0 bytes=0
flush: in=0 kept=0 rows=0 bytes=0
flush: in=52560 kept=27573 rows=27573 bytes=691884
flush: in=115668 kept=52912 rows=52912 bytes=1365996
flush: in=178797 kept=74475 rows=74475 bytes=1963461
flush: in=241722 kept=96067 rows=96067 bytes=2543065
flush: in=304704 kept=115713 rows=115713 bytes=3078530
flush: in=367881 kept=135065 rows=135065 bytes=3603741
flush: in=393882 kept=142693 rows=142693 bytes=3824371

$ examples/toi-noi-location/go/bin/history-writer inspect --out examples/toi-noi-location/go/out/history-live
schema (file đầu tiên):
  event_time     NOT NULL TIMESTAMP(isAdjustedToUTC=true,unit=MILLIS)
  received_time  NOT NULL TIMESTAMP(isAdjustedToUTC=true,unit=MILLIS)
  driver_ref     NULLABLE STRING
  id_level       NOT NULL INT(32,true)
  trip_id        NULLABLE STRING
  lat_e7         NOT NULL INT(32,true)
  lng_e7         NOT NULL INT(32,true)
  accuracy_dm    NULLABLE INT(32,true)
  speed_dmps     NULLABLE INT(32,true)
  heading_ddeg   NULLABLE INT(32,true)
  seq            NOT NULL INT(64,true)
  flags          NOT NULL INT(32,true)
  sample_reason  NOT NULL STRING
metadata: toinoi.schema_version=1 toinoi.id_level=1
driver_ref của hai dòng đầu: 0003183859138434, 0003183859138434 (id_level=1)
date=2026-11-06/city=CT      rows=11368    bytes=315876
date=2026-11-06/city=DN      rows=14153    bytes=387156
date=2026-11-06/city=HCM     rows=56750    bytes=1492964
date=2026-11-06/city=HN      rows=42891    bytes=1131908
date=2026-11-06/city=HP      rows=10111    bytes=283178
date=2026-11-06/city=NT      rows=7420     bytes=213289
tổng: 142693 dòng trong 6 phân vùng

$ du -sh examples/toi-noi-location/go/out/history-live; find examples/toi-noi-location/go/out/history-live -name '*.parquet' | wc -l
3.7M	/Users/quandh/Other/github/antblog/.claude/worktrees/location-series/examples/toi-noi-location/go/out/history-live
      42
```

Số cho thấy: writer nhận đúng 393.882 sự kiện bằng số ingest đã ghi vào Kafka, giữ 142.693 (36,2%) và ghi 42 file, 3,7 MB. Tỷ lệ giữ cao hơn lần demo vì 60 giây chưa đủ để tài xế đi đủ xa: 25.000 điểm «đầu tiên» (6,3%) đều được giữ.

#### Python đọc lại được (schema v1)

```bash
uv run --no-project --with polars --with pyarrow python -I examples/toi-noi-location/go/scripts/readback_history.py examples/toi-noi-location/go/out/history-live
```

```text
$ uv run --no-project --with polars --with pyarrow python -I examples/toi-noi-location/go/scripts/readback_history.py examples/toi-noi-location/go/out/history-live
polars 2.1.0 rows 142693
Schema({'event_time': Datetime(time_unit='ms', time_zone='UTC'), 'received_time': Datetime(time_unit='ms', time_zone='UTC'), 'driver_ref': String, 'id_level': Int32, 'trip_id': String, 'lat_e7': Int32, 'lng_e7': Int32, 'accuracy_dm': Int32, 'speed_dmps': Int32, 'heading_ddeg': Int32, 'seq': Int64, 'flags': Int32, 'sample_reason': String, 'date': Date, 'city': String})
shape: (6, 2)
┌──────┬───────┐
│ city ┆ rows  │
│ ---  ┆ ---   │
│ str  ┆ u32   │
╞══════╪═══════╡
│ CT   ┆ 11368 │
│ DN   ┆ 14153 │
│ HCM  ┆ 56750 │
│ HN   ┆ 42891 │
│ HP   ┆ 10111 │
│ NT   ┆ 7420  │
└──────┴───────┘
speed_dmps null: 0 | speed_dmps == 0: 7869
sample_reason: [('first', 25000), ('flagged', 32), ('heartbeat', 36), ('move', 93037), ('turn', 24588)]
pyarrow rows 142693 | event_time type: timestamp[ms, tz=UTC]
```

Polars 2.1.0 và PyArrow đều đọc được thư mục kiểu Hive: `event_time` là `Datetime[ms, UTC]`, các cột nullable giữ NULL, và `speed_dmps` có 7.869 giá trị 0 tách biệt hẳn với NULL (không có NULL nào trong lần chạy này vì mọi điểm đều có tốc độ). `date` và `city` đến từ đường dẫn, không nằm trong file. Mô tả đầy đủ schema và mức định danh: `examples/toi-noi-location/proto/location/v1/gps-history-parquet.md`.

#### Mức định danh

| Mức | `driver_ref` ví dụ (hai dòng đầu, không in toạ độ) | Toạ độ | Ghi chú đo được |
|---|---|---|---|
| 0 raw | `DR-00009` | nguyên | 130.226 byte cho 6.459 dòng (300 tài xế × 5 phút) |
| 1 pseudonym | `015e3d6bec00c070` (HMAC theo tháng) | nguyên | cùng quỹ đạo, tên không đọc ra mã tài xế, không nối được qua tháng (test) |
| 2 coarse | NULL | làm tròn ~111 m | 80.655 byte cho cùng 6.459 dòng; bỏ cả `trip_id` |

Mức pseudonym **vẫn là dữ liệu cá nhân**: quỹ đạo đủ để nhận ra người, nên bài cần nói rõ HMAC chỉ che mã tài xế. Nghĩa vụ pháp lý ở Việt Nam (thời hạn lưu, căn cứ xử lý dữ liệu vị trí) **chưa kiểm**; các con số retention trong lab (30 và 7 ngày) chỉ là tham số thí nghiệm, không phải khuyến nghị pháp lý. Cần kiểm lại văn bản hiện hành khi viết bài.

Test (`internal/history`, `-race`): giảm mẫu theo từng lý do; phân vùng ngày theo giờ Việt Nam (18:00 UTC thuộc ngày hôm sau); đọc lại schema, NULL khác 0, metadata `toinoi.schema_version`; pseudonym ổn định trong tháng và đổi giữa các tháng; retention theo tên thư mục, không đụng thư mục lạ.


## Test, vet, gofmt và verify

```bash
go -C examples/toi-noi-location/go vet ./...
go -C examples/toi-noi-location/go test -race -count=1 ./...
bash examples/toi-noi-location/verify-p02-go.sh
```

(`p02-go-raw/tests_all.txt`)

```text
$ uptime
23:29  up 35 days, 15:12, 1 user, load averages: 137.72 99.93 85.64

$ test -z "$(gofmt -l examples/toi-noi-location/go)" && echo gofmt-sach
gofmt-sach

$ go -C examples/toi-noi-location/go vet ./...
(exit=0)

$ go -C examples/toi-noi-location/go test -race -count=1 ./...
?   	toinoi/location/cmd/history-writer	[no test files]
?   	toinoi/location/cmd/kafka_lab	[no test files]
?   	toinoi/location/cmd/loadgen	[no test files]
ok  	toinoi/location/cmd/loc11_nhap_mon	1.946s
?   	toinoi/location/cmd/loc13_backpressure	[no test files]
?   	toinoi/location/cmd/location-ingest	[no test files]
?   	toinoi/location/cmd/tracking-hub	[no test files]
?   	toinoi/location/gen/location/v1	[no test files]
ok  	toinoi/location/internal/backpressure	2.213s
ok  	toinoi/location/internal/geo	2.797s
ok  	toinoi/location/internal/history	3.353s
ok  	toinoi/location/internal/hub	2.762s
ok  	toinoi/location/internal/ingest	3.823s
ok  	toinoi/location/internal/kafkax	4.286s
ok  	toinoi/location/internal/labauth	4.706s
ok  	toinoi/location/internal/sim	5.178s
?   	toinoi/location/internal/stat	[no test files]
ok  	toinoi/location/internal/validate	5.625s
(exit=0)
```

Test cần Kafka nằm sau biến môi trường `TOI_NOI_KAFKA` (`1` hoặc `host:port`); không đặt thì bỏ qua, đặt mà không kết nối được thì **thất bại** có thông báo rõ. Lần chạy đầu của test này **FAIL sau 30 giây** (`kafkax_test.go:63: không đọc lại được bản ghi vừa ghi`), và lỗi đó đã được điều tra (`p02-go-raw/tests_kafka.txt`).

Nguyên nhân là **lỗi của test**, không phải của `Sink` và không phải cuộc đua: test ghi một bản ghi (đã `Flush` xong, `delivered = 1`) rồi mở consumer với `ConsumeResetOffset(AtStart())` và đọc **từ đầu topic** cho tới khi gặp key của mình. Topic `location.updates.v1` của lab tích luỹ hàng triệu bản ghi qua các lần chạy tải (LOC-12, 13, 19, 20), nên thời gian của test tỉ lệ với kích thước topic và hết hạn 30 giây của `context` trước khi đọc tới bản ghi nằm ở cuối. Tái hiện: dựng lại Kafka (topic trống), ghi bản ghi giả bằng `kafka_lab fill`, chạy test chưa sửa:

| Số bản ghi trong topic | Test chưa sửa |
|---|---|
| 5.394.002 (+ vài bản ghi của test) | qua, 8,23 giây |
| 20.394.002 (+ vài bản ghi của test) | **FAIL sau 30,26 giây**, đúng thông báo cũ |

Cách sửa (`internal/kafkax/kafkax_test.go`): trước khi ghi, lấy offset cuối của từng partition (`kadm.ListEndOffsets`), rồi consumer chỉ đọc từ các offset đó (`kgo.ConsumePartitions`), nên thời gian test không còn phụ thuộc kích thước topic. Sau khi sửa, test qua trong 0,41 giây trên topic đã có 42.050.569 bản ghi, và ca «đã yêu cầu mà Kafka không có» vẫn FAIL với thông báo rõ (nay ở `kafkax_test.go:26`).

```bash
go -C examples/toi-noi-location/go run ./cmd/kafka_lab fill 5000000       # rồi 15000000 để tái hiện FAIL
TOI_NOI_KAFKA=1 go -C examples/toi-noi-location/go test -race -count=1 -v ./internal/kafkax
```

Output sau khi sửa, trên topic 42.050.569 bản ghi (`p02-go-raw/tests_kafka.txt`, khối [4]):

```text
$ uptime
23:22  up 35 days, 15:06, 1 user, load averages: 62.14 65.37 72.52

$ TOI_NOI_KAFKA=1 go -C examples/toi-noi-location/go test -race -count=1 -v ./internal/kafkax
=== RUN   TestSinkGhiDungKeyVaNoiDung
--- PASS: TestSinkGhiDungKeyVaNoiDung (0.41s)
PASS
ok  	toinoi/location/internal/kafkax	1.979s
(exit=0)

$ TOI_NOI_KAFKA=127.0.0.1:1 go -C examples/toi-noi-location/go test -count=1 ./internal/kafkax   # đã yêu cầu mà Kafka không có: phải FAIL
--- FAIL: TestSinkGhiDungKeyVaNoiDung (0.00s)
    kafkax_test.go:26: TOI_NOI_KAFKA đã đặt nhưng Kafka không dùng được: không kết nối được Kafka [127.0.0.1:1]: unable to dial: dial tcp 127.0.0.1:1: connect: connection refused
FAIL
FAIL	toinoi/location/internal/kafkax	0.525s
FAIL
(exit=1)
```

`verify-p02-go.sh` (`p02-go-raw/verify.txt`, kết thúc bằng `PASS`) không bật Docker. Nó chạy gofmt, vet, `go test -race`, các lab LOC-11 (kiểm `-race` bắt được lỗi), LOC-13 (rút gọn), LOC-20 (rút gọn), và ingest + loadgen nhỏ có tiêm lỗi. Không nằm trong script: test Kafka (cần `TOI_NOI_KAFKA` và Docker), mọi phép đo 25.000 kết nối, sweep, `docker pause`, đo qua hub, luồng `history-writer run`; `VERIFY_PROTO=1` để kiểm sinh lại protobuf.

## Chưa làm, chưa đo

- gRPC stream không được dựng hay đo; phần so sánh WebSocket/gRPC là lập luận.
- Kịch bản mạng chập chờn thật (mất kết nối, nối lại, gửi lại lô chưa ack) chưa chạy. Chỉ có test đơn vị cho điểm đến sau và trùng.
- Mọi số đo là một lần chạy trên một máy với Kafka một broker trong Docker; chưa lặp, chưa có khoảng tin cậy, và loadgen chiếm CPU cùng máy.
- Nguyên nhân gãy ở 100.000 điểm/s chưa tách trọn vẹn (xem LOC-12, mục E và F).
- `kafka.failed = 20000` ở lần dừng Kafka (LOC-13): đã xác định là `RecordDeliveryTimeout` hết hạn của client Kafka (xem mục điều tra ở LOC-13). Còn chưa giải quyết: vì sao cùng hạn và cùng thời gian dừng mà lúc lỗi lúc không (cuộc đua bên trong franz-go chưa lần theo), và vì sao lần gốc lỗi ở thời gian dừng danh nghĩa 26 giây trong khi các lần lặp 26 giây không lỗi.
- Tuổi điểm đã giao khi Kafka thật bị dừng chưa đo (chỉ có ở thí nghiệm offline).
- Redis (cổng 6391) được bật cùng compose nhưng phần Go không dùng.
- Nghĩa vụ pháp lý Việt Nam về lưu vị trí chưa kiểm; không có số nào trong lab là khuyến nghị pháp lý.
- `route_offset_m` của `TrackingUpdate` để trống trong lab (map matching ở LOC-16/17); nội suy phía app chỉ có hợp đồng bằng chữ, chưa có mã client.
