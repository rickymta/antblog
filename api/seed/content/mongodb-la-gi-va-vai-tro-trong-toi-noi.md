---
slug: mongodb-la-gi-va-vai-tro-trong-toi-noi
title: "MongoDB 8.0 là gì và nằm ở đâu trong Tới Nơi?"
summary: "Tổng đài muốn mở chuyến 100004 và thấy ngay cước, tài xế, quán và lịch sử trạng thái trong một lần đọc: nếu dữ liệu chuyến nằm trên MongoDB thì bản gốc trông thế nào và ai giữ luật? Bài này phân biệt instance, database, collection, document, giới thiệu replica set và vị trí của MongoDB 8.0 trong Tới Nơi."
category: mongodb
level: beginner
tags: [mongodb, mg-001, mongodb-8, document-database, bson, replica-set, toi-noi]
language: vi
---

Trưa 02/11, anh Võ Minh Đức đặt một suất mì Quảng qua Tới Nơi ở Đà Nẵng. Sáng hôm sau anh gọi tổng đài: "Hôm qua tôi bị trừ 55.000 đ, chuyến đó có thật không, ai giao, giao lúc mấy giờ?" Nhân viên chăm sóc khách hàng cần bốn thông tin trên cùng một màn hình: cước, tài xế, quán và các mốc thời gian của chuyến. Trong series PostgreSQL, bốn thông tin đó nằm ở nhiều bảng và màn hình tra cứu phải ghép chúng lại. Một đồng nghiệp đề xuất: "Sao không lưu cả chuyến thành một hồ sơ, mở ra là đủ?"

Ý tưởng "lưu cả chuyến thành một hồ sơ" chính là cách một **document database** nhìn dữ liệu. Nhưng đổi cách lưu không đổi câu hỏi gốc: **dữ liệu chuyến có đúng một nơi giữ bản gốc không, và nơi đó tự giữ luật tới đâu?** Series [PostgreSQL qua Tới Nơi](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi) và [SQL Server qua Tới Nơi](/articles/mssql-la-gi-va-vai-tro-trong-toi-noi) đặt bản gốc lên database quan hệ. Series này đặt cùng nghiệp vụ, cùng dữ liệu lên **MongoDB 8.0**, nhưng thiết kế theo cơ chế của MongoDB thay vì dịch từng bảng thành collection. Bài đầu tiên trả lời: MongoDB là gì, một "MongoDB" gồm những lớp nào, và nó giữ trách nhiệm gì trong Tới Nơi.

<div class="tkh-callout">
<span class="tkh-callout-title">MG-001 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>. Không cần biết MongoDB hay SQL. Lab: <code>examples/toi-noi-mongodb/p01</code> (MongoDB 8.0.38 chạy trong Docker, dựng ở MG-002). Sau bài này, bạn phân biệt được instance, database, collection, document; biết replica set là gì ở mức khái niệm; biết khi nào document database hợp và không hợp; và nói rõ trách nhiệm lưu dữ liệu nằm ở đâu trong Tới Nơi.</p>
</div>

## 1. Nơi giữ bản gốc phải làm được gì?

**Nguồn dữ liệu gốc** (*source of truth*) là nơi giữ bản dữ liệu có thẩm quyền: khi các bản sao lệch nhau, bản ở đây thắng. Với Tới Nơi, khách hàng, tài xế, quán và chuyến là dữ liệu không được sai, vì chúng là tiền và là lời hứa với ba bên.

Bài [PG-001](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi) đã rút ra bốn yêu cầu với nơi giữ bản gốc. Chúng không phụ thuộc engine, nên MongoDB phải đáp ứng đủ cả bốn, chỉ là bằng công cụ khác:

| Yêu cầu | Ví dụ ở Tới Nơi | PostgreSQL / SQL Server đáp ứng bằng | MongoDB đáp ứng bằng | Học ở |
|---|---|---|---|---|
| **Bền**: đã báo thành công thì sập máy không mất | Chuyến 100004 đã thanh toán 55.000 đ | WAL / transaction log | Journal của WiredTiger, write concern, replica set | Phần 4, 5 |
| **Nhiều bên dùng cùng lúc an toàn** | Hai tài xế bấm nhận cùng một chuyến | Transaction, khoá dòng, MVCC | Ghi **nguyên tử trong một document**, update có điều kiện; transaction nhiều document khi thật cần | Phần 1, 4 |
| **Tự giữ luật dữ liệu** | Số điện thoại không trùng, cước không âm | PRIMARY KEY, FOREIGN KEY, UNIQUE, CHECK | `_id`, index unique, **validator** `$jsonSchema`; **không có khoá ngoại** | Phần 1 |
| **Hỏi được nhiều kiểu** | Chuyến của khách X tháng này, doanh thu theo thành phố | SQL, index, query plan | Truy vấn `find`, aggregation pipeline, index, `explain` | Phần 2, 3 |

Hai ô in đậm ở cột MongoDB là chỗ khác nhau thật sự, và cũng là chỗ người mới hay vấp:

- **Nguyên tử trong một document.** MongoDB đảm bảo một thao tác ghi lên **một document** là nguyên tử (*atomic*): hoặc mọi thay đổi trong document đó được ghi, hoặc không thay đổi nào được ghi. Nếu cả chuyến cùng lịch sử trạng thái nằm trong một document, thì "đổi trạng thái và thêm một dòng lịch sử" là **một** lệnh ghi, không cần transaction. Thiết kế document vì thế quyết định bạn được bảo vệ tới đâu.
- **Không có khoá ngoại.** MongoDB không có ràng buộc kiểu "chuyến phải trỏ tới một khách có thật". Luật đó phải được giữ bằng cách khác: ứng dụng, transaction, và **kiểm tra định kỳ** (lab có hẳn một file làm việc này, MG-010).

Log để điều tra, cache để đọc nhanh, file Excel để chia sẻ báo cáo; **MongoDB giữ bản gốc** trong series này. Phần còn lại của series là cách MongoDB đáp ứng từng yêu cầu, và chỗ nó làm khác PostgreSQL.

## 2. MongoDB 8.0 là gì?

**MongoDB** là một **document database**: hệ quản trị cơ sở dữ liệu lưu dữ liệu thành các **document**, mỗi document là một tập cặp *trường: giá trị* giống một object JSON, có thể lồng object con và mảng. Thay vì ngôn ngữ SQL, client gửi lệnh có cấu trúc (lọc bằng một document, cập nhật bằng toán tử như `$set`, `$push`), gọi chung là **MQL** (*MongoDB Query Language*), cùng **aggregation pipeline** cho truy vấn nhiều bước.

Trên đĩa và trên đường truyền, document không phải chuỗi JSON mà là **BSON** (*Binary JSON*): định dạng nhị phân có nhiều kiểu hơn JSON, như số nguyên 32/64 bit, `Decimal128`, ngày giờ, `ObjectId`, dữ liệu nhị phân (UUID). [MG-003](/articles/mongodb-document-bson-id) mổ một document BSON thật; [MG-004](/articles/mongodb-kieu-bson-tien-thoi-gian-unicode) chọn kiểu BSON cho tiền và thời gian.

Lab của series ghim một bản cụ thể và in nó ra ngay ở file đầu tiên, `00-reset.js` (trích `OUTPUT.txt`):

```text
/* Phiên bản và vai trò của server */
phien_ban|replica_set|vai_tro|database|fcv
---------|-----------|-------|--------|---
8.0.38|rs0|PRIMARY|toinoi|8.0
(1 document)
```

Đọc từng cột:

- `phien_ban` = **8.0.38**: số hiệu của bản phát hành. Dòng 8.0 là một bản phát hành chính; số cuối tăng theo các bản vá.
- `replica_set` = `rs0` và `vai_tro` = `PRIMARY`: server đang chạy trong một **replica set** tên `rs0` và đang là node nhận ghi (mục 5).
- `database` = `toinoi`: database mà phiên kết nối đang làm việc.
- `fcv` = **8.0**: *featureCompatibilityVersion*, công tắc cho biết server được phép dùng những tính năng mới của dòng nào. Khi nâng cấp từ 7.0 lên 8.0, người vận hành chạy binary mới trước, rồi mới nâng FCV; nhờ vậy còn đường lùi nếu có sự cố. Chủ đề này quay lại ở Phần 6.

