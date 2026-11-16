---
slug: cassandra-la-gi-va-vai-tro-trong-toi-noi
title: "Apache Cassandra là gì và nằm ở đâu trong Tới Nơi?"
summary: "6.250 điểm GPS mỗi giây và 16 triệu sự kiện chuyến mỗi ngày: dữ liệu nào của Tới Nơi nên đặt vào Apache Cassandra 5.0, dữ liệu nào phải ở lại PostgreSQL? Bài này giải thích cluster, datacenter, node, keyspace, bảng, partition, dòng, ô, kiến trúc không có master và ranh giới trách nhiệm của Cassandra."
category: cassandra
level: beginner
tags: [cassandra, cs-001, cassandra-5, cql, partition, keyspace, masterless, toi-noi]
language: vi
---

18:00 thứ Hai, giờ tan tầm ở TP.HCM. 25.000 tài xế Tới Nơi đang chạy, mỗi điện thoại gửi vị trí về máy chủ 4 giây một lần: khoảng **6.250 điểm GPS mỗi giây**. Cùng lúc đó, mỗi chuyến đi qua sáu đến tám trạng thái (`requested` → `searching` → `accepted` → … → `completed`), và mỗi lần đổi trạng thái sinh ra một sự kiện: khoảng **16 triệu dòng `trip_events` mỗi ngày**. Khách mở ứng dụng xem "Chuyến của tôi", tài xế xem "Thu nhập hôm nay", màn hình điều phối xem chuyến mới theo từng giờ.

Series [PostgreSQL qua Tới Nơi](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi) đặt bản gốc của chuyến, tiền và ví vào PostgreSQL. Dồn thêm dòng GPS và dòng sự kiện vào cùng database đó thì database phải gánh hàng chục nghìn lượt ghi mỗi giây, bảng phình ra hàng tỷ dòng mỗi năm và chỉ có một máy nhận ghi. Series này giới thiệu **Apache Cassandra 5.0**, một database được thiết kế cho đúng kiểu tải ấy, và trả lời câu hỏi quan trọng hơn: **dữ liệu nào nên vào Cassandra, dữ liệu nào không**.

<div class="tkh-callout">
<span class="tkh-callout-title">CS-001 · Phần 1/6 · Cơ bản</span>
<p>Đọc trước: <a href="/articles/du-an-mau-toi-noi">Dự án mẫu Tới Nơi</a>. Không cần biết CQL hay Cassandra. Lab: <code>examples/toi-noi-cassandra</code> (Cassandra 5.0.9 chạy trong Docker, dựng ở CS-002). Sau bài này, bạn phân biệt được cluster, datacenter, node, keyspace, bảng, partition, dòng và ô; giải thích được vì sao Cassandra không có máy chủ chính; và chọn được dữ liệu nào của Tới Nơi hợp với Cassandra.</p>
</div>

## 1. Apache Cassandra là gì?

**Apache Cassandra** là một **database phân tán** (*distributed database*) mã nguồn mở của Apache Software Foundation: dữ liệu được chia nhỏ và rải trên nhiều máy, mỗi phần có thể có nhiều bản sao, và cả cụm máy hoạt động như một database duy nhất. Cassandra thuộc nhóm **wide-column store**: dữ liệu vẫn nằm trong bảng có cột, nhưng cách tổ chức bên dưới khác hẳn bảng của database quan hệ (mục 3).

Ngôn ngữ truy vấn của Cassandra là **CQL** (*Cassandra Query Language*). CQL trông giống SQL (`CREATE TABLE`, `SELECT … WHERE`, `INSERT`, `UPDATE`), nhưng không có JOIN, không có transaction nhiều dòng, và từ chối mọi truy vấn có thể phải quét toàn bộ dữ liệu. Những giới hạn đó không phải thiếu sót mà là cái giá để đổi lấy khả năng mở rộng.

Cassandra nhắm tới ba mục tiêu:

| Mục tiêu | Nghĩa là gì | Ví dụ ở Tới Nơi |
|---|---|---|
| **Ghi rất nhiều, ghi đều** | Nhận lượng ghi lớn liên tục, tốc độ ghi không chậm dần khi bảng lớn lên | 6.250 điểm GPS/giây chỉ là phần vị trí |
| **Mở rộng ngang** | Thêm máy thì thêm sức chứa và sức ghi, không cần máy to hơn | Dữ liệu sự kiện tăng theo số chuyến, không có trần |
| **Luôn nhận ghi** | Một vài máy chết, hệ thống vẫn đọc ghi được | Tài xế vẫn gửi vị trí khi một node đang bảo trì |

