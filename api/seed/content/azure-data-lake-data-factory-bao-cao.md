---
slug: azure-data-lake-data-factory-bao-cao
title: "Data Lake, Data Factory và Synapse: báo cáo thu nhập tài xế"
summary: "Bài 38: tách báo cáo thu nhập tài xế của Tới Nơi khỏi PostgreSQL bằng Data Lake Storage Gen2, pipeline Data Factory copy bảng trips sang Parquet theo ngày và truy vấn bằng Synapse serverless OPENROWSET."
category: devops-azure
level: advanced
tags: [azure, cloud, devops, toi-noi, data-lake, data-factory, synapse, analytics]
language: vi
---

Đây là **bài 38** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 4 · Dữ liệu và tích hợp (bài 9/10 của phần).

Bài này đưa báo cáo thu nhập tài xế ra khỏi database giao dịch: copy `trips` mỗi ngày sang Parquet trong Data Lake Storage Gen2, rồi truy vấn bằng Synapse serverless. Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Đã có read replica của PostgreSQL rồi thì cứ cho bộ phận báo cáo query thẳng vào replica, cần gì data lake?</p>
<details><summary>Xem đáp án</summary><p>Replica gỡ tải đọc khỏi primary nhưng không đổi bản chất query: vẫn quét hàng chục triệu dòng trên engine thiết kế cho giao dịch, và query dài vẫn có thể xung đột với replay WAL. Replica hợp làm nguồn để copy đi, ít hợp làm nơi chạy báo cáo nặng thường xuyên.</p></details>
</div>

## Báo cáo 30 ngày không nên chạy trên database đang nhận chuyến

Bảng `trips` của Tới Nơi ghi nhiều và sống nhờ truy vấn ngắn theo khoá. Bộ phận vận hành lại muốn thu nhập của từng tài xế trong 30 ngày, chia theo thành phố, so với tháng trước: query quét phần lớn bảng, gom nhóm, sắp xếp, và hay được mở đúng giờ cao điểm.

Chạy trên primary là cách nhanh nhất để làm chậm việc đặt xe: query tranh CPU, bộ nhớ đệm và I/O với giao dịch thật. Read replica (bài 30) đỡ hơn, nhưng query dài có thể bị huỷ khi xung đột với WAL cần replay, và mỗi báo cáo lại tính từ đầu trên dữ liệu thô.

Hướng đi của bài là **tách OLTP khỏi analytics**: mỗi đêm một job copy dữ liệu hôm qua sang Parquet (định dạng cột) trong data lake, báo cáo đọc từ file. Cái giá là độ trễ một ngày. Với đối soát thu nhập theo ngày thì chấp nhận được; dashboard theo phút cần cơ chế khác.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Luồng báo cáo: PostgreSQL replica, Data Factory copy, Data Lake vùng curated, Synapse serverless."><defs><marker id="az38-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="34" y="95" class="t-b">PostgreSQL replica</text><text x="34" y="123" class="t-small">Nguồn đọc, không phải primary</text><path d="M210 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az38-0-a)"/><rect x="240" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="254" y="95" class="t-b">Data Factory</text><text x="254" y="123" class="t-small">Copy mỗi đêm, có retry</text><path d="M430 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az38-0-a)"/><rect x="460" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="474" y="95" class="t-b">ADLS Gen2 curated</text><text x="474" y="123" class="t-small">Parquet theo dt=YYYY-MM-DD</text><path d="M650 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az38-0-a)"/><rect x="680" y="65" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="694" y="95" class="t-b">Synapse serverless</text><text x="694" y="123" class="t-small">Tính tiền theo dữ liệu quét</text></svg>
</div>
<figcaption>Hình 1. Luồng báo cáo: PostgreSQL replica, Data Factory copy, Data Lake vùng curated, Synapse serverless.</figcaption>
</figure>

## Data Lake Storage Gen2 là Blob Storage có thư mục thật

Data Lake Storage Gen2 không phải dịch vụ riêng mà là storage account ([bài 8](/articles/azure-storage-account-blob-tier-sas)) bật **hierarchical namespace** (cờ `--enable-hierarchical-namespace`). Blob thường có không gian phẳng: `curated/trips/dt=2026-11-05/part-0.parquet` là một cái tên dài, dấu `/` chỉ là quy ước. Bật hierarchical namespace thì thư mục là đối tượng thật, kéo theo hai hậu quả:

- Đổi tên hoặc xoá cả thư mục là một thao tác metadata, không phải copy rồi xoá từng blob.
- Có ACL kiểu POSIX trên từng thư mục và file, ngoài RBAC của Azure. Bạn có thể cho nhóm phân tích đọc `curated` nhưng không đọc thư mục chứa thông tin cá nhân.

Một số tính năng Blob chưa hỗ trợ đầy đủ trên account loại này; kiểm bảng tính năng trước khi bật, vì bật rồi không tắt lại được.

Tới Nơi chia lake thành ba container:

| Vùng | Nội dung | Ai đọc |
| --- | --- | --- |
| `raw` | Dữ liệu nguyên trạng: file Capture, bản dump chưa làm sạch | Pipeline xử lý và kỹ sư dữ liệu |
| `curated` | Đã làm sạch, định dạng Parquet, chia thư mục theo ngày | Nhóm phân tích, Synapse |
| `serving` | Bảng dựng sẵn cho báo cáo, đã bỏ hoặc băm thông tin cá nhân | Công cụ BI, bộ phận vận hành |

Tách vùng để kiểm soát quyền và vòng đời: `raw` quyền hẹp nhất, `serving` nhiều người đọc nhất nên không chứa dữ liệu họ không cần.

## Data Factory copy mỗi ngày một partition, chạy bằng managed identity

Azure Data Factory chủ yếu điều phối việc di chuyển dữ liệu. Bốn khái niệm cần nắm:

- **Pipeline** là chuỗi activity; pipeline của Tới Nơi ban đầu chỉ có một activity **copy**. **Linked service** là thông tin kết nối tới nguồn hoặc đích, **dataset** mô tả hình dạng dữ liệu trên kết nối đó.
- **Integration runtime** là hạ tầng chạy copy. Azure integration runtime chỉ tới được nguồn có đường mạng. PostgreSQL chỉ mở trong mạng riêng ([bài 17](/articles/azure-private-endpoint-private-dns-service-endpoint)) thì cần managed virtual network kèm managed private endpoint, hoặc self-hosted integration runtime đặt trong mạng đó; hỗ trợ khác nhau theo connector.
- **Trigger** chạy pipeline theo lịch. Schedule trigger đơn giản; tumbling window có thêm khoảng thời gian, phụ thuộc và chạy bù, hợp khi cần nạp lại ngày bị lỡ.

Factory có system-assigned managed identity ([bài 13](/articles/azure-managed-identity-workload-identity)); role `Storage Blob Data Contributor` trên storage account đủ để ghi lake, không cần key hay SAS. Phía PostgreSQL, hỗ trợ xác thực Entra của connector tuỳ phiên bản. Nếu chưa dùng được, tạo database role chỉ `SELECT` trên cột cần, lưu mật khẩu trong Key Vault để linked service lấy qua managed identity.

Khối dưới là thân pipeline rút gọn: một copy activity đọc các chuyến hoàn tất trong một ngày, ghi Parquet vào dataset tham số hoá theo `dt`, retry hai lần. Linked service và dataset được giả định đã tạo; khối này không chứng minh kết nối hay quyền ghi chạy được.

```json
{
  "parameters": { "dt": { "type": "String" } },
  "activities": [{
    "name": "CopyTripsToParquet",
    "type": "Copy",
    "policy": { "retry": 2, "retryIntervalInSeconds": 120, "timeout": "0.02:00:00" },
    "inputs": [{ "referenceName": "ds_pg_replica_trips", "type": "DatasetReference" }],
    "outputs": [{
      "referenceName": "ds_curated_trips", "type": "DatasetReference",
      "parameters": { "dt": "@pipeline().parameters.dt" }
    }],
    "typeProperties": {
      "source": {
        "type": "PostgreSqlV2Source",
        "query": {
          "type": "Expression",
          "value": "SELECT trip_id, driver_id, city_id, fare_amount, driver_payout, distance_km, completed_at FROM trips WHERE status = 'completed' AND completed_at >= timestamptz '@{pipeline().parameters.dt} 00:00+07' AND completed_at < timestamptz '@{formatDateTime(addDays(pipeline().parameters.dt, 1), 'yyyy-MM-dd')} 00:00+07'"
        }
      },
      "sink": {
        "type": "ParquetSink",
        "storeSettings": { "type": "AzureBlobFSWriteSettings" },
        "formatSettings": { "type": "ParquetWriteSettings" }
      }
    }
  }]
}
```