### Community, Enterprise và Atlas

| Bản | Là gì | Dùng khi | Trong series |
|---|---|---|---|
| **Community Server** | Bản tải về miễn phí, giấy phép SSPL | Tự vận hành trên máy, VM, container | **Lab dùng bản này**, ảnh Docker chính thức `mongo:8.0` |
| **Enterprise Advanced** | Bản thương mại, cùng engine cộng thêm tính năng doanh nghiệp | Cần LDAP/Kerberos, auditing, mã hoá dữ liệu lúc lưu (*encryption at rest*), hỗ trợ từ hãng | Chỉ nhắc tới, ghi rõ "không có trong Community" |
| **MongoDB Atlas** | Dịch vụ MongoDB được quản lý trên AWS, Azure, Google Cloud | Không muốn tự vận hành; cần Atlas Search, Vector Search | Không dùng trong lab |

Engine lưu trữ, MQL, index, transaction, replica set mà series học thì giống nhau giữa ba bản. Khi một bài nhắc tới tính năng chỉ có ở Enterprise hay Atlas, bài sẽ ghi rõ, vì "chạy được trên Atlas" chưa có nghĩa là "chạy được trên server công ty tự dựng".

## 3. Instance, database, collection, document

Bốn từ này là bốn lớp lồng nhau. Người đã học SQL sẽ thấy chúng gần giống instance, database, bảng, dòng, nhưng "gần giống" là chỗ dễ sai nhất.

**Instance** là một tiến trình **`mongod`** đang chạy: nó giữ thư mục dữ liệu, quản lý bộ nhớ, lắng nghe một cổng mạng (mặc định **27017**). Lab có đúng một `mongod` trong container. Cách nó được khởi động đọc được từ chính server:

```js
db.adminCommand({ getCmdLineOpts: 1 }).parsed
```

```text
{
  net: {
    bindIp: '*'
  },
  replication: {
    replSet: 'rs0'
  }
}
```

`replSet: 'rs0'` là lý do server tự nhận là thành viên replica set; `bindIp: '*'` cho phép kết nối từ ngoài container, còn Docker chỉ mở cổng đó trên `127.0.0.1` của máy bạn (MG-002). Khi hệ thống lớn lên và phải chia dữ liệu ra nhiều máy, sẽ có thêm tiến trình **`mongos`** làm bộ định tuyến đứng trước nhiều `mongod`; đó là sharding ở Phần 6.

**Database** là một nhóm collection có tên trong instance. Mỗi instance có sẵn ba **database hệ thống**:

| Database | Vai trò |
|---|---|
| `admin` | Người dùng, vai trò, lệnh quản trị cấp instance (`adminCommand`) |
| `config` | Thông tin nội bộ: session, transaction; với cụm sharding là metadata của cụm |
| `local` | Dữ liệu chỉ thuộc về node này, không được sao sang node khác. Replica set giữ **oplog** ở đây (`local.oplog.rs`) |

Một điểm khác SQL: **database được tạo khi có dữ liệu đầu tiên**, không có bước `CREATE DATABASE`. File `00-reset.js` xoá `toinoi`, kết nối vào đó, rồi liệt kê database; `toinoi` chưa có trong danh sách dù phiên đang "đứng" ở `toinoi` (trích `OUTPUT.txt`):

```text
/* Database có sẵn trong instance (trước khi tạo dữ liệu) */
name
----
admin
config
local
(3 document)
```

Chỉ khi `01-schema.js` tạo collection đầu tiên, `toinoi` mới thật sự xuất hiện.

**Collection** là một nhóm document trong database, tương đương vai trò "bảng" nhưng **không bắt buộc mọi document cùng hình dạng**. Collection cũng được tự tạo ở lần ghi đầu tiên. Tiện, nhưng có mặt trái: gõ sai tên collection khi ghi không báo lỗi, MongoDB lặng lẽ tạo một collection mới. Lab thử đúng bẫy này, ghi vào `trip` (thiếu chữ `s`), trích `OUTPUT.txt`:

```text
PASS B3 collection "trip" (thiếu s) được tự tạo = true
```

Vì thế lab tạo mọi collection **tường minh** bằng `db.createCollection(...)` kèm validator, và luôn kiểm số document sau khi ghi.

**Document** là một bản ghi: một thành phố, một khách, một chuyến. Document nhỏ nhất của lab là một thành phố, đọc trực tiếp từ lab:

```js
db.cities.findOne({ _id: 'DN' })
```

```text
{
  _id: 'DN',
  name: 'Đà Nẵng',
  tz: 'Asia/Ho_Chi_Minh'
}
```

Mọi document có trường **`_id`**, định danh duy nhất trong collection; ở đây `_id` là mã thành phố. Document lớn hơn, như một chuyến với các mốc thời gian và mảng lịch sử, là chủ đề của [MG-003](/articles/mongodb-document-bson-id).

### Nhìn bốn lớp trong lab

Sau khi chạy lab P01 (00 → 07), instance trong container trông như sau:

```text
Instance mongod 8.0.38 (container toinoi-mongodb-lab-db-1, cổng 27018 trên máy bạn)
├── admin, config, local                 ← database hệ thống
└── database toinoi
    ├── cities (3)      customers (6)     drivers (8)
    ├── merchants (3)   ← menu nhúng trong từng quán
    ├── trips (16)      ← lịch sử trạng thái nhúng trong từng chuyến
    └── counters (1)    ← bộ đếm cấp mã chuyến
```

Số trong ngoặc là số document, khớp bảng "Kiểm kê cuối P01" ở cuối `OUTPUT.txt`.

So với lab SQL Server, ở đây **không có** collection `trip_events` hay bảng món ăn: lịch sử trạng thái nằm trong mảng `events` của từng chuyến, món ăn nằm trong mảng `menu` của từng quán. Quyết định nhúng hay tách là bài [MG-008](/articles/mongodb-nhung-hay-tham-chieu).

Trong DBStudio, cây kết nối cho thấy đúng bốn lớp này: dưới kết nối là các database `admin`, `config`, `local`, `toinoi`; mở `toinoi` là sáu collection:

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/mongodb-la-gi-va-vai-tro-trong-toi-noi/dbstudio-01.webp" alt="Cây đối tượng DBStudio: kết nối Tới Nơi lab · MongoDB với database admin, config, local, toinoi và sáu collection của toinoi" width="340" height="470" loading="lazy"></div><figcaption>DBStudio 0.1.7 · cây đối tượng: database <code>toinoi</code> với sáu collection</figcaption></figure>

Công cụ GUI vẽ cây này bằng cách gọi các lệnh như `listDatabases` và `listCollections`; bạn không cần nhớ chúng lúc này, chỉ cần biết mọi câu hỏi "có những gì trong instance" đều trả lời được bằng lệnh.

## 4. "Schema linh hoạt" không có nghĩa là không có luật

MongoDB thường được giới thiệu là *schemaless*. Chính xác hơn: **MongoDB không bắt buộc schema**. Một collection không có validator nhận mọi document, kể cả document thiếu `fare`, có `fare` là chuỗi, hay có thêm trường `ghi_chu_tam` mà không ai biết.

Với dữ liệu tiền của Tới Nơi, "nhận mọi thứ" là rủi ro. Vì vậy lab **khai báo validator ngay từ đầu**: một bộ luật `$jsonSchema` (kiểu BSON, trường bắt buộc, danh sách giá trị cho phép, cấm trường lạ) cộng các quy tắc liên trường như "chuyến đồ ăn phải có quán". MongoDB kiểm bộ luật này ở **server**, mỗi lần insert hay update, bất kể ghi từ ứng dụng, script hay công cụ GUI. Kết quả của `01-schema.js` (trích `OUTPUT.txt`):

```text
/* Collection và validator */
name|co_validator|validationLevel|validationAction
----|------------|---------------|----------------
cities|true|(mặc định)|(mặc định)
counters|false|(mặc định)|(mặc định)
customers|true|(mặc định)|(mặc định)
drivers|true|(mặc định)|(mặc định)
merchants|true|(mặc định)|(mặc định)
trips|true|strict|error
(6 document)
PASS collection có validator = 5
```