Lab của series in phiên bản ngay ở file đầu tiên (`p01/00-reset.cql`):

```sql
SELECT cluster_name, release_version, data_center, rack, cql_version FROM system.local;
```

```text
 cluster_name | release_version | data_center | rack  | cql_version
--------------+-----------------+-------------+-------+-------------
   ToiNoi Lab |           5.0.9 |         dc1 | rack1 |       3.4.7
```

`release_version` là bản Cassandra (5.0.9), `cql_version` là phiên bản ngôn ngữ CQL mà node hỗ trợ (3.4.7). Hai số này đi riêng, đừng nhầm.

## 2. Từ cluster tới ô: các lớp của Cassandra

Database quan hệ có instance → database → schema → bảng → dòng. Cassandra có hai nhóm lớp: lớp **vật lý** (máy nào chạy) và lớp **logic** (dữ liệu được tổ chức thế nào).

### Lớp vật lý: cluster, datacenter, rack, node

- **Node** là một tiến trình Cassandra đang chạy, thường mỗi máy (hoặc mỗi container) một node. Mỗi node giữ một phần dữ liệu trên đĩa của nó.
- **Rack** là nhóm node có chung rủi ro hỏng (cùng tủ máy, cùng nguồn điện). Cassandra cố đặt các bản sao ở rack khác nhau.
- **Datacenter** (DC) là nhóm rack, thường tương ứng một trung tâm dữ liệu hoặc một vùng cloud. Datacenter ở đây là khái niệm *logic* do người vận hành khai báo.
- **Cluster** (cụm) là tập mọi node cùng chia nhau một vòng dữ liệu, có chung tên cluster.

Lab Phần 1 chỉ có **một node** trong cluster `ToiNoi Lab`, datacenter `dc1`, rack `rack1` (kết quả ở mục 1). Phần 3 dựng cụm ba node.

### Lớp logic: keyspace, bảng, partition, dòng, ô

- **Keyspace** là không gian tên chứa bảng, tương tự database của PostgreSQL. Điểm khác quan trọng: keyspace mang **chiến lược nhân bản** (*replication*), tức là mỗi dòng có bao nhiêu bản sao ở mỗi datacenter. Lab dùng keyspace `toinoi` với 1 bản sao ở `dc1`.
- **Bảng** (*table*) có cột với kiểu xác định và một **khoá chính** (*primary key*) bắt buộc.
- **Partition** là nhóm dòng có cùng **partition key** (phần đầu của khoá chính). Partition là đơn vị phân tán: mọi dòng của một partition nằm cùng nhau trên cùng các node, và Cassandra quyết định node nào giữ partition bằng cách băm partition key thành một con số gọi là **token**.
- **Dòng** (*row*) trong partition được phân biệt và **sắp xếp sẵn** theo **clustering key** (phần còn lại của khoá chính).
- **Ô** (*cell*) là giá trị của một cột trong một dòng. Mỗi ô mang theo **thời điểm ghi** riêng (`WRITETIME`), nên một dòng thực chất là tập các ô được ghi vào lúc khác nhau. CS-005 dùng tính chất này để giải thích vì sao mọi câu ghi đều là upsert.

Ví dụ với bảng `trips_by_customer` của lab: partition là "mọi chuyến của khách 1", dòng là "chuyến 11 của khách 1 lúc 17:45 ngày 03/11", ô là "cước của chuyến 11". Khoá chính của bảng viết là `PRIMARY KEY ((customer_id), requested_at, trip_id)`: `customer_id` là partition key, `requested_at` và `trip_id` là clustering key. CS-003 đọc kỹ cú pháp này.

