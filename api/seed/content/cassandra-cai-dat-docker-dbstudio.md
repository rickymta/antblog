---
slug: cassandra-cai-dat-docker-dbstudio
title: "Dựng Cassandra 5.0 bằng Docker và kết nối bằng DBStudio"
summary: "Dựng một node Apache Cassandra 5.0.9 cho lab Tới Nơi bằng Docker Compose, chạy file CQL bằng cqlsh trong container, kết nối DBStudio từ máy host, đọc phiên bản đầu tiên, ghi phiếu lab và dọn môi trường. Bài kể cả lỗi Connect timeout gặp thật khi node quảng bá IP nội bộ của Docker."
category: cassandra
level: beginner
tags: [cassandra, cs-002, cassandra-5, docker, dbstudio, cqlsh, toi-noi]
language: vi
---

Lần đầu dựng lab Cassandra cho series này, mọi thứ trông đều ổn: container báo `healthy`, `cqlsh` trong container chạy được `SELECT`, cổng CQL đã mở ra máy host. Thế nhưng DBStudio trên chính máy đó bấm "Kiểm tra kết nối" thì đứng một lúc rồi báo:

```text
No connections in the pool … Connect timeout elapsed
```

Cổng mở, node sống, vậy driver kết nối đi đâu? Câu trả lời nằm ở cách driver Cassandra làm việc: nó **hỏi node danh sách các node trong cụm, rồi kết nối tới địa chỉ mà node tự khai**. Node trong Docker tự khai IP nội bộ của mạng Docker (`172.x`), một địa chỉ mà máy host không với tới. Bài này dựng lab đã sửa lỗi đó, giải thích từng dòng cấu hình, rồi kết nối bằng DBStudio và chạy câu truy vấn đầu tiên.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-002 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/cassandra-la-gi-va-vai-tro-trong-toi-noi">CS-001</a>. Cần Docker Desktop và Node.js 22. Lab: <code>examples/toi-noi-cassandra</code>. Sau bài này, bạn có một node Cassandra 5.0.9 riêng, chạy được lab P01 với kết quả <code>20 PASS, 0 FAIL</code>, dùng được <code>cqlsh</code> trong container, kết nối được bằng DBStudio và biết cách dọn sạch.</p>
</div>

## 1. Lab cần đạt bốn điều

| Yêu cầu | Lab làm thế nào |
|---|---|
| **Tái lập** | Ảnh `cassandra:5.0.9` ghim theo digest; keyspace `toinoi` được xoá và tạo lại bằng `00-reset.cql`; dữ liệu cố định; `p01/OUTPUT.txt` là kết quả mẫu để so |
| **Cô lập** | Chạy trong container, cổng CQL riêng `19042`, chỉ mở trên `127.0.0.1` |
| **Ghi được phiên bản** | File đầu tiên in cluster, phiên bản, datacenter, rack, phiên bản CQL |
| **Dọn được** | Một lệnh xoá container và dữ liệu |

Không cần cài Cassandra, Java hay Python trên máy: `cqlsh` và `nodetool` có sẵn **bên trong** ảnh Docker.

## 2. Chuẩn bị

- **Docker Desktop** đang chạy, engine Linux (trên Windows là WSL 2). Mỗi node bị giới hạn `mem_limit: 1600m`. Phần 1 chỉ cần một node; Phần 3 chạy ba node nên Docker cần ít nhất 6 GB RAM.
- **Node.js 22** để chạy runner `run.mjs` và test. Runner không có dependency npm, không cần `npm install`.

```sh
docker version --format '{{.Server.Version}}'
node --version
```

## 3. Đọc `docker-compose.yml`

File `examples/toi-noi-cassandra/docker-compose.yml` mô tả cả ba node, nhưng hai node sau nằm trong profile `cluster` nên mặc định chỉ `node1` chạy. Phần chung của mọi node đặt trong khối `x-node`:

```yaml
x-node: &node
  image: cassandra:5.0.9@sha256:acd754f07c73f3a166bb477354b56ce9a05690c3e5e2aaacc1aab2afddba6436
  mem_limit: 1600m
  environment: &env
    CASSANDRA_CLUSTER_NAME: "ToiNoi Lab"
    CASSANDRA_DC: "dc1"
    CASSANDRA_RACK: "rack1"
    CASSANDRA_ENDPOINT_SNITCH: "GossipingPropertyFileSnitch"
    CASSANDRA_NUM_TOKENS: "16"
    CASSANDRA_SEEDS: "node1"
    MAX_HEAP_SIZE: "768M"
    HEAP_NEWSIZE: "200M"
    CASSANDRA_BROADCAST_RPC_ADDRESS: "127.0.0.1"
    TZ: "Asia/Ho_Chi_Minh"
  entrypoint: ["sh", "/lab/docker/entry.sh"]
  healthcheck:
    test: ["CMD-SHELL", "cqlsh 127.0.0.1 $$NATIVE_PORT -e 'SELECT release_version FROM system.local' > /dev/null"]
    interval: 10s
    timeout: 15s
    retries: 30
    start_period: 60s
```

| Dòng | Ý nghĩa |
|---|---|
| `image: cassandra:5.0.9@sha256:…` | Tag `5.0.9` có thể bị đẩy lại; **digest** khoá đúng một bản ảnh |
| `mem_limit`, `MAX_HEAP_SIZE`, `HEAP_NEWSIZE` | Cassandra chạy trên JVM. Mặc định ảnh tự tính heap theo RAM máy, có thể quá lớn cho laptop; lab đặt heap 768 MB trong giới hạn 1600 MB của container |
| `CASSANDRA_CLUSTER_NAME` | Tên cluster. Node chỉ nhận node khác vào cụm khi trùng tên |
| `CASSANDRA_DC`, `CASSANDRA_RACK`, `CASSANDRA_ENDPOINT_SNITCH` | Snitch `GossipingPropertyFileSnitch` đọc datacenter và rack của node từ cấu hình rồi loan báo cho cụm qua gossip (CS-025) |
| `CASSANDRA_NUM_TOKENS: "16"` | Số token (vnode) mỗi node nhận trên vòng token (CS-021) |
| `CASSANDRA_SEEDS: "node1"` | Node "hạt giống": node mới liên hệ seed để biết cụm gồm những ai |
| `CASSANDRA_BROADCAST_RPC_ADDRESS: "127.0.0.1"` | Địa chỉ node **khai cho driver**. Đây là dòng sửa lỗi Connect timeout (mục 4) |
| `TZ` | Múi giờ của container: ảnh hưởng cách `cqlsh` hiển thị `timestamp` và cách node hiểu chuỗi thời gian không ghi độ lệch (CS-004) |
| `entrypoint` | Chạy `docker/entry.sh` trước entrypoint gốc của ảnh |
| `healthcheck` | Container chỉ "healthy" khi `cqlsh` chạy được một câu `SELECT`, không phải khi tiến trình Java vừa lên. Cho phép tới 60 giây khởi động cộng 30 lần thử, mỗi lần cách 10 giây |

Phần riêng của `node1`:

```yaml
  node1:
    <<: *node
    hostname: node1
    environment:
      <<: *env
      NATIVE_PORT: "19042"
    ports:
      - "127.0.0.1:19042:19042"
    volumes:
      - node1-data:/var/lib/cassandra
      - ./:/lab:ro
```

`NATIVE_PORT` là cổng CQL của node; cổng trong container **trùng** cổng mở ra máy host (`19042:19042`) và chỉ mở cho `127.0.0.1`. Dữ liệu nằm trong volume `node1-data`; cả thư mục lab được mount **chỉ đọc** vào `/lab` để `cqlsh` đọc file CQL.

### `docker/entry.sh`

```sh
set -eu
yaml=/etc/cassandra/cassandra.yaml
sed -i "s/^native_transport_port:.*/native_transport_port: ${NATIVE_PORT:-9042}/" "$yaml"
exec docker-entrypoint.sh cassandra -f
```

Ảnh Cassandra không có biến môi trường cho cổng CQL, nên script sửa dòng `native_transport_port` trong `cassandra.yaml` thành `NATIVE_PORT`, rồi chuyển quyền cho entrypoint gốc (`docker-entrypoint.sh`), entrypoint này lo các biến `CASSANDRA_*` còn lại.

