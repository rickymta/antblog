---
slug: mongodb-cai-dat-docker-dbstudio
title: "Dựng MongoDB 8.0 bằng Docker và kết nối bằng DBStudio"
summary: "Muốn thử dữ liệu chuyến xe của Tới Nơi trên MongoDB mà không đụng server dùng chung? Dựng một replica set MongoDB 8.0 riêng bằng Docker, chạy lab bằng mongosh trong container, kết nối DBStudio bằng URI, chạy lệnh đầu tiên và dọn môi trường sạch sẽ."
category: mongodb
level: beginner
tags: [mongodb, mg-002, mongodb-8, docker, replica-set, mongosh, dbstudio, toi-noi]
language: vi
---

Tuần đầu tiên làm với MongoDB của Tới Nơi, Minh, backend developer mới vào nhóm, cài thử MongoDB lên máy cá nhân bằng bộ cài Windows, giữ nguyên cấu hình mặc định. Mọi thứ chạy ổn cho tới khi anh thử đoạn code "gán tài xế và đổi trạng thái chuyến trong một transaction" của đồng nghiệp: trên máy đồng nghiệp chạy được, trên máy Minh lại báo lỗi. Hoá ra máy Minh chạy một `mongod` đơn lẻ, còn transaction nhiều document chỉ có trên replica set. Hai máy khác bản, khác cách khởi động, và không ai nhớ mình đã chọn gì trong trình cài đặt.

Lab của series tránh chuyện đó bằng một nguyên tắc: **môi trường là code**. Một file `docker-compose.yml` mô tả đúng bản MongoDB, chế độ replica set, múi giờ, cổng; một runner chạy các script theo thứ tự và in kết quả để so. Bài này dựng lab đó, chạy lệnh đầu tiên bằng `mongosh`, kết nối bằng công cụ GUI **DBStudio** và dọn môi trường.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-002 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/mongodb-la-gi-va-vai-tro-trong-toi-noi">MG-001</a>. Cần Docker Desktop và Node.js 22. Lab: <code>examples/toi-noi-mongodb</code>. Sau bài này, bạn có một replica set MongoDB 8.0 riêng, chạy được lab P01 với kết quả <code>115 PASS, 0 FAIL</code>, kết nối được bằng DBStudio qua URI và biết cách dọn sạch.</p>
</div>

## 1. Lab cần đạt bốn điều

| Yêu cầu | Lab làm thế nào |
|---|---|
| **Tái lập**: ai chạy cũng ra cùng kết quả | Ảnh `mongo:8.0` ghim theo digest; `00-reset.js` xoá và dựng lại database `toinoi`; dữ liệu cố định (UUID, thời điểm ghi sẵn); `OUTPUT.txt` là kết quả mẫu để so |
| **Cô lập**: không đụng MongoDB khác trên máy | Chạy trong container, cổng riêng `27018`, chỉ mở trên `127.0.0.1` |
| **Giống môi trường thật ở chỗ quan trọng** | Chạy dưới dạng **replica set** `rs0` ngay từ đầu, để transaction (Phần 4) và change stream (Phần 6) dùng được |
| **Dọn được** | Một lệnh xoá container và dữ liệu; không để lại service chạy nền |

Không cần cài MongoDB hay `mongosh` trên máy: `mongosh` có sẵn **bên trong** ảnh container, runner gọi nó qua `docker compose exec`.

## 2. Chuẩn bị

- **Docker Desktop** đang chạy, engine Linux (trên Windows là WSL 2). Compose giới hạn container ở **1 GB RAM** (`mem_limit: 1g`), đủ cho dữ liệu của lab.
- **Node.js 22** để chạy runner `run.mjs` và test. Runner **không có dependency npm**, không cần `npm install`.
- Cổng **27018** trên máy còn trống. Lab cố ý không dùng 27017 để tránh đụng MongoDB bạn có thể đã cài sẵn.

Kiểm nhanh:

```sh
docker version --format '{{.Server.Version}}'
node --version
```

Máy viết bài: Docker Desktop engine 29.9, Node 22.18.0, Windows 11.