<figure class="tkh-fig">
<div class="tkh-fig-box"><img src="/images/articles/cassandra/cs-001/01-cay-keyspace-va-ddl.webp" alt="DBStudio: cây đối tượng bên trái gồm kết nối toinoi-lab có huy hiệu CA, keyspace toinoi, nhóm Bảng với sáu bảng trip_events_by_trip, trips, trips_by_city_day, trips_by_city_hour, trips_by_customer, trips_by_driver_day. Bên phải là DDL của trips_by_customer do DBStudio sinh: các cột, PRIMARY KEY (customer_id, requested_at, trip_id), WITH CLUSTERING ORDER BY (requested_at DESC, trip_id ASC) và các tuỳ chọn mặc định như compaction SizeTieredCompactionStrategy, compression LZ4Compressor, gc_grace_seconds = 864000." width="2560" height="1520" loading="lazy"></div>
<figcaption><b>Hình 1 · Keyspace toinoi và DDL của một bảng trong DBStudio.</b> Cây bên trái đi từ kết nối (cluster) xuống keyspace rồi tới bảng; không có lớp "schema" như PostgreSQL. Bên phải, dòng 11 và 13 là phần cần đọc: khoá chính và thứ tự clustering. Các dòng <code>AND …</code> phía dưới là tuỳ chọn mặc định của bảng (compaction, nén, <code>gc_grace_seconds</code>), sẽ được giải thích dần ở Phần 3–5. Bảng <code>trips_by_city_day</code> là bảng thí nghiệm của CS-009.</figcaption>
</figure>

Cây keyspace bên trái chỉ có hai tầng dưới kết nối: keyspace và bảng. DDL bên phải không do bạn gõ mà do DBStudio đọc từ catalog `system_schema` và sinh lại, kèm mọi tuỳ chọn mặc định mà câu `CREATE TABLE` gốc không nêu.

## 3. Không có máy chủ chính: kiến trúc peer-to-peer

Hệ PostgreSQL có replication thường có **một primary** nhận mọi lượt ghi và các standby chỉ đọc. Primary chết thì phải chuyển vai (failover) trước khi ghi tiếp.

Cassandra không có vai trò đó. Mọi node **ngang hàng** (*peer-to-peer*, còn gọi là *masterless*):

- Client gửi câu lệnh tới **bất kỳ node nào**. Node nhận câu lệnh đóng vai **coordinator** (node điều phối) cho riêng câu đó: tính token của partition key, tìm các node đang giữ bản sao của partition, chuyển câu ghi hoặc câu đọc tới chúng và gom kết quả.
- Các node tự biết nhau và biết node nào còn sống qua giao thức **gossip**: mỗi giây, mỗi node trao đổi thông tin trạng thái với vài node khác (CS-025).
- Không có điểm hỏng duy nhất: một node chết thì các bản sao ở node khác vẫn phục vụ, miễn là đủ bản sao còn sống cho mức nhất quán mà câu lệnh yêu cầu.

Đổi lại, Cassandra không có một nơi duy nhất "biết chắc" trạng thái mới nhất của mọi dòng. Đó là lý do mục 5 tồn tại.

## 4. Vì sao Cassandra ghi nhanh?

Khi nhận một câu ghi, mỗi node giữ bản sao chỉ làm hai việc:

1. **Nối thêm** (*append*) bản ghi vào cuối **commit log** trên đĩa, để không mất dữ liệu nếu node sập.
2. Ghi vào **memtable**, một cấu trúc dữ liệu đã sắp xếp nằm trong bộ nhớ.

Rồi trả lời "đã ghi". Không đọc dòng cũ, không tìm chỗ trên đĩa để sửa tại chỗ, không kiểm khoá ngoại. Khi memtable đầy, node **flush** nó xuống đĩa thành một file bất biến gọi là **SSTable**; các SSTable được gộp dần ở nền bằng **compaction**. Toàn bộ đường ghi chỉ là ghi tuần tự, nên tốc độ ghi gần như không phụ thuộc kích thước bảng.

Cái giá phải trả nằm ở đường đọc (một dòng có thể nằm rải trên nhiều SSTable) và ở việc xoá (xoá cũng là ghi thêm một dấu xoá gọi là **tombstone**). Phần 3 (CS-027, CS-028) và CS-031 mổ xẻ chi tiết; ở Phần 1 bạn chỉ cần nhớ: **Cassandra tối ưu cho ghi, và bạn phải thiết kế bảng để đọc rẻ**.

## 5. AP và độ nhất quán chọn theo từng câu lệnh

**Định lý CAP** nói rằng khi mạng bị chia cắt (*partition tolerance*, P), một hệ phân tán phải chọn giữa trả lời nhất quán (*consistency*, C) và luôn trả lời (*availability*, A). Cassandra thường được xếp vào nhóm **AP**: mặc định nó ưu tiên tiếp tục nhận ghi và trả lời đọc, chấp nhận các bản sao có thể lệch nhau trong chốc lát rồi tự đồng bộ lại.