Năm collection nghiệp vụ có validator; chỉ `counters` (bộ đếm nội bộ) để trống. Giá trị mặc định của `validationLevel` và `validationAction` là `strict` và `error`, nghĩa là mọi document ghi vào đều bị kiểm và ghi sai thì bị từ chối; `trips` ghi rõ hai giá trị đó cho người đọc khỏi phải nhớ. Cách viết validator và đọc lỗi 121 là bài [MG-005](/articles/mongodb-schema-validation-unique-index).

Vậy "linh hoạt" nằm ở đâu? Ở chỗ **hình dạng document được phép khác nhau trong giới hạn luật**: chuyến chưa có tài xế thì không có trường `driver_id`, chuyến đặt xe thì không có `merchant_id`. Đổi schema cũng không cần `ALTER TABLE` khoá bảng; bù lại bạn phải tự quản phiên bản schema ([MG-009](/articles/mongodb-schema-migration-tuong-thich)).

### Luật mà validator không giữ được

Validator chỉ nhìn **một document** tại thời điểm ghi. Nó không biết khách `c0000000-0000-4000-8000-0000000000ff` có tồn tại trong `customers` hay không. Lab thử chèn một chuyến trỏ tới khách không có thật, và MongoDB nhận (trích `OUTPUT.txt`):

```text
/* Bẫy: KHÔNG có lỗi, dữ liệu sai lọt vào */
PASS B1 chuyến của khách không tồn tại đã vào = 1
PASS B2 tài xế ở thành phố không tồn tại đã vào = 1
```

Ở PostgreSQL, khoá ngoại chặn ngay câu lệnh này. Ở MongoDB, lab bù bằng hai lớp: ứng dụng kiểm trước khi ghi, và một bộ **kiểm tra bất biến** chạy định kỳ, tìm tham chiếu mồ côi bằng `$lookup` (trích `OUTPUT.txt` của `07-checks.js`):

```text
/* Tham chiếu (thay cho khoá ngoại) */
PASS K1 chuyến trỏ tới khách không tồn tại = 0
PASS K2 chuyến trỏ tới tài xế không tồn tại = 0
PASS K3 chuyến trỏ tới quán không tồn tại = 0
```

Đây là đánh đổi phải chấp nhận khi chọn MongoDB: một phần luật dời từ "database chặn" sang "hệ thống phát hiện".

## 5. Replica set: một node hôm nay, nhiều node về sau

**Replica set** là một nhóm `mongod` giữ cùng một bộ dữ liệu. Một node là **primary**, nhận mọi lệnh ghi; các node còn lại là **secondary**, chép lại các thay đổi của primary qua **oplog** (*operation log*: nhật ký thao tác nằm trong database `local`). Khi primary hỏng, các node còn lại bầu primary mới. Production thường chạy ít nhất ba node.

Lab chạy replica set **một node** tên `rs0`. Một node thì chưa có dự phòng, vậy dựng replica set để làm gì? Vì một số tính năng **chỉ có trên replica set** (hoặc cụm sharding):

- **Transaction nhiều document** (Phần 4);
- **Change stream**, theo dõi thay đổi theo thời gian thực, đọc từ oplog (Phần 6);
- Read concern `majority`, write concern theo số node (Phần 4).

Dựng sẵn từ P01 thì mọi phần dùng chung một môi trường, không phải dựng lại giữa chừng. Lệnh `db.hello()` cho biết node đang đóng vai gì; chạy trên lab và chỉ giữ vài trường:

```js
const h = db.hello();
printjson({ isWritablePrimary: h.isWritablePrimary, setName: h.setName, hosts: h.hosts, me: h.me,
            maxBsonObjectSize: h.maxBsonObjectSize, maxWireVersion: h.maxWireVersion });
```

```text
{
  isWritablePrimary: true,
  setName: 'rs0',
  hosts: [
    'localhost:27017'
  ],
  me: 'localhost:27017',
  maxBsonObjectSize: 16777216,
  maxWireVersion: 25
}
```