## 4. Vì sao phải quảng bá `127.0.0.1` với cổng riêng?

Driver Cassandra (trong DBStudio hay trong ứng dụng) không chỉ dùng địa chỉ bạn gõ vào. Sau khi kết nối tới địa chỉ ban đầu (*contact point*), driver đọc bảng `system.local` và `system.peers_v2` để biết cụm có những node nào và **địa chỉ CQL mà mỗi node khai báo**, rồi mở các kết nối tiếp theo tới những địa chỉ đó. Đây là cách driver tự phát hiện node mới và chọn node giữ dữ liệu.

Trong Docker, địa chỉ mặc định node tự khai là IP của container trên mạng nội bộ Docker. Đọc trực tiếp trên lab (câu chỉ đọc, chạy bằng `cqlsh` trong container):

```sql
SELECT partitioner, broadcast_address, rpc_address, rpc_port FROM system.local;
```

```text
 partitioner                                 | broadcast_address | rpc_address | rpc_port
---------------------------------------------+-------------------+-------------+----------
 org.apache.cassandra.dht.Murmur3Partitioner |        172.20.0.2 |   127.0.0.1 |    19042
```

`broadcast_address` (địa chỉ node dùng để nói chuyện với node khác) vẫn là `172.20.0.2`, vì các node trong mạng Docker cần nó. Còn địa chỉ khai cho client (`rpc_address`, `rpc_port`) là `127.0.0.1:19042` nhờ `CASSANDRA_BROADCAST_RPC_ADDRESS` và `entry.sh`. Trước khi sửa, driver nhận `172.20.0.2` và cổng 9042, không kết nối được và báo `Connect timeout`.

Vì sao mỗi node một cổng? Ở Phần 3, ba node cùng khai `127.0.0.1`; nếu cùng cổng 9042 thì driver không phân biệt được node nào. Mỗi node một cổng (`19042`, `19043`, `19044`), và cổng trong container phải bằng cổng ngoài máy host, vì driver kết nối đúng cổng node khai. Cách này chỉ dành cho lab trên một máy; cụm thật dùng địa chỉ mạng thật của từng máy.

## 5. Khởi động và dùng `cqlsh` trong container

```sh
cd examples/toi-noi-cassandra
npm run db:up            # = docker compose up -d --wait
docker compose ps
```

```text
# đã lược cột IMAGE, COMMAND, CREATED
NAME                           SERVICE   STATUS                 PORTS
toinoi-cassandra-lab-node1-1   node1     Up 3 hours (healthy)   127.0.0.1:19042->19042/tcp
```

Lần đầu Docker tải ảnh (khoảng 360 MB) và Cassandra khởi tạo thư mục dữ liệu, nên chờ khoảng một phút là bình thường.

**cqlsh** là trình dòng lệnh CQL đi kèm Cassandra. Mở một phiên tương tác:

```sh
docker compose exec node1 cqlsh 127.0.0.1 19042
```

Phải ghi rõ `127.0.0.1 19042`. Gõ `cqlsh` không tham số thì nó thử cổng mặc định 9042, và trên lab cổng đó không còn ai nghe:

```text
Connection error: ('Unable to connect to any servers', {'127.0.0.1:9042': ConnectionRefusedError(111, "Tried connecting to [('127.0.0.1', 9042)]. Last error: Connection refused")})
```

Chạy một câu rồi thoát ngay bằng `-e`:

```sh
docker compose exec -T node1 cqlsh 127.0.0.1 19042 -e "DESCRIBE KEYSPACES;"
```

```text
system       system_distributed  system_traces  system_virtual_schema
system_auth  system_schema       system_views   toinoi
```

Các keyspace bắt đầu bằng `system` là của Cassandra: `system_schema` chứa định nghĩa bảng (CS-003), `system_auth` chứa role (CS-041), `system_views` và `system_virtual_schema` là bảng ảo để giám sát (CS-043). `toinoi` là keyspace của lab, chỉ có sau khi chạy `npm run p01`.