Ba chi tiết đáng đọc kỹ. Query liệt kê cột, không `SELECT *`, nên cột tên hay số điện thoại không rời database. Khoảng thời gian viết theo giờ Việt Nam (`+07`) còn `completed_at` lưu UTC; sai múi giờ làm mọi con số lệch mà không báo lỗi. Cuối cùng, `retry` chỉ an toàn khi pipeline **idempotent** theo ngày. Nguồn là database mà dataset đích không đặt tên file thì mỗi lần chạy sinh tên file mới theo run ID, chạy lại sẽ nhân đôi dữ liệu. Hãy đặt tên file cố định trong `ds_curated_trips` hoặc thêm activity Delete xoá thư mục `dt=` trước khi copy.

## Mô hình báo cáo: một bảng fact, hai dimension, và dữ liệu cá nhân ở lại vùng hẹp

Parquet trong `curated` mới là bản sao của bảng nguồn. Để báo cáo dễ hỏi, Tới Nơi dựng mô hình sao trong `serving`:

| Bảng | Mỗi dòng là | Cột chính |
| --- | --- | --- |
| `fact_trips` | Một chuyến hoàn tất | `trip_id`, `driver_key`, `city_key`, `date_key`, `fare_amount`, `driver_payout`, `distance_km` |
| `dim_driver` | Một tài xế | `driver_key`, loại xe, ngày bắt đầu hoạt động, nhóm hạng |
| `dim_city` | Một thành phố | `city_key`, tên thành phố, vùng |

Thu nhập tài xế theo ngày là tổng `driver_payout` của các chuyến hoàn tất, nhóm theo `driver_key`. Không dùng `fare_amount` khách trả, vì chênh lệch là phí nền tảng.

Họ tên, số điện thoại, biển số và toạ độ đón chính xác đều nhạy cảm, và data lake không có masking tự động để bật. Che dữ liệu phải làm ở pipeline:

- Chỉ chọn cột cần thiết ngay ở câu SELECT nguồn, như khối trên.
- Thay `driver_id` bằng `driver_key` được băm có muối, muối lưu trong Key Vault. Cùng một tài xế luôn ra cùng một khoá để nối bảng, nhưng người đọc báo cáo không suy ngược ra hồ sơ.
- Nếu thật sự cần bảng ánh xạ ngược, giữ trong thư mục riêng với ACL chỉ cho một nhóm rất nhỏ.

Băm có muối là giả danh hoá, không phải ẩn danh: ai có cả muối lẫn danh sách id vẫn dò ngược được.

## Capture của Event Hubs cho km thực tế, nhưng cần một bước đổi định dạng

Cột `distance_km` trong `trips` thường là quãng đường ước tính lúc đặt; km chạy thật cần chuỗi toạ độ GPS. Ở [bài 35](/articles/azure-event-hubs-stream-gps-consumer-group), luồng đó đi qua Event Hubs và bật Capture. Trỏ Capture vào container `raw` thì GPS tới lake mà không phải viết thêm consumer.

Capture ghi Avro, thư mục theo namespace, event hub, partition và thời điểm, không theo `dt=`. Synapse serverless không đọc Avro, nên cần một bước đổi sang Parquet theo ngày trước khi vào `curated` (hoặc ghi Parquet từ đầu bằng Stream Analytics no-code editor). Tính km thực tế cũng là bài toán dữ liệu: điểm GPS chỉ mang `driverId`, nên phải ghép vào chuyến theo tài xế và khoảng thời gian chuyến, cộng khoảng cách giữa các điểm liên tiếp và lọc điểm nhảy do nhiễu. Kết quả là cột `distance_km_gps` trong `fact_trips`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Ba vùng của lake: raw nhận Capture và dump, curated giữ Parquet theo ngày, serving giữ bảng báo cáo đã bỏ thông tin cá nhân, kèm lifecycle chuyển tầng."><defs><marker id="az38-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">raw</text><text x="34" y="123" class="t-small">Avro từ Capture, dump</text><path d="M210 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az38-1-a)"/><rect x="240" y="65" width="190" height="85" rx="10" class="f-phys-soft s-phys"/><text x="254" y="95" class="t-b">curated</text><text x="254" y="123" class="t-small">Parquet, dt= theo ngày</text><path d="M430 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az38-1-a)"/><rect x="460" y="65" width="190" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">serving</text><text x="474" y="123" class="t-small">fact, dim, đã bỏ PII</text><path d="M650 108 L680 108" class="f-none s-warn" stroke-width="2" marker-end="url(#az38-1-a)"/><rect x="680" y="65" width="190" height="85" rx="10" class="f-warn-soft s-warn"/><text x="694" y="95" class="t-b">Lifecycle</text><text x="694" y="123" class="t-small">Cool, Archive, xoá</text></svg>
</div>
<figcaption>Hình 2. Ba vùng của lake: raw nhận Capture và dump, curated giữ Parquet theo ngày, serving giữ bảng báo cáo đã bỏ thông tin cá nhân, kèm lifecycle chuyển tầng.</figcaption>
</figure>