Điểm đặc biệt là mức đánh đổi **không cố định cho cả hệ thống**. Mỗi câu lệnh mang một **consistency level** (mức nhất quán) cho biết coordinator phải chờ bao nhiêu bản sao trả lời:

| Consistency level | Coordinator chờ | Hợp với |
|---|---|---|
| `ONE` | 1 bản sao | Điểm GPS: mất một điểm không sao, cần nhanh và luôn ghi được |
| `QUORUM` / `LOCAL_QUORUM` | Quá nửa số bản sao (trong cả cụm / trong datacenter của coordinator) | Lịch sử chuyến khách xem lại, chat: cần đọc thấy cái vừa ghi |
| `ALL` | Mọi bản sao | Hiếm khi dùng: một node chết là câu lệnh thất bại |

Với 3 bản sao, ghi `QUORUM` (chờ 2) rồi đọc `QUORUM` (chờ 2) thì luôn có ít nhất một bản sao chung, nên đọc thấy dữ liệu vừa ghi. CS-024 chứng minh quy tắc này trên cụm ba node. Lab Phần 1 chỉ có một node, nên mọi mức đều như nhau.

## 6. Dữ liệu nào của Tới Nơi hợp Cassandra?

Cassandra hợp với dữ liệu có ba đặc điểm: **ghi nhiều, ít sửa**; **luôn đọc theo một khoá biết trước** (tài xế X, chuyến Y, khách Z); và **không cần ràng buộc giữa các bảng**. Đối chiếu với hồ sơ Tới Nơi:

| Dữ liệu | Tải (hồ sơ Tới Nơi) | Đọc theo | Vì sao hợp Cassandra |
|---|---|---|---|
| Vị trí tài xế | ≈ 6.250 điểm/giây lúc cao điểm | Tài xế + vài phút gần nhất | Chỉ chèn, tự hết hạn bằng TTL, mất vài điểm không sai nghiệp vụ (CS-035) |
| Dòng thời gian sự kiện chuyến (`trip_events`) | ~16 triệu dòng/ngày | Mã chuyến, theo thứ tự thời gian | Chỉ chèn, mỗi chuyến là một partition nhỏ đã sắp sẵn |
| Lịch sử chuyến theo khách, theo tài xế | 2 triệu chuyến/ngày, ≈ 730 triệu dòng/năm | Khách hoặc tài xế + khoảng thời gian | Bản sao chỉ-đọc của chuyến, sắp sẵn mới nhất trước |
| Chat tài xế–khách | Theo số chuyến | Cuộc trò chuyện, theo thời gian | Chèn nhiều, đọc theo partition (CS-020) |
| Hộp thư thông báo | Theo số người dùng | Người nhận, mới nhất trước | Chèn nhiều, dữ liệu cũ tự hết hạn |
| Bộ đếm, khoá idempotency | Theo số sự kiện | Khoá biết trước | Bảng counter, ghi có điều kiện kèm TTL (CS-014, CS-039) |

Ngược lại, có những dữ liệu **không** nên đặt vào Cassandra:

| Dữ liệu | Vì sao không | Ở đâu |
|---|---|---|
| Số dư ví, thanh toán, hoàn tiền | Cần trừ tiền và ghi sổ trong **một transaction** trên nhiều dòng, nhiều bảng; Cassandra không có | PostgreSQL / SQL Server |
| Trạng thái chuyến "đang sống" (ai đang nhận chuyến nào) | Cần ràng buộc liên bảng và đổi trạng thái có kiểm tra; ≈ 560 UPDATE/giây vào bảng `trips` là việc của database quan hệ | PostgreSQL |
| Danh mục khách, tài xế, quán | Ít ghi, cần UNIQUE (số điện thoại), khoá ngoại, truy vấn tuỳ ý | PostgreSQL |
| Báo cáo doanh thu tuỳ biến | Hỏi theo nhiều chiều không biết trước; Cassandra chỉ trả lời tốt câu hỏi đã thiết kế sẵn | Kho phân tích (CS-055) |

Lab Phần 1 giữ bảng `trips` trong Cassandra để có bản ghi tra theo mã chuyến, nhưng đó là **bản sao phục vụ đọc**. Bản gốc của tiền và trạng thái chuyến vẫn ở PostgreSQL. CS-052 vẽ lại ranh giới này cho cả kiến trúc.

## 7. Edition và bản phân phối