## 6. Chạy lab bằng runner

```sh
npm run p01
```

Runner `run.mjs` chạy từng file `p01/NN-*.cql` theo thứ tự tên bằng `cqlsh -f` **bên trong** container (mỗi file một phiên cqlsh), và chạy file `.sh` bằng `sh` trong container. Kết quả cuối phải là:

```text
-- [lab] File đã chạy: 10
-- [lab] PASS: 20
-- [lab] FAIL: 0
-- [lab] KẾT QUẢ: ĐẠT
```

Trong file CQL có hai loại chú thích đặc biệt:

| Chú thích | Runner làm gì |
|---|---|
| `-- ## Tiêu đề` | In tiêu đề vào kết quả để biết đoạn kết quả thuộc câu nào |
| `-- EXPECT: <chuỗi>` (ngay trước một câu lệnh) | Câu đó **phải** lỗi và thông điệp chứa `<chuỗi>` thì in `PASS`; chạy được hoặc lỗi khác thì `FAIL` |

Mọi lỗi không khai báo bằng `EXPECT` đều tính là `FAIL`.

Cách chèn tiêu đề hơi khác thường vì `cqlsh -f` không in lại câu lệnh nó chạy. Runner thay mỗi dòng `-- ## …` bằng một câu cố ý sai cú pháp `@@MARK;`. cqlsh báo lỗi cú pháp kèm số dòng; runner tra số dòng đó trong bảng đã ghi nhớ và thay thông báo lỗi bằng tiêu đề. Lỗi của câu lệnh thật cũng được nhận ra theo số dòng, nên runner biết lỗi đó có nằm trong danh sách `EXPECT` hay không. Ví dụ trong `OUTPUT.txt`:

```text
-- ## Kiểu của một cột không đổi được sang kiểu không tương thích
PASS lỗi mong đợi (dòng 68): Altering column types is no longer supported
```

`npm test` chạy lại toàn bộ và so từng dòng với `p01/OUTPUT.txt`, bỏ qua những gì phụ thuộc máy và thời điểm chạy (thời gian của `COPY FROM`, ước lượng kích thước của `nodetool`).

## 7. Kết nối bằng DBStudio