- `isWritablePrimary: true`: node này nhận ghi.
- `hosts`: danh sách thành viên mà replica set **tự khai báo**. Driver dùng danh sách này để tìm primary. Địa chỉ `localhost:27017` là địa chỉ **bên trong container**; MG-002 giải thích vì sao từ máy bạn phải thêm `directConnection=true`.
- `maxBsonObjectSize: 16777216`: một document tối đa 16 MB, giới hạn sẽ gặp lại ở MG-003 và MG-008.

## 6. MongoDB nằm ở đâu trong Tới Nơi?

Hồ sơ [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi) chốt **PostgreSQL là database chủ đạo** của giai đoạn đầu. MongoDB có mặt trong lộ trình cho những dữ liệu hợp mô hình document (kết quả OCR, hồ sơ phân tích, read model) và để học so sánh khi có số liệu đo. Hồ sơ nói rõ: MongoDB được chọn theo mô hình dữ liệu và cách truy cập, không phải thuốc chữa cho mọi vấn đề hiệu năng.

Series này làm một thí nghiệm có chủ đích: đặt **toàn bộ** nghiệp vụ chuyến lên MongoDB, cùng dữ liệu với lab PostgreSQL và SQL Server, để thấy cơ chế của MongoDB giải quyết từng yêu cầu thế nào và chỗ nào phải trả giá. Đây là bối cảnh học, không phải khuyến nghị chuyển hệ thống thật sang MongoDB.

| Việc | Ai chịu trách nhiệm | Ví dụ Tới Nơi trong lab |
|---|---|---|
| Lưu bền, không mất dữ liệu đã xác nhận | MongoDB (journal, write concern, replica set) | Chuyến 100004 còn nguyên sau khi container khởi động lại |
| Luật trong một document | MongoDB (validator, index unique) | Cước không âm, số điện thoại không trùng, đồ ăn thì phải có quán |
| Luật giữa nhiều document | Ứng dụng + kiểm tra định kỳ (+ transaction khi cần) | Chuyến trỏ tới khách có thật; tài xế `busy` khi và chỉ khi có đúng một chuyến đang chạy |
| Đổi trạng thái an toàn khi nhiều bên cùng ghi | MongoDB (update có điều kiện, nguyên tử trong một document) | Hai tài xế cùng bấm nhận một chuyến, chỉ một người thắng |
| Quyết định nghiệp vụ | Ứng dụng | Chọn tài xế nào, tính cước bao nhiêu |

Dòng thứ tư có bằng chứng trong lab. Ở `04-lifecycle.js`, tài xế 8007 và 8003 cùng gửi lệnh "nhận chuyến nếu chuyến vẫn đang `searching`". Điều kiện và thay đổi nằm trong **một** lệnh ghi lên **một** document, nên người đến sau không khớp document nào (trích `OUTPUT.txt`):

```text
/* Hai tài xế cùng bấm nhận (8007 trước, 8003 sau) */
PASS 8007 nhận: matched = 1
PASS 8003 nhận muộn: matched (không ghi gì) = 0
PASS tài xế của chuyến = d0000000-0000-4000-8000-000000008007
```

Không cần transaction, không cần khoá tường minh: thiết kế document đặt trạng thái, tài xế và lịch sử vào cùng một chỗ, và tính nguyên tử trong một document làm phần còn lại. [MG-006](/articles/mongodb-insert-update-delete) đi hết vòng đời chuyến này.

### Khi nào document database hợp, khi nào không