| Sản phẩm | Là gì |
|---|---|
| **Apache Cassandra** | Bản mã nguồn mở, giấy phép Apache 2.0. Series dùng bản này (ảnh Docker chính thức `cassandra:5.0.9`) |
| **DataStax Enterprise, Astra DB** | Bản thương mại và dịch vụ đám mây của DataStax, xây trên Cassandra, có thêm tính năng riêng. Series không dùng tính năng chỉ có ở đây |
| **ScyllaDB** | Database khác, viết lại bằng C++, **tương thích giao thức và CQL** của Cassandra. So sánh ở CS-056 |

Ngoài ra các nhà cung cấp cloud có dịch vụ tương thích Cassandra; mức tương thích mỗi dịch vụ một khác, nên mọi kết quả trong series chỉ khẳng định cho Apache Cassandra 5.0.

## 8. Cassandra 5.0 có gì mới?

Cassandra 5.0 (phát hành tháng 9/2024) là bản lớn sau 4.1. Những thay đổi series sẽ gặp:

- **Storage-Attached Index (SAI)**: index phụ được xây cùng memtable và SSTable của bảng, cho phép lọc theo cột không thuộc khoá chính hiệu quả hơn secondary index cũ (CS-018).
- **Kiểu `vector` và tìm kiếm vector** trên SAI, phục vụ tìm theo độ tương đồng (CS-053).
- **Unified Compaction Strategy (UCS)**: chiến lược compaction mới, điều chỉnh được để hành xử giống các chiến lược cũ (CS-029).
- **Trie memtable và định dạng SSTable dạng trie (BTI)**: cấu trúc lưu trữ mới, tiết kiệm bộ nhớ hơn.
- Chạy được trên **Java 17**, thêm hàm toán học và **dynamic data masking**.

Một điểm cần biết ngay: file cấu hình mặc định `cassandra.yaml` của 5.0 **vẫn giữ hành vi cũ** để nâng cấp an toàn. Trong lab, memtable mặc định là `SkipListMemtable`, `storage_compatibility_mode` là `CASSANDRA_4`, và bảng mới vẫn dùng compaction `SizeTieredCompactionStrategy` (thấy ở dòng 17 Hình 1). Ảnh Docker có kèm `cassandra_latest.yaml` bật sẵn các mặc định mới; series nói rõ mỗi khi bật một tính năng như vậy.

## 9. So với PostgreSQL

| Khái niệm | PostgreSQL | Cassandra |
|---|---|---|
| Ghi | Một primary nhận ghi | Mọi node nhận ghi |
| Chia dữ liệu | Một máy giữ toàn bộ (trừ khi tự sharding) | Tự chia theo token của partition key |
| Khoá chính | Định danh dòng, có index | Quyết định **node nào** giữ dòng và **thứ tự** dòng |
| JOIN, khoá ngoại | Có | Không có |
| Transaction | Nhiều dòng, nhiều bảng | Chỉ trong phạm vi hẹp (batch, ghi có điều kiện trên một partition) |
| Thiết kế bảng | Chuẩn hoá theo dữ liệu, truy vấn tuỳ ý | Theo **truy vấn**: mỗi truy vấn một bảng (CS-007) |
| Độ nhất quán | Mạnh trên primary | Chọn theo từng câu lệnh |

## 10. Những hiểu nhầm thường gặp

- **"Cassandra là NoSQL nên không có schema."** Cassandra có schema chặt: bảng, cột, kiểu, khoá chính đều phải khai báo trước.
- **"CQL giống SQL nên viết như SQL là chạy."** Câu `SELECT … WHERE status = 'cancelled'` hợp lệ trong SQL sẽ bị Cassandra từ chối nếu `status` không thuộc khoá (CS-006).
- **"Thêm node là truy vấn nào cũng nhanh hơn."** Thêm node tăng sức chứa và sức ghi tổng; một truy vấn sai thiết kế (quét toàn bảng) vẫn chậm, thậm chí chậm hơn.
- **"Cassandra thay được PostgreSQL."** Không: Cassandra thiếu transaction nhiều bảng và ràng buộc. Tới Nơi dùng cả hai, mỗi database một trách nhiệm.

## 11. Bản đồ series