## 3. Đọc file `docker-compose.yml`

Toàn bộ cấu hình nằm trong `examples/toi-noi-mongodb/docker-compose.yml`. Đọc từng dòng một lần, vì mỗi dòng là một quyết định:

```yaml
name: toinoi-mongodb-lab

services:
  db:
    image: mongo:8.0@sha256:d97fea17ccb8d004cd137be8dab54d70d0df11be69c9f422aefe4484c8414ab7
    command: ["mongod", "--replSet", "rs0", "--bind_ip_all"]
    environment:
      TZ: "Asia/Ho_Chi_Minh"
    ports:
      - "127.0.0.1:27018:27017"
    volumes:
      - toinoi-mongo8-data:/data/db
      - ./lib:/lab/lib:ro
      - ./p01:/lab/p01:ro
    healthcheck:
      test: ["CMD", "mongosh", "--quiet", "--eval", "try { rs.status() } catch (e) { rs.initiate({ _id: 'rs0', members: [{ _id: 0, host: 'localhost:27017' }] }) }; if (!db.hello().isWritablePrimary) quit(1)"]
      interval: 3s
      timeout: 10s
      retries: 40
      start_period: 10s
    mem_limit: 1g

volumes:
  toinoi-mongo8-data:
```

| Dòng | Ý nghĩa |
|---|---|
| `image: mongo:8.0@sha256:…` | Tag `8.0` trôi theo bản vá mới; **digest** khoá đúng một bản. Bản đã kiểm chứng: **MongoDB 8.0.38** Community |
| `command: mongod --replSet rs0` | Khởi động `mongod` như một thành viên của replica set tên `rs0`. Thiếu tham số này, server chạy đơn lẻ và không có transaction nhiều document |
| `--bind_ip_all` | `mongod` nghe trên mọi địa chỉ **của container**, để Docker chuyển được kết nối từ máy bạn vào |
| `TZ` | Múi giờ của hệ điều hành trong container. Dữ liệu ngày giờ trong MongoDB luôn lưu theo UTC, nên `TZ` chỉ ảnh hưởng log và lệnh shell, không ảnh hưởng dữ liệu (MG-004) |
| `127.0.0.1:27018:27017` | Cổng 27017 của container mở ra cổng **27018** trên máy bạn, **chỉ cho chính máy bạn** |
| `volumes` | Dữ liệu nằm trong volume có tên `toinoi-mongo8-data`, sống qua lần khởi động lại; `lib/` và `p01/` được mount **chỉ đọc** vào `/lab` để `mongosh` trong container đọc script |
| `healthcheck` | Lần đầu: khởi tạo replica set. Mọi lần: chỉ báo "healthy" khi node đã là primary (mục 4) |
| `mem_limit: 1g` | Giới hạn bộ nhớ; WiredTiger tự chia cache theo giới hạn này |

Lab **không bật xác thực**: không có user, không có mật khẩu, ai kết nối được tới cổng là có toàn quyền. Bù lại, cổng chỉ mở trên `127.0.0.1`. Phần 5 bật xác thực và phân quyền. **Không bao giờ chạy MongoDB không xác thực trên một cổng mở ra mạng**: đó là cách nhiều database bị lấy sạch dữ liệu và để lại lời nhắn đòi tiền chuộc.

## 4. Khởi động và chờ "healthy"

```sh
cd examples/toi-noi-mongodb
npm run db:up            # = docker compose up -d --wait
docker compose ps
```

Kết quả trên máy viết bài:

```text
NAME                      IMAGE                                                                               COMMAND                  SERVICE   CREATED          STATUS                    PORTS
toinoi-mongodb-lab-db-1   mongo:8.0@sha256:d97fea17ccb8d004cd137be8dab54d70d0df11be69c9f422aefe4484c8414ab7   "docker-entrypoint.s…"   db        18 minutes ago   Up 18 minutes (healthy)   127.0.0.1:27018->27017/tcp
```

`--wait` làm lệnh `up` chờ tới khi healthcheck báo **healthy** rồi mới trả về. Với MongoDB, "tiến trình đã chạy" và "nhận ghi được" là hai việc khác nhau, nên healthcheck của lab làm hai bước:

1. **Khởi tạo replica set nếu chưa có.** `rs.status()` ném lỗi khi replica set chưa được khởi tạo; khi đó healthcheck gọi `rs.initiate()` với một thành viên duy nhất `localhost:27017`. Cấu hình này được lưu trong volume, nên các lần khởi động sau `rs.status()` chạy bình thường và bước này bị bỏ qua.
2. **Chờ node lên primary.** Sau `rs.initiate()`, node mất vài giây để tự bầu mình làm primary. `db.hello().isWritablePrimary` còn `false` thì healthcheck thoát mã 1 (chưa healthy) và Docker thử lại sau 3 giây, tối đa 40 lần.

Trong log của container (`docker compose logs db`), dòng cho biết `mongod` đã mở cổng là:

```text
"msg":"Waiting for connections","attr":{"port":27017,"ssl":"off"}
```

`"ssl":"off"` nhắc lại rằng lab không mã hoá kết nối; như xác thực, đó là việc của Phần 5.

## 5. Lệnh đầu tiên bằng `mongosh` trong container

**mongosh** là shell dòng lệnh chính thức của MongoDB: một môi trường JavaScript có sẵn đối tượng `db` và các hàm trợ giúp (`db.trips.find()`, `rs.status()`…). Mở shell tương tác bên trong container:

```sh
docker compose exec db mongosh "mongodb://localhost:27017/toinoi?directConnection=true"
```

Hoặc chạy một lệnh rồi thoát, tiện để chép kết quả:

```sh
docker compose exec -T db mongosh --quiet "mongodb://localhost:27017/toinoi?directConnection=true" --eval 'db.version()'
```

```text
8.0.38
```

Lệnh tiếp theo hỏi server đang đóng vai gì trong replica set. Kết quả đầy đủ của `db.hello()` khá dài; dưới đây là kết quả chép từ lab, các dòng `...` là chỗ đã lược bớt (thời điểm ghi, `$clusterTime`…):

```js
db.hello()
```

```text
{
  topologyVersion: {
    processId: ObjectId('6afb0b294fbbf10ec77b8b6d'),
    counter: Long('6')
  },
  hosts: [ 'localhost:27017' ],
  setName: 'rs0',
  setVersion: 1,
  isWritablePrimary: true,
  secondary: false,
  primary: 'localhost:27017',
  me: 'localhost:27017',
  ...
  maxBsonObjectSize: 16777216,
  maxMessageSizeBytes: 48000000,
  maxWriteBatchSize: 100000,
  ...
  readOnly: false,
  ok: 1,
  ...
}
```

`db.hello()` là lệnh mọi driver gửi ngay khi kết nối để biết node là primary hay secondary, thuộc replica set nào, và giới hạn kích thước ra sao. Trạng thái thành viên xem bằng `rs.status()`:

```js
rs.status().members.map(m => ({ name: m.name, stateStr: m.stateStr, health: m.health }))
```

```text
[ { name: 'localhost:27017', stateStr: 'PRIMARY', health: 1 } ]
```

Một thành viên, đang là `PRIMARY`, khoẻ (`health: 1`).

### Không ghi database trong URI thì đang ở đâu?

Chạy `mongosh --quiet --eval 'db.getName()'` không kèm URI, kết quả trên lab là:

```text
test
```

`mongosh` mặc định đứng ở database `test`. Gõ `db.trips.find()` ở đó sẽ trả về rỗng mà không báo lỗi, vì collection không tồn tại cũng "đọc được" (MG-001). Đổi database bằng `use toinoi`, hoặc ghi sẵn `/toinoi` trong URI như các lệnh ở trên.

## 6. Chạy lab bằng runner

```sh
npm run p01
```

Runner `run.mjs` gọi, cho từng file `p01/NN-*.js` theo thứ tự tên:

```sh
docker compose exec -T db mongosh --quiet --norc "mongodb://localhost:27017/toinoi?directConnection=true" /lab/lib/check.js /lab/p01/00-reset.js
```