| Dấu hiệu | Hợp với document database | Cân nhắc kỹ / nghiêng về database quan hệ |
|---|---|---|
| Cách đọc | Thường đọc **cả một thực thể** cùng phần con của nó (chuyến + lịch sử, quán + menu) | Thường ghép nhiều thực thể độc lập theo nhiều chiều khác nhau |
| Phần con | Có giới hạn, thuộc về đúng một cha, ít khi đọc riêng | Lớn dần không giới hạn, hoặc dùng chung giữa nhiều cha |
| Hình dạng dữ liệu | Khác nhau giữa các bản ghi (kết quả OCR mỗi loại giấy tờ một kiểu) | Đồng nhất, ổn định |
| Luật liên thực thể | Ít, hoặc chấp nhận kiểm định kỳ | Nhiều luật phải chặn tức thì (khoá ngoại, ràng buộc giữa bảng) |
| Ghi nhiều thực thể cùng lúc | Hiếm; phần lớn thao tác chỉ chạm một document | Thường xuyên, mỗi nghiệp vụ chạm nhiều bảng trong một transaction |
| Báo cáo | Theo một vài trục đã biết trước | Ad-hoc, nhiều chiều, nhiều JOIN |

Dữ liệu chuyến của Tới Nơi nằm ở giữa: lịch sử trạng thái và menu rất hợp để nhúng, nhưng quan hệ chuyến–khách–tài xế–quán là tham chiếu chéo cần kiểm. Series sẽ đi qua cả hai mặt, kể cả những chỗ MongoDB phải làm vất vả hơn.

## 7. So với PostgreSQL / SQL Server

| Khái niệm | PostgreSQL | SQL Server | MongoDB |
|---|---|---|---|
| Đơn vị server | Database cluster | Instance | Instance (`mongod`); cụm sharding có thêm `mongos` |
| Lớp chứa dữ liệu | Database → schema → bảng | Database → schema → bảng | Database → collection (không có lớp schema) |
| Một bản ghi | Dòng, cột cố định | Dòng, cột cố định | Document BSON, có object lồng và mảng |
| Tạo database / bảng | `CREATE DATABASE`, `CREATE TABLE` | `CREATE DATABASE`, `CREATE TABLE` | Tự tạo ở lần ghi đầu; lab vẫn `createCollection` tường minh |
| Ngôn ngữ | SQL + PL/pgSQL | T-SQL | MQL, aggregation pipeline |
| Luật dữ liệu | PK, FK, UNIQUE, CHECK | PK, FK, UNIQUE, CHECK | `_id`, index unique, validator; **không có FK** |
| Nguyên tử | Transaction bao mọi câu lệnh | Transaction bao mọi câu lệnh | Mặc định trong **một document**; transaction nhiều document trên replica set |
| Nhân bản | Streaming replication (WAL) | Availability group, log shipping | Replica set (oplog) |

Bảng này chỉ để định vị. Nhiều chữ giống nhau nhưng cơ chế khác: "transaction" ở MongoDB có giới hạn thời gian và kích thước riêng; "index" cũng là B-tree nhưng có index trên mảng. Mỗi bài sau có mục so sánh ngắn khi cơ chế thật sự khác.

## 8. Những hiểu nhầm thường gặp

- **"MongoDB không có schema nên không cần thiết kế."** MongoDB không *bắt buộc* schema, nhưng dữ liệu tiền vẫn cần luật. Không khai báo validator nghĩa là mọi đường ghi (API, job, script, người sửa tay) đều có thể làm bẩn dữ liệu.
- **"Collection là bảng, document là dòng, đổi tên là xong."** Dịch bảng sang collection một-một sẽ mất lợi thế lớn nhất (nhúng dữ liệu đọc cùng nhau, nguyên tử trong một document) mà vẫn chịu điểm yếu (không có khoá ngoại).
- **"MongoDB không có transaction."** MongoDB có transaction nhiều document trên replica set và cụm sharding. Nhưng thiết kế tốt thường không cần đến nó cho phần lớn thao tác.
- **"Gõ sai tên collection thì sẽ báo lỗi."** Khi ghi, MongoDB tạo collection mới mà không báo gì; khi đọc, collection không tồn tại chỉ trả về rỗng. Đếm lại sau khi ghi, và tạo collection tường minh.
- **"Replica set một node là thừa."** Trên lab, nó không để dự phòng mà để có transaction, change stream và oplog giống môi trường thật.

## 9. Bài tập

**Bài 1.** Sắp xếp từ lớn tới nhỏ và nêu tên cụ thể trong lab: document, collection, instance, database, trường.