## Synapse serverless tính tiền theo dữ liệu quét, nên chia thư mục là tối ưu chi phí

**Synapse serverless SQL pool** không có máy chủ để quản lý: nó đọc file trực tiếp trong storage và tính tiền theo dữ liệu được xử lý. `OPENROWSET` coi một đường dẫn (kèm wildcard) như một bảng. Microsoft Fabric là hướng mới cho cùng bài toán, gom lake, SQL và báo cáo vào một nền tảng; bài này chỉ nhắc tên.

Chi phí của luồng báo cáo gồm ba khoản, mỗi khoản có một cách làm nhỏ lại:

| Khoản | Tính theo | Cách giảm |
| --- | --- | --- |
| Copy trong Data Factory | DIU-giờ (data integration unit) của copy và số lần chạy activity | Copy theo ngày thay vì copy lại cả bảng; không đặt lịch dày hơn nhu cầu |
| Lưu trữ trong lake | GB-tháng theo tầng nóng, mát, lạnh, archive | Lifecycle policy chuyển `raw` và `curated` cũ sang tầng rẻ hơn |
| Query serverless | Dữ liệu được xử lý (theo TB) | Parquet, chọn đúng cột, lọc theo thư mục `dt=` |

Parquet lưu theo cột nên query chọn ba cột không đọc các cột còn lại, và thư mục `dt=YYYY-MM-DD` cho phép bỏ qua cả ngày không cần. Cùng dữ liệu lưu CSV và quét hết mỗi lần thì trả nhiều hơn mà không nhanh hơn.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Mỗi query serverless là một khoản tiền. Dashboard tự làm mới mỗi phút trên toàn bộ <code>curated/trips</code> không lọc ngày sẽ quét lại cả năm dữ liệu hàng nghìn lần mỗi ngày. Bắt buộc điều kiện theo thư mục, đặt budget alert ([bài 5](/articles/azure-chi-phi-pricing-cost-management-budget)), và cho công cụ BI đọc <code>serving</code> đã tổng hợp.</p>
</div>

Lifecycle management là policy JSON gắn vào storage account, chẳng hạn chuyển blob trong `raw/` sang Cool sau một số ngày kể từ lần sửa cuối, rồi sang Archive. Archive lấy lại phải rehydrate, mất hàng giờ và có phí, nên đừng đẩy xuống đó dữ liệu còn query hằng tuần.

## Lab: từ trips hôm qua tới tổng thu nhập theo tài xế

Lab mô tả cách làm; bạn cần quyền tạo resource và một PostgreSQL có bảng `trips`. Khối đầu tạo storage account bật hierarchical namespace và ba container; `--auth-mode login` đòi bạn có role ghi blob trên account:

```bash
RG=rg-toinoi-data-dev-sea
ST=sttoinoidatadev

az group create -n $RG -l southeastasia

az storage account create -n $ST -g $RG -l southeastasia \
  --sku Standard_LRS --kind StorageV2 \
  --enable-hierarchical-namespace true \
  --allow-blob-public-access false --min-tls-version TLS1_2

for c in raw curated serving; do
  az storage container create --account-name $ST -n $c --auth-mode login
done
```

Khối thứ hai tạo factory, cấp quyền ghi lake cho managed identity của nó, nạp pipeline từ `pipeline.json` (khối JSON ở trên) và chạy thử một ngày. `az datafactory` là extension, cú pháp có thể đổi; kiểm bằng `-h`. Linked service và dataset cần tạo trước, bằng portal hoặc `az datafactory linked-service create` và `dataset create`:

```bash
ADF=adf-toinoi-dev-sea
az extension add --name datafactory
az datafactory create -g $RG -n $ADF -l southeastasia

PID=$(az datafactory show -g $RG -n $ADF --query identity.principalId -o tsv)
az role assignment create --assignee-object-id $PID \
  --assignee-principal-type ServicePrincipal \
  --role "Storage Blob Data Contributor" \
  --scope $(az storage account show -g $RG -n $ST --query id -o tsv)

az datafactory pipeline create -g $RG --factory-name $ADF \
  -n pl-toinoi-trips-daily --pipeline @pipeline.json
az datafactory pipeline create-run -g $RG --factory-name $ADF \
  -n pl-toinoi-trips-daily --parameters '{"dt":"2026-11-05"}'
```