| Thành phần | Vì sao |
|---|---|
| `exec -T` | Chạy lệnh trong container đang chạy, không cấp terminal tương tác; output đi thẳng về runner |
| `--quiet --norc` | Bỏ lời chào và không nạp file cấu hình cá nhân `.mongoshrc.js`, để mọi máy in cùng một kết quả |
| `/lab/lib/check.js` | Nạp trước: hàm `table()` in document thành bảng `cột\|cột` (giờ theo giờ VN), `expectValue()` và `expectError()` in dòng `PASS`/`FAIL` |
| `/lab/p01/00-reset.js` | File của lab, đọc từ thư mục mount chỉ đọc |

Mỗi file là **một kết nối riêng**: biến JavaScript khai báo ở file trước không sang file sau. Lỗi *mong đợi* (ví dụ cố ý chèn cước âm để xem validator chặn) được bắt bằng `expectError` và in thành `PASS` khi đúng mã lỗi; lỗi *không* được bắt làm `mongosh` thoát mã khác 0 và runner tính là `FAIL`. Kết quả cuối phải là:

```text
// [lab] File đã chạy: 8
// [lab] PASS: 115
// [lab] FAIL: 0
// [lab] KẾT QUẢ: ĐẠT
```

`npm test` chạy lại toàn bộ, so từng dòng với `p01/OUTPUT.txt`, thử script hoàn tác của MG-009 hai lần và chạy thí nghiệm kiểu BSON của MG-004.

### File đầu tiên in gì?

`00-reset.js` xoá database `toinoi` nếu đã có, rồi in phiên bản, replica set, vai trò và FCV (trích `OUTPUT.txt`):

```text
/* Phiên bản và vai trò của server */
phien_ban|replica_set|vai_tro|database|fcv
---------|-----------|-------|--------|---
8.0.38|rs0|PRIMARY|toinoi|8.0
(1 document)
PASS phiên bản 8.0.x = 8.0.
PASS replica set = rs0
```

Hai dòng `PASS` là chốt chặn: chạy lab trên một server 7.0, hay trên một `mongod` không có replica set, sẽ thấy `FAIL` ngay ở file đầu thay vì những lỗi khó hiểu ở Phần 4. `00-reset.js` gọi `db.dropDatabase()`; chỉ chạy nó trên lab, không bao giờ trỏ URI của runner sang server dùng chung.

## 7. Kết nối bằng DBStudio