<details><summary><b>Lời giải</b></summary><p>Instance (tiến trình <code>mongod</code> 8.0.38 trong container, replica set <code>rs0</code>) → database (<code>toinoi</code>) → collection (ví dụ <code>trips</code>) → document (ví dụ chuyến có <code>code</code> 100004) → trường (ví dụ <code>fare</code> = 55000).</p></details>

**Bài 2.** Với mỗi luật, nói MongoDB tự chặn được lúc ghi hay phải kiểm bằng cách khác: (a) cước không âm; (b) không có hai khách trùng số điện thoại; (c) chuyến phải trỏ tới một khách có thật; (d) tài xế đang `busy` thì phải có đúng một chuyến đang chạy.

<details><summary><b>Lời giải</b></summary><p>(a) Chặn được bằng validator (<code>minimum: 0</code>). (b) Chặn được bằng index unique trên <code>phone</code> (lỗi 11000). (c) Không: không có khoá ngoại; kiểm ở ứng dụng và bằng kiểm tra định kỳ dùng <code>$lookup</code> (K1 trong <code>07-checks.js</code>). (d) Không: luật này nằm trên hai collection (<code>drivers</code> và <code>trips</code>); giữ bằng cách ghi cẩn thận (có thể dùng transaction, Phần 4) và kiểm định kỳ (K7).</p></details>

**Bài 3.** Một đồng nghiệp chạy `db.trip.countDocuments()` và nhận `0`, trong khi bạn vừa đếm được 16 chuyến. Liệt kê ba khả năng và cách kiểm từng khả năng.

<details><summary><b>Lời giải</b></summary><p>(1) Gõ sai tên collection (<code>trip</code> thay cho <code>trips</code>): đọc collection không tồn tại không báo lỗi; chạy <code>db.getCollectionNames()</code>. (2) Đang ở database khác (mongosh không ghi database trong URI sẽ vào <code>test</code>): chạy <code>db.getName()</code>. (3) Lab vừa được dựng lại (<code>00-reset.js</code> chạy mà chưa chạy <code>02</code>): đếm từng collection, so với bảng kiểm kê ở mục 3. Không kết luận "mất dữ liệu" trước khi loại trừ ba khả năng này.</p></details>

**Bài 4.** Đề xuất: "Dùng MongoDB cho dữ liệu chuyến vì nó không có khoá ngoại nên ghi nhanh hơn." Nêu một điểm đúng và một điểm thiếu của lập luận này.

<details><summary><b>Lời giải</b></summary><p>Đúng: không kiểm khoá ngoại thì mỗi lệnh ghi bớt một bước tra cứu. Thiếu: luật "chuyến trỏ tới khách có thật" vẫn phải được giữ, nên chi phí chỉ dời sang ứng dụng và job kiểm tra, kèm rủi ro dữ liệu sai tồn tại trong khoảng giữa hai lần kiểm. Lý do chọn MongoDB nên là mô hình dữ liệu và cách đọc (nhúng lịch sử, nguyên tử trong một document), và phải có số liệu đo trên cùng tải trước khi nói "nhanh hơn".</p></details>

## 10. Đọc tiếp

- [Introduction to MongoDB](https://www.mongodb.com/docs/manual/introduction/): tổng quan document database và MQL.
- [Databases and Collections](https://www.mongodb.com/docs/manual/core/databases-and-collections/): database và collection được tạo ngầm như thế nào.
- [Replication](https://www.mongodb.com/docs/manual/replication/): primary, secondary, oplog, bầu chọn.
- [Schema Validation](https://www.mongodb.com/docs/manual/core/schema-validation/): validator `$jsonSchema`, `validationLevel`, `validationAction`.
- [Atomicity and Transactions](https://www.mongodb.com/docs/manual/core/write-operations-atomicity/): nguyên tử trong một document.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): số liệu tải và vai trò của từng database.
- [PG-001](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi) và [MS-001](/articles/mssql-la-gi-va-vai-tro-trong-toi-noi): cùng câu hỏi, trả lời bằng PostgreSQL và SQL Server.

Bài tiếp theo: [MG-002 · Dựng MongoDB 8.0 bằng Docker và kết nối bằng DBStudio](/articles/mongodb-cai-dat-docker-dbstudio).