| Phần | Chủ đề | Bạn sẽ làm được |
|---|---|---|
| P01 | Nền tảng và mô hình hoá theo truy vấn | Mô hình lịch sử chuyến và dòng thời gian sự kiện |
| P02 | CQL cho nghiệp vụ | Chat tài xế–khách, hộp thư, bộ đếm, khoá idempotency |
| P03 | Kiến trúc phân tán | Cụm 3 node, replication, consistency, diễn tập node chết |
| P04 | Hiệu năng, nhất quán và đồng thời | Vị trí tài xế, tranh chấp nhận chuyến bằng lightweight transaction |
| P05 | Bảo mật, bảo trì và phục hồi | Xác thực, repair, snapshot và restore, thay node |
| P06 | Mở rộng, tích hợp và vận hành | Nhiều datacenter, CDC, nâng cấp, di chuyển dữ liệu |

## 12. Bài tập

**Bài 1.** Sắp xếp từ lớn tới nhỏ và nêu tên cụ thể trong lab: dòng, keyspace, cluster, ô, partition, bảng, datacenter.

<details><summary><b>Lời giải</b></summary><p>Cluster (<code>ToiNoi Lab</code>) → datacenter (<code>dc1</code>) → keyspace (<code>toinoi</code>) → bảng (<code>trips_by_customer</code>) → partition (mọi chuyến của khách 1) → dòng (chuyến 11 lúc 17:45 ngày 03/11) → ô (cước của chuyến 11). Node và rack là lớp vật lý, nằm ngang hàng với datacenter chứ không chứa keyspace: một keyspace trải trên mọi node của cluster.</p></details>

**Bài 2.** Chọn nơi lưu (Cassandra hay PostgreSQL) và giải thích một câu: (a) lần nạp 200.000 đ vào ví của khách; (b) tin nhắn "Em đến cổng rồi ạ" của tài xế; (c) điểm GPS của tài xế lúc 18:00:04; (d) số điện thoại đăng ký của khách, phải không trùng.

<details><summary><b>Lời giải</b></summary><p>(a) PostgreSQL: là tiền, cần transaction cộng số dư và ghi sổ cùng lúc. (b) Cassandra: chèn nhiều, đọc theo cuộc trò chuyện và thời gian. (c) Cassandra: tải ghi lớn nhất hệ thống, chỉ chèn, tự hết hạn. (d) PostgreSQL: cần ràng buộc UNIQUE trên toàn bảng; Cassandra không có ràng buộc duy nhất ngoài khoá chính.</p></details>

**Bài 3.** Node nhận câu `INSERT` từ ứng dụng không giữ bản sao nào của partition đó. Câu lệnh có bị từ chối không?

<details><summary><b>Lời giải</b></summary><p>Không. Node đó làm coordinator: tính token của partition key, chuyển câu ghi tới các node giữ bản sao, chờ đủ số bản sao theo consistency level rồi trả lời ứng dụng. Mọi node đều nhận được mọi câu lệnh.</p></details>

**Bài 4.** Một đồng nghiệp đề xuất ghi điểm GPS với consistency level `ALL` "cho chắc chắn". Với 3 bản sao, chuyện gì xảy ra khi một node đang khởi động lại?

<details><summary><b>Lời giải</b></summary><p>Mọi câu ghi vị trí có bản sao trên node đó thất bại, vì coordinator không chờ đủ 3 bản sao. Với dữ liệu mất vài điểm không sai nghiệp vụ, <code>ONE</code> hoặc <code>LOCAL_ONE</code> hợp hơn: luôn ghi được khi còn ít nhất một bản sao sống.</p></details>

## 13. Đọc tiếp

- [Apache Cassandra 5.0: Overview](https://cassandra.apache.org/doc/5.0/cassandra/architecture/overview.html) — tổng quan kiến trúc.
- [Dynamo](https://cassandra.apache.org/doc/5.0/cassandra/architecture/dynamo.html) — phân tán theo token, nhân bản, consistency level.
- [What's new in 5.0](https://cassandra.apache.org/doc/5.0/cassandra/new/index.html) — danh sách thay đổi chính thức.
- [Dự án mẫu Tới Nơi](/articles/du-an-mau-toi-noi): số liệu tải và các bảng chính.
- [PG-001](/articles/postgresql-la-gi-va-vai-tro-trong-toi-noi): nơi giữ bản gốc của chuyến và tiền.

Bài tiếp theo: [CS-002 · Dựng Cassandra 5.0 bằng Docker và kết nối bằng DBStudio](/articles/cassandra-cai-dat-docker-dbstudio).