**DBStudio** là công cụ quản trị nhiều loại database (PostgreSQL, MySQL, SQL Server, MongoDB, Redis…) của tác giả series, viết bằng Rust và Tauri, mã nguồn ở [github.com/rickymta/dbstudio](https://github.com/rickymta/dbstudio). Series dùng bản **0.1.7** trên Windows 11; mọi ảnh minh hoạ trong series chụp từ bản này, chạy với lab thật. DBStudio là tuỳ chọn: mọi kết quả dạng chữ trong bài đều tái lập được bằng `mongosh` trong container.

### Chuỗi kết nối

Từ máy bạn, lab dùng chuỗi kết nối (*connection string*) sau:

```text
mongodb://127.0.0.1:27018/toinoi?directConnection=true
```

| Phần | Ý nghĩa |
|---|---|
| `mongodb://` | Lược đồ URI chuẩn của MongoDB |
| `127.0.0.1:27018` | Máy và cổng **phía máy bạn** (Docker chuyển vào cổng 27017 của container) |
| `/toinoi` | Database mặc định của kết nối |
| `directConnection=true` | Nói chuyện **trực tiếp** với đúng node này, không đi tìm các thành viên khác của replica set |

Tham số cuối là chỗ dễ vấp nhất. Khi kết nối tới một replica set, driver không chỉ dùng địa chỉ bạn đưa: nó gửi `hello`, đọc danh sách `hosts` mà replica set tự khai báo, rồi kết nối tới **các địa chỉ đó**. Replica set của lab khai báo `localhost:27017` (mục 5), là địa chỉ bên trong container. Từ máy bạn, `localhost:27017` hoặc không có gì, hoặc tệ hơn, là một MongoDB khác bạn đã cài sẵn. `directConnection=true` bảo driver bỏ qua bước tìm kiếm đó và dùng đúng `127.0.0.1:27018`. Phần 6, khi lab có nhiều node, sẽ đặt tên host đúng để driver tìm được cả cụm.

### Tạo kết nối trong DBStudio

Trong DBStudio, bấm **+** ở khung **Kết nối** để mở hộp thoại **Kết nối mới**, dán chuỗi trên vào ô **URI kết nối** rồi bấm **Điền từ URI**. DBStudio tự chọn loại MongoDB, điền máy chủ `127.0.0.1`, cổng `27018`, database `toinoi`, chế độ xác thực **Không cần xác thực**, và giữ `directConnection` trong mục tuỳ chọn driver (phần **Nâng cao**). Ô URI được xoá sau khi điền, đó là bình thường. Đặt tên kết nối dễ nhận (ảnh dưới dùng `Tới Nơi lab · MongoDB`), bấm **Kiểm tra kết nối**: dòng "Đã kết nối tới MongoDB 8.0.38" xác nhận bạn đang nói chuyện với đúng server. Rồi bấm **Lưu**:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-cai-dat-docker-dbstudio/dbstudio-01.webp" alt="Hộp thoại Kết nối mới của DBStudio đã điền từ URI: MongoDB, 127.0.0.1, cổng 27018, database toinoi, kiểm tra kết nối thành công tới MongoDB 8.0.38" width="588" height="740" loading="lazy"></div><figcaption>DBStudio · Kết nối mới sau khi <b>Điền từ URI</b> và <b>Kiểm tra kết nối</b></figcaption></figure>

Mở kết nối, cây bên trái hiện các database của instance, gồm `admin`, `config`, `local` và `toinoi` (MG-001 đã giải thích từng database). Nếu cây không có `toinoi`, bạn chưa chạy `npm run p01`: database chỉ xuất hiện khi đã có collection.

### Lệnh đầu tiên trong query editor

Query editor của DBStudio nhận lệnh theo cú pháp mongosh (một tập con: đủ cho `find`, `aggregate`, `insertOne`, `updateOne`… và các hàm `UUID('…')`, `ISODate('…')`, nhưng không chạy JavaScript tuỳ ý như mongosh). Bấm **+** trên thanh tab để mở tab truy vấn mới; ô chọn kết nối và database ở đầu tab cho biết lệnh sẽ chạy ở đâu (ở đây là `Tới Nơi lab · MongoDB` › `toinoi`). Gõ và chạy:

```js
db.hello()
db.version()
```

Chạy cả script (`Ctrl+Shift+Enter`), mỗi câu lệnh có một tab kết quả riêng. Kết quả phải khớp với những gì `mongosh` trong container trả về ở mục 5: tab **#1 DB.VERSION** có `"version": "8.0.38"`; tab **#2 DB.HELLO**, xem ở chế độ **Tài liệu**, có `"setName": "rs0"` và `"isWritablePrimary": true`. DBStudio hiển thị kết quả dạng **Extended JSON**: ObjectId thành `{"$oid": …}`, số nguyên 64 bit thành `{"$numberLong": …}`, ngày giờ thành `{"$date": "…Z"}` theo UTC:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-cai-dat-docker-dbstudio/dbstudio-02.webp" alt="Query editor DBStudio chạy db.version() và db.hello(), tab DB.HELLO ở chế độ Tài liệu hiện setName rs0 và isWritablePrimary true" width="1086" height="860" loading="lazy"></div><figcaption>DBStudio · <code>db.version()</code> và <code>db.hello()</code>, kết quả dạng Extended JSON</figcaption></figure>

Cùng một server, hai client khác nhau, cùng một câu trả lời. Đó là cách kiểm tra rằng bạn đang nói chuyện với đúng lab chứ không phải một MongoDB nào khác trên máy. Từ MG-003, bài nào có ảnh DBStudio thì cũng có khối lệnh và kết quả dạng chữ để bạn chép lại được.

## 8. Ghi phiếu lab

Mỗi lần chạy lab cho một bài viết hay một báo cáo, ghi lại:

| Mục | Giá trị trên máy viết bài | Lấy bằng |
|---|---|---|
| MongoDB | 8.0.38 Community, FCV 8.0 | `db.version()`, dòng đầu `OUTPUT.txt` |
| Ảnh | `mongo:8.0@sha256:d97fea17…` | `docker compose ps` |
| Topology | Replica set `rs0`, một node, `PRIMARY` | `rs.status()` |
| Xác thực / TLS | Tắt / tắt | `"ssl":"off"` trong log; không có user |
| Docker | Docker Desktop, engine 29.9, Windows 11, WSL 2 | `docker version` |
| Node.js | 22.18.0 | `node --version` |
| Kết quả | `npm run p01`: 8 file, 115 PASS, 0 FAIL | dòng cuối của runner |

Khi kết quả của bạn khác `OUTPUT.txt`, so phiếu lab trước tiên: khác bản MongoDB, thiếu replica set hay chạy nhầm database giải thích phần lớn chênh lệch.

## 9. Dữ liệu giữa các lần chạy và dọn môi trường

| Lệnh | Container | Dữ liệu và cấu hình replica set trong volume |
|---|---|---|
| `docker compose stop` / `start` | Dừng / chạy lại | Giữ nguyên |
| `docker compose down` | Xoá | **Giữ** (volume `toinoi-mongo8-data` còn) |
| `npm run db:down` (= `docker compose down -v`) | Xoá | **Xoá**; lần `db:up` sau, healthcheck khởi tạo lại replica set |

Với lab, mất dữ liệu không sao: `npm run p01` dựng lại trong vài giây. Ảnh Docker vẫn nằm trên máy sau khi dọn (`docker image ls` trên máy viết bài báo 1,29 GB); xoá hẳn bằng `docker image rm mongo:8.0` nếu không học tiếp. Nhớ xoá luôn kết nối `Tới Nơi lab · MongoDB` trong DBStudio nếu sau này cổng 27018 được dùng cho việc khác.

## 10. Lỗi thường gặp

- **Quên `directConnection=true` khi kết nối từ máy.** Driver đi tìm `localhost:27017` theo danh sách của replica set: kết nối treo rồi hết thời gian chờ, hoặc nối nhầm sang MongoDB khác đang chạy ở 27017 trên máy bạn. Trường hợp sau nguy hiểm hơn vì không có lỗi nào báo.
- **`Bind for 127.0.0.1:27018 failed: port is already allocated`.** Một tiến trình khác giữ cổng; đổi số bên trái trong `ports` và trong chuỗi kết nối.
- **`docker compose up --wait` chờ mãi rồi báo unhealthy.** Xem `docker compose logs db`. Nguyên nhân hay gặp: volume cũ được tạo với cấu hình replica set khác. Trên lab, `npm run db:down` rồi `npm run db:up` là cách nhanh nhất.
- **`db.trips.find()` trả về rỗng.** Đang ở database `test` (`db.getName()`), hoặc chưa chạy `npm run p01`.
- **Chạy lệnh trong Git Bash báo không tìm thấy file `C:/Program Files/Git/lab/...`.** Git Bash tự đổi đường dẫn bắt đầu bằng `/`. Đặt `MSYS_NO_PATHCONV=1` trước lệnh; runner đã làm sẵn việc này.
- **Chạy `00-reset.js` khi DBStudio đang mở `toinoi`.** Không lỗi, nhưng kết quả truy vấn và cây đối tượng đang mở vẫn hiển thị trạng thái cũ cho tới khi bạn chạy lại lệnh hoặc bấm **Làm mới** trên cây. Đừng tin màn hình trước khi làm mới.

## 11. Bài tập

**Bài 1.** Chạy `npm run p01` hai lần liên tiếp. Dòng nào chỉ xuất hiện ở lần thứ hai, và vì sao test bỏ qua đúng dòng đó khi so với `OUTPUT.txt`?

<details><summary><b>Lời giải</b></summary><p>Dòng <code>Đã xoá database toinoi cũ</code>: lần đầu database chưa có nên <code>00-reset.js</code> không xoá gì. Dòng này phụ thuộc trạng thái trước khi chạy chứ không phụ thuộc kết quả của lab, nên test loại nó ra trước khi so.</p></details>

**Bài 2.** Trong `mongosh` bên trong container, chạy `use toinoi` rồi `db.trips.countDocuments()`. Sau đó chạy `use toinoii` (thừa chữ `i`) rồi cùng lệnh đó. Giải thích hai kết quả, và kiểm xem `toinoii` có xuất hiện trong `show dbs` không.

<details><summary><b>Lời giải</b></summary><p>Lần đầu ra 16 (sau khi chạy đủ P01). Lần sau ra 0 mà không có lỗi: <code>use</code> chỉ đổi database của phiên, đọc collection không tồn tại trả về rỗng. <code>toinoii</code> không xuất hiện trong <code>show dbs</code> vì chưa có collection nào; database chỉ được tạo khi có dữ liệu đầu tiên (MG-001). Chỉ cần một lệnh ghi nhầm vào đó là nó xuất hiện thật.</p></details>

**Bài 3.** Đồng nghiệp đề xuất bỏ `--replSet rs0` khỏi `command` cho "nhẹ hơn, P01 đâu có dùng transaction". Điều gì xảy ra với `npm run p01`, và với Phần 4?

<details><summary><b>Lời giải</b></summary><p>Trên server không chạy replica set, <code>rs.status()</code> ném lỗi, nhánh <code>catch</code> gọi <code>rs.initiate()</code> và lệnh này cũng ném lỗi vì server không được khởi động với replication; lỗi không được bắt làm healthcheck thoát mã khác 0, nên container không bao giờ "healthy" và <code>npm run db:up</code> báo thất bại. Kể cả khi sửa healthcheck, <code>00-reset.js</code> sẽ in <code>FAIL replica set</code> vì <code>hello()</code> không có <code>setName</code>. Phần 4 cần transaction nhiều document, chỉ có trên replica set hoặc cụm sharding. Giữ replica set từ đầu rẻ hơn dựng lại giữa chừng.</p></details>

**Bài 4.** Bạn cần mở lab cho một đồng nghiệp trong cùng mạng LAN xem thử, và định đổi `127.0.0.1:27018:27017` thành `27018:27017`. Nêu rủi ro và việc phải làm trước.

<details><summary><b>Lời giải</b></summary><p>Bỏ <code>127.0.0.1</code> là mở cổng ra mọi giao diện mạng của máy. Lab không có xác thực, nên bất kỳ ai trong mạng tới được cổng đó đều đọc, sửa, xoá được mọi database. Trước khi mở phải bật xác thực, tạo user có quyền tối thiểu và bật TLS (Phần 5); cách an toàn hơn là để đồng nghiệp tự chạy lab trên máy họ.</p></details>

## 12. Đọc tiếp

- [MongoDB Shell (mongosh)](https://www.mongodb.com/docs/mongodb-shell/): cài đặt, tuỳ chọn dòng lệnh, `--eval`.
- [Connection Strings](https://www.mongodb.com/docs/manual/reference/connection-string/): cú pháp URI và tuỳ chọn `directConnection`.
- [hello](https://www.mongodb.com/docs/manual/reference/command/hello/): ý nghĩa từng trường trong kết quả.
- [Deploy a Self-Managed Replica Set](https://www.mongodb.com/docs/manual/tutorial/deploy-replica-set/): `rs.initiate()` và cấu hình thành viên.
- [Security Checklist](https://www.mongodb.com/docs/manual/administration/security-checklist/): vì sao không bao giờ mở MongoDB không xác thực ra mạng.
- [Ảnh Docker `mongo`](https://hub.docker.com/_/mongo) và [DBStudio](https://github.com/rickymta/dbstudio).
- [PG-002](/articles/postgresql-cai-dat-lab-psql-ket-noi): lab tương ứng của series PostgreSQL.

Bài tiếp theo: [MG-003 · Document, BSON và `_id`: một chuyến xe trông thế nào](/articles/mongodb-document-bson-id).