**DBStudio** là công cụ quản trị nhiều loại database viết bằng Rust và Tauri, mã nguồn ở [github.com/rickymta/dbstudio](https://github.com/rickymta/dbstudio). Series dùng bản **0.1.7**. Với Cassandra, DBStudio có cây keyspace → bảng, xem DDL, query editor CQL, lưới kết quả sửa được theo khoá, sơ đồ keyspace và tracing.

Mở hộp thoại **Kết nối mới**, dán vào ô **URI kết nối**:

```text
cassandra://127.0.0.1:19042/toinoi
```

rồi bấm **Điền từ URI**.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-002/01-hop-thoai-ket-noi.webp" alt="Hộp thoại Kết nối mới của DBStudio sau khi dán URI cassandra://127.0.0.1:19042/toinoi và bấm Điền từ URI: dòng Đã điền các trường từ URI, Loại cơ sở dữ liệu Cassandra, Tên toinoi-lab, Máy chủ 127.0.0.1, Cổng 19042, ô Keyspace toinoi, Nhóm Không nhóm, Môi trường Phát triển, công tắc Chỉ đọc tắt, phần Xác thực chọn Không cần xác thực." width="1160" height="1532" loading="lazy"></div>
<figcaption><b>Hình 1 · Điền kết nối từ URI.</b> Nhìn ô thứ năm: với Cassandra, DBStudio ghi <b>Keyspace</b> thay cho "Cơ sở dữ liệu", và giá trị <code>toinoi</code> lấy từ phần đường dẫn của URI. Cổng là 19042, đúng cổng node khai báo, không phải 9042.</figcaption>
</figure>

Kiểm lại từng phần:

1. **Loại cơ sở dữ liệu**: Cassandra. **Máy chủ** `127.0.0.1`, **Cổng** `19042`, **Keyspace** `toinoi` (keyspace mặc định cho tab truy vấn, tương đương `USE toinoi`). Đặt **Tên** là `toinoi-lab`.
2. **Môi trường**: **Phát triển**. Với kết nối gắn nhãn **Sản xuất**, DBStudio bắt gõ lại tên trước khi chạy câu lệnh phá huỷ. Công tắc **Chỉ đọc** chặn mọi câu ghi; kết nối lab để tắt.
3. **Xác thực**: **Không cần xác thực**. Lab chưa bật `PasswordAuthenticator`, node nhận mọi kết nối; bật xác thực và tách role là việc của CS-041.
4. **TLS**: **Tắt**. Lab chưa cấu hình chứng chỉ; CS-042 bật TLS cho client và giữa các node.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-002/02-kiem-tra-ket-noi.webp" alt="Phần dưới của hộp thoại Kết nối mới: Môi trường Phát triển, Chỉ đọc tắt, Xác thực Không cần xác thực với giải thích máy chủ không đòi đăng nhập, ô Tên đăng nhập không bắt buộc để trống, Chế độ TLS Tắt với cảnh báo không mã hoá, SSH tunnel tắt, phần Nâng cao. Dưới cùng có dòng chữ xanh Đã kết nối tới Apache Cassandra 5.0.9 sau khi bấm Kiểm tra kết nối." width="1160" height="1532" loading="lazy"></div>
<figcaption><b>Hình 2 · Kiểm tra kết nối.</b> Dòng xanh ở cuối hộp thoại xác nhận driver đã kết nối được và đọc ra phiên bản 5.0.9. Hai lựa chọn "Không cần xác thực" và TLS "Tắt" chỉ hợp với lab trên máy cá nhân: DBStudio ghi rõ ngay dưới ô TLS rằng dữ liệu đi dạng văn bản thuần.</figcaption>
</figure>

Bấm **Kiểm tra kết nối**, thấy dòng "Đã kết nối tới Apache Cassandra 5.0.9" thì bấm **Lưu**. Nếu thay vào đó là `Connect timeout`, xem lại mục 4.

## 8. Câu truy vấn đầu tiên

Mở cây kết nối `toinoi-lab`, bạn thấy keyspace `toinoi`. Mở một tab truy vấn và gõ:

```sql
SELECT cluster_name, release_version, data_center, rack, cql_version
FROM system.local;
```

`Ctrl+Enter` chạy câu đang đặt con trỏ; `Ctrl+Shift+Enter` chạy cả script, mỗi câu một tab kết quả.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-002/03-truy-van-dau-tien.webp" alt="Tab Truy vấn 1 của DBStudio, kết nối toinoi-lab Phát triển, keyspace toinoi. Câu SELECT cluster_name, release_version, data_center, rack, cql_version FROM system.local; cho một dòng: ToiNoi Lab, 5.0.9, dc1, rack1, 3.4.7. Thanh dưới lưới ghi 1 dòng, 2 ms, Chỉ đọc: Thiếu cột khoá key trong SELECT; thanh trạng thái ghi Tự động commit." width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 3 · Truy vấn đầu tiên trên system.local.</b> Kết quả giống hệt dòng đầu của <code>OUTPUT.txt</code>. Hai chi tiết ở đáy màn hình: lưới ghi "Chỉ đọc: Thiếu cột khoá key trong SELECT" vì DBStudio chỉ cho sửa ô khi kết quả có đủ khoá chính; thanh trạng thái ghi "Tự động commit" vì Cassandra không có transaction để mở hay huỷ.</figcaption>
</figure>

`system.local` là bảng hệ thống mô tả **chính node đang trả lời**, nên kết quả luôn có đúng một dòng. Trên cụm nhiều node, câu này trả thông tin của node mà driver chọn làm coordinator.

Một vài khác biệt so với DBStudio khi nối SQL Server hay PostgreSQL: nút "Giải thích kế hoạch" bị ẩn vì Cassandra không có execution plan (CS-028 dùng tracing thay thế); chuột phải trên bảng có **Mở dữ liệu**, **Xem DDL**, **Mở trình soạn thảo**, **Xuất…**, **Tạo script**; chuột phải trên keyspace có **Mở trình soạn thảo**, **Xem sơ đồ**, **Xuất…**, **Sao lưu…**.

### Cùng một thời điểm, hai cách hiển thị

`cqlsh` trong container hiển thị `timestamp` theo múi giờ của container, còn lưới DBStudio hiển thị theo giờ UTC dạng ISO. Chuyến 1 của lab, đọc bằng cqlsh:

```text
 ms            | requested_at
---------------+---------------------------------
 1793580120000 | 2026-11-02 07:42:00.000000+0700
```

Trong DBStudio, cùng ô đó hiện `2026-11-02T00:42:00.000Z`. Hai chuỗi chỉ cùng một thời điểm: Cassandra lưu `timestamp` là số mili giây tính từ 1970 theo UTC (cột `ms`), cách hiển thị là việc của công cụ. Khi so kết quả giữa hai công cụ, đổi về cùng múi giờ trước. CS-004 đi sâu vào chuyện này.

## 9. Ghi phiếu lab

| Mục | Giá trị trên máy viết bài |
|---|---|
| Cassandra | 5.0.9, cluster `ToiNoi Lab`, `dc1`/`rack1`, CQL 3.4.7, Murmur3Partitioner, `num_tokens: 16` |
| Ảnh | `cassandra:5.0.9@sha256:acd754f0…` |
| Docker | Docker Desktop, engine 29.9, Windows 11, WSL 2 |
| Bộ nhớ | `mem_limit: 1600m`, heap 768 MB |
| Múi giờ container | `Asia/Ho_Chi_Minh` (từ `TZ`) |
| Công cụ GUI | DBStudio 0.1.7 |
| Kết quả | `npm run p01`: 10 file, 20 PASS, 0 FAIL |

Khi kết quả của bạn khác `OUTPUT.txt`, so phiếu lab trước tiên: khác bản Cassandra hay khác múi giờ container giải thích phần lớn chênh lệch.

## 10. Dữ liệu giữa các lần chạy và dọn môi trường

| Lệnh | Container | Dữ liệu trong volume |
|---|---|---|
| `docker compose stop` / `start` | Dừng / chạy lại | Giữ nguyên |
| `docker compose down` | Xoá | **Giữ** (volume `node1-data` còn) |
| `npm run db:down` (= `docker compose --profile cluster down -v`) | Xoá cả ba node nếu có | **Xoá** |

Mất dữ liệu lab không sao: `npm run p01` dựng lại từ đầu. Ảnh Docker vẫn còn sau khi dọn; xoá hẳn bằng `docker image rm cassandra:5.0.9` nếu không học tiếp.

## 11. Lỗi thường gặp

- **`docker compose up --wait` chờ rất lâu lần đầu.** Tải ảnh, khởi động JVM và khởi tạo keyspace hệ thống mất thời gian; healthcheck cho phép 60 giây khởi động cộng tới 30 lần thử. Theo dõi bằng `docker compose logs -f node1` và chờ dòng `Starting listening for CQL clients on /0.0.0.0:19042` rồi `Startup complete`.
- **`Bind for 127.0.0.1:19042 failed: port is already allocated`.** Một tiến trình khác giữ cổng. Đừng chỉ đổi số bên trái trong `ports`: driver kết nối theo cổng node khai báo, nên phải đổi cả `NATIVE_PORT` để hai số bằng nhau, rồi đổi URI trong DBStudio.
- **DBStudio hoặc ứng dụng báo `Connect timeout` dù container healthy.** Node đang khai địa chỉ mà máy host không với tới. Kiểm `SELECT rpc_address, rpc_port FROM system.local;` phải ra `127.0.0.1` và cổng đã mở (mục 4).
- **Git Bash báo không tìm thấy `C:/Program Files/Git/lab/...`.** Git Bash tự đổi đường dẫn bắt đầu bằng `/` trong tham số của `docker compose exec`. Đặt `MSYS_NO_PATHCONV=1` trước lệnh; runner đã đặt sẵn biến này.
- **`cqlsh` báo `Connection refused` ở cổng 9042.** Thiếu `127.0.0.1 19042` sau `cqlsh` (mục 5).
- **Container bị dừng với mã 137 hoặc khởi động lại liên tục.** Thiếu RAM: tiến trình Java bị kill khi vượt giới hạn. Tăng bộ nhớ cấp cho Docker Desktop (trên Windows là giới hạn của WSL 2) và đóng bớt ứng dụng nặng.

## 12. Bài tập

**Bài 1.** Chạy `docker compose exec -T node1 cqlsh 127.0.0.1 19042 -e "SELECT broadcast_address, rpc_address, rpc_port FROM system.local;"`. Giải thích vì sao hai cột địa chỉ khác nhau và driver trên máy host dùng cột nào.

<details><summary><b>Lời giải</b></summary><p><code>broadcast_address</code> (<code>172.20.0.2</code>) là địa chỉ node dùng với node khác trong mạng Docker. <code>rpc_address</code> và <code>rpc_port</code> (<code>127.0.0.1</code>, <code>19042</code>) là địa chỉ khai cho client. Driver dùng cặp sau để mở kết nối tới node, nên cặp này phải là địa chỉ máy host với tới được.</p></details>

**Bài 2.** Đồng nghiệp đổi `ports` thành `"127.0.0.1:29042:19042"` vì cổng 19042 bận, và URI DBStudio thành `cassandra://127.0.0.1:29042/toinoi`. Kết nối có chạy không?

<details><summary><b>Lời giải</b></summary><p>Lần kết nối đầu tới 29042 thành công, nhưng node khai <code>127.0.0.1:19042</code>. Driver mở kết nối tiếp theo tới 19042, nơi tiến trình khác đang chiếm hoặc không có ai nghe, nên kết nối lỗi hoặc chập chờn. Cách sửa đúng: đổi cả <code>NATIVE_PORT</code> thành 29042 để cổng trong và ngoài container bằng nhau.</p></details>

**Bài 3.** Thêm một dòng `-- EXPECT: Unable to make int` trước một câu `INSERT` hợp lệ trong file CQL của bạn rồi chạy runner. Kết quả là gì?

<details><summary><b>Lời giải</b></summary><p>Câu <code>INSERT</code> chạy được, nên runner in <code>FAIL câu lệnh dòng … lẽ ra phải lỗi "Unable to make int" nhưng đã chạy được</code> và kết quả cuối là <code>CHƯA ĐẠT</code>. <code>EXPECT</code> khẳng định câu lệnh <em>phải</em> lỗi; đó là cách lab chứng minh Cassandra từ chối một thao tác.</p></details>

**Bài 4.** Trong DBStudio, chuyến 1 hiện `2026-11-02T00:42:00.000Z`; trong cqlsh hiện `2026-11-02 07:42:00.000000+0700`. Một bạn kết luận "DBStudio lưu sai giờ". Bạn trả lời thế nào?

<details><summary><b>Lời giải</b></summary><p>Không ai lưu sai: Cassandra lưu một số mili giây UTC (<code>1793580120000</code>). cqlsh hiển thị theo múi giờ container (+0700), DBStudio hiển thị theo UTC (hậu tố <code>Z</code>). 00:42 UTC chính là 07:42 giờ Việt Nam.</p></details>

## 13. Đọc tiếp

- [Installing Cassandra](https://cassandra.apache.org/doc/5.0/cassandra/installing/installing.html) — gồm cách chạy bằng Docker.
- [cqlsh: the CQL shell](https://cassandra.apache.org/doc/5.0/cassandra/managing/tools/cqlsh.html) — tham số dòng lệnh và lệnh đặc biệt (`DESCRIBE`, `COPY`, `TRACING`).
- [Ảnh Docker chính thức `cassandra`](https://hub.docker.com/_/cassandra) — các biến `CASSANDRA_*`.
- [DBStudio](https://github.com/rickymta/dbstudio) — mã nguồn và bộ cài.
- [PG-002](/articles/postgresql-cai-dat-lab-psql-ket-noi): lab tương ứng của series PostgreSQL.

Bài tiếp theo: [CS-003 · Keyspace, bảng và khoá chính: partition key và clustering key](/articles/cassandra-keyspace-bang-khoa-chinh).