Nếu `principalId` trả về rỗng, factory chưa có system-assigned identity: bật ở mục Managed identities trong portal rồi gán role lại. Run thành công thì kỳ vọng có file `.parquet` trong `curated/trips/dt=2026-11-05/`. Chạy mỗi đêm thì tạo schedule trigger truyền `dt` là ngày hôm qua theo giờ Việt Nam.

Cuối cùng tạo Synapse workspace dùng chính storage account này, mật khẩu SQL lấy từ biến môi trường để khỏi nằm trong lịch sử shell:

```bash
az synapse workspace create -n syn-toinoi-dev-sea -g $RG -l southeastasia \
  --storage-account $ST --file-system serving \
  --sql-admin-login-user sqladmin \
  --sql-admin-login-password "$SYNAPSE_SQL_PASSWORD"
```

Để serverless đọc file, danh tính chạy query (tài khoản Entra của bạn khi dùng passthrough) cần role `Storage Blob Data Reader` trên account; thiếu role này là lỗi thường gặp nhất. Workspace cũng chặn kết nối từ ngoài tới khi bạn thêm firewall rule cho IP của mình (`az synapse workspace firewall-rule create`). Truy vấn tổng thu nhập theo tài xế:

```sql
SELECT r.driver_id,
       COUNT(*)            AS so_chuyen,
       SUM(r.driver_payout) AS thu_nhap
FROM OPENROWSET(
       BULK 'https://sttoinoidatadev.dfs.core.windows.net/curated/trips/dt=2026-11-05/*.parquet',
       FORMAT = 'PARQUET'
     ) AS r
GROUP BY r.driver_id
ORDER BY thu_nhap DESC;
```

Câu SQL chứng minh file đọc được, chưa chứng minh số liệu khớp nguồn: hãy đối chiếu tổng `driver_payout` cùng ngày với một query trên replica. Xong thì dọn bằng `az group delete -n $RG`.

**Bài tập:** pipeline đêm chạy xong lúc 02:00 nhưng một chuyến hoàn tất lúc 23:58 lại không có trong `dt=` của ngày đó. Nêu hai nguyên nhân có thể và cách kiểm.

<details><summary>Xem lời giải</summary><p>Thứ nhất là múi giờ: query lọc theo UTC trong khi ngày báo cáo tính theo giờ Việt Nam, nên chuyến rơi sang ngày khác. Thứ hai là trạng thái: lúc copy chuyến chưa ở <code>completed</code>, hoặc <code>completed_at</code> được ghi muộn hơn. Kiểm bằng cách so <code>completed_at</code> với hai biên của query rồi chạy lại ngày đó; chạy lại an toàn khi đích ghi đè đúng một thư mục <code>dt=</code>.</p></details>

## Học tiếp từ đây

Bạn vừa tách báo cáo ra khỏi database giao dịch: một lake ba vùng, pipeline copy `trips` sang Parquet theo ngày bằng managed identity, và một query serverless tính thu nhập theo tài xế.

Bài cuối của Phần 4 là [bài 39](/articles/azure-api-management-gateway-policy-versioning): API Management làm cổng cho đối tác của Tới Nơi, với policy, versioning và product. Luồng sự kiện đổ vào lake xem lại ở [bài 37](/articles/azure-chon-messaging-storage-queue-service-bus-event-hubs).

## Tài liệu gốc

- [Azure Data Lake Storage Gen2 introduction](https://learn.microsoft.com/en-us/azure/storage/blobs/data-lake-storage-introduction)
- [Azure Data Factory documentation](https://learn.microsoft.com/en-us/azure/data-factory/)
- [Copy activity trong Azure Data Factory](https://learn.microsoft.com/en-us/azure/data-factory/copy-activity-overview)
- [Synapse serverless SQL pool overview](https://learn.microsoft.com/en-us/azure/synapse-analytics/sql/on-demand-workspace-overview)
- [Quản lý vòng đời Blob Storage](https://learn.microsoft.com/en-us/azure/storage/blobs/lifecycle-management-overview)
- [Microsoft Fabric documentation](https://learn.microsoft.com/en-us/fabric/)
