---
slug: aws-s3-nang-cao-storage-class-replication-object-lock
title: "S3 nâng cao: storage class, replication, Object Lock, event notification và S3 Tables"
summary: "Bài 36: tối ưu kho 730 triệu chứng từ và log mỗi năm của Tới Nơi: chọn storage class theo mẫu truy cập, Intelligent-Tiering, Cross-Region và Same-Region Replication, Object Lock cho hoá đơn giữ 10 năm, event notification tới EventBridge, Batch Operations, Access Point và S3 Tables cho Iceberg."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, s3, storage-class, replication, object-lock, s3-tables, event-notification]
language: vi
---

Sau một năm, Tới Nơi có khoảng 730 triệu chuyến, mỗi chuyến để lại một hoá đơn, vài dòng log và (với đơn giao hàng) một tấm ảnh chứng từ. Kế toán yêu cầu giữ hoá đơn 10 năm và không ai được sửa hay xoá. Hạ tầng nhìn hoá đơn S3 thì thấy phần lớn tiền trả để giữ ảnh của chuyến đã giao từ nhiều tháng trước, hầu như không ai mở lại. Bài này giải quyết hai việc ngược nhau: giữ rẻ hơn và giữ chắc hơn.

Đây là **bài 36** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần đã đọc bài về bucket, versioning và lifecycle cơ bản ([bài 06](/articles/aws-s3-bucket-versioning-presigned-url)) và đã làm quen với IAM role ([bài 03](/articles/aws-iam-user-role-policy)). Con số dung lượng là giả định: ảnh 1 MB cho khoảng 1,2 triệu chuyến mỗi ngày (như bài 06), hoá đơn PDF khoảng 50 KB mỗi chuyến.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bucket hoá đơn đã bật Object Lock chế độ COMPLIANCE với thời hạn giữ 10 năm. Một quản trị viên có quyền <code>s3:*</code> trên bucket có xoá được một hoá đơn trước hạn không? Còn tài khoản root?</p>
<details><summary>Xem đáp án</summary><p><b>Không cho cả hai.</b> Ở chế độ COMPLIANCE, không người dùng nào, kể cả root, rút ngắn được retention hay xoá version đó cho tới ngày hết hạn. Đó chính là điểm khác với GOVERNANCE, nơi người có quyền <code>s3:BypassGovernanceRetention</code> vẫn bỏ qua được.</p></details>
</div>

## Chọn storage class theo mẫu truy cập

Mỗi object S3 nằm trong một **storage class**, khác nhau ở giá lưu, giá lấy dữ liệu ra và thời gian chờ. Lớp càng rẻ để lưu thì càng đắt hoặc càng chậm để đọc, và thường có thời gian lưu tối thiểu.

| Storage class | Hợp với | Lấy dữ liệu | Điều cần để ý |
| --- | --- | --- | --- |
| Standard | Dữ liệu nóng | Mili giây | Không phí lấy dữ liệu |
| Standard-IA | Hiếm đọc nhưng cần ngay | Mili giây | Tối thiểu 30 ngày; có phí lấy; object nhỏ tính như 128 KB |
| One Zone-IA | Bản tái tạo được | Mili giây | Chỉ 1 AZ: mất AZ là mất dữ liệu |
| Intelligent-Tiering | Mẫu truy cập khó đoán | Mili giây (ba tầng đầu) | Tự chuyển tầng; phí giám sát theo số object; không phí lấy |
| Glacier Instant Retrieval | Lưu lâu, đọc vài lần mỗi quý | Mili giây | Tối thiểu 90 ngày |
| Glacier Flexible Retrieval | Lưu lâu, đọc hiếm | Phút tới giờ | Tối thiểu 90 ngày; phải `restore` trước khi đọc |
| Glacier Deep Archive | Lưu pháp lý | Hàng chục giờ | Tối thiểu 180 ngày; rẻ nhất để lưu |
| Express One Zone | Request cực nhiều, trễ thấp | Mili giây | Directory bucket, một AZ; hợp job phân tích |

Áp vào Tới Nơi: ảnh chứng từ (1 MB) đi Standard 30 ngày, rồi IA, rồi Glacier Instant Retrieval để hỗ trợ viên vẫn mở được trong mili giây khi khiếu nại muộn. Log đi Intelligent-Tiering vì không biết ngày nào cần đọc lại. Hoá đơn PDF 50 KB là ca khó vì nhỏ hơn 128 KB, IA không có lợi: hãy gộp thành tệp lớn hơn trước khi xuống lớp lạnh.

## Lifecycle với filter và Intelligent-Tiering

Lifecycle rule lọc theo prefix, tag và kích thước. Rule ảnh chứng từ của lab chỉ chuyển object từ 128 KB ở prefix `proof/`:

```hcl
rule {
  id     = "proof-photos-tiering"
  status = "Enabled"

  filter {
    and {
      prefix                   = "proof/"
      object_size_greater_than = 131072
    }
  }

  transition {
    days          = 30
    storage_class = "STANDARD_IA"
  }

  transition {
    days          = 90
    storage_class = "GLACIER_IR"
  }
}
```

Từ 09/2024, lifecycle mặc định bỏ qua object dưới 128 KB; filter trong lab chỉ viết rõ ý đó. Mỗi lần chuyển lớp là một request tính phí, với hàng triệu object nhỏ có thể lớn hơn tiền lưu tiết kiệm được. Lifecycle chạy bất đồng bộ nên ngày chuyển không chính xác từng giờ.

Với log, lab dùng `INTELLIGENT_TIERING` từ ngày 0. Ba tầng đầu (Frequent, Infrequent sau 30 ngày, Archive Instant sau 90 ngày) chạy tự động; hai tầng Archive Access và Deep Archive Access là tuỳ chọn, chỉ bật khi chấp nhận truy xuất chậm. Object nhỏ hơn 128 KB không được chuyển tầng.

## Replication: bản sao ở region khác cho hoá đơn

**S3 Replication** nhân bản object từ bucket nguồn sang bucket đích, bất đồng bộ. **Cross-Region Replication (CRR)** sang region khác, dùng cho DR; **Same-Region Replication (SRR)** trong cùng region, để gom log về tài khoản bảo mật hoặc tách bản cho môi trường thử. Cả hai cần **versioning ở hai bucket** và một **IAM role** cho S3 assume (principal `s3.amazonaws.com`) với quyền đọc nguồn, ghi đích.

Vài hành vi hay gây bất ngờ:

- **Chỉ object ghi sau khi bật rule** mới được nhân bản. Object cũ cần **S3 Batch Replication**.
- **Không nhân bản dây chuyền**: bản sao không tự nhân bản tiếp sang bucket thứ ba.
- Xoá một version cụ thể ở nguồn **không** lan sang đích, cố ý để kẻ xấu không xoá được cả hai nơi. Delete marker có lan hay không tuỳ `delete_marker_replication`.
- **Replication Time Control (RTC)** cam kết 99,99% object tới nơi trong 15 phút (có SLA) kèm metric, tính phí riêng; **replica modification sync** đồng bộ thay đổi metadata trên bản sao, chỉ có tác dụng khi cấu hình replication hai chiều.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 290" role="img" aria-label="Hình 1. Hoá đơn được ghi vào bucket nguồn ở ap-southeast-1 có Object Lock, rồi nhân bản bất đồng bộ sang bucket có Object Lock ở ap-northeast-1; xoá version cụ thể ở nguồn không lan sang bản sao."><defs><marker id="aws36-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws36-1-phys" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="20" width="170" height="60" rx="10" class="f-phys-soft s-phys"/><text x="105" y="48" text-anchor="middle" class="t-b">Service billing</text><text x="105" y="66" text-anchor="middle" class="t-small">PUT invoices/...</text><rect x="20" y="120" width="320" height="100" rx="10" class="f-pk-soft s-pk"/><text x="36" y="146" class="t-b">Bucket nguồn · ap-southeast-1</text><text x="36" y="168" class="t-small t-mono">toinoi-lab-invoices-&lt;account&gt;</text><text x="36" y="188" class="t-small">Versioning + Object Lock (retention mặc định)</text><text x="36" y="206" class="t-small">Prefix invoices/ được nhân bản</text><rect x="540" y="120" width="320" height="100" rx="10" class="f-pk-soft s-pk"/><text x="556" y="146" class="t-b">Bucket bản sao · ap-northeast-1</text><text x="556" y="168" class="t-small t-mono">toinoi-lab-invoices-replica-&lt;account&gt;</text><text x="556" y="188" class="t-small">Versioning + Object Lock</text><text x="556" y="206" class="t-small">Lifecycle: Glacier IR sau 30 ngày</text><path d="M105 80 V118" class="f-none s-phys" stroke-width="2" marker-end="url(#aws36-1-phys)"/><path d="M340 170 H538" class="f-none s-pk" stroke-width="2" marker-end="url(#aws36-1-pk)"/><text x="439" y="152" text-anchor="middle" class="t-small">replication bất đồng bộ</text><text x="439" y="192" text-anchor="middle" class="t-small">role tin cậy s3.amazonaws.com</text><rect x="20" y="240" width="840" height="38" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="264" class="t-small">Xoá một version cụ thể ở nguồn không được nhân bản; Object Lock còn chặn xoá trong thời hạn retention ở cả hai bucket.</text></svg>
</div>
<figcaption>Hình 1. Hai lớp bảo vệ độc lập: bản sao ở region thứ hai chống mất region, Object Lock chống xoá và sửa trong thời hạn giữ.</figcaption>
</figure>

Replication không thay backup: ghi đè nhầm một object thì bản sai cũng được nhân bản. Backup có kiểm chứng nằm ở [bài 37](/articles/aws-backup-disaster-recovery-rpo-rto).

## Object Lock: giữ chứng từ không ai sửa được

**Object Lock** cho mô hình WORM (write once, read many). Nó khoá từng **version** chứ không phải key: ghi đè key tạo version mới, version cũ vẫn bị khoá. Bucket phải bật versioning. Từ cuối 2023 có thể bật Object Lock cho bucket đang có (đã bật versioning), nhưng bật rồi thì không tắt được nữa; lab bật ngay lúc tạo bucket.

| Cơ chế | Ai bỏ qua được | Dùng khi |
| --- | --- | --- |
| Retention GOVERNANCE | Principal có `s3:BypassGovernanceRetention` và gửi header bypass | Chống nhầm lẫn, vẫn có lối thoát |
| Retention COMPLIANCE | Không ai, kể cả root, tới ngày hết hạn | Yêu cầu WORM thực sự |
| Legal hold | Principal có `s3:PutObjectLegalHold` | Khoá không thời hạn tới khi vụ việc kết thúc |

Retention đặt trên từng object hoặc làm **mặc định** cho bucket (lab dùng `default_retention`). Legal hold độc lập với retention: hết hạn retention mà còn legal hold thì vẫn không xoá được.

```bash
aws s3api put-object --bucket toinoi-lab-invoices-123456789012 \
  --key invoices/2026/11/inv-100123.pdf --body inv-100123.pdf \
  --object-lock-mode GOVERNANCE --object-lock-retain-until-date 2026-11-08T00:00:00Z
```

Lệnh ghi một hoá đơn và khoá version vừa tạo tới ngày chỉ định. Lab dùng GOVERNANCE và 1 ngày vì ở COMPLIANCE, đặt nhầm 10 năm là trả tiền lưu 10 năm, không đường lui. Với hoá đơn thật, hãy để pháp chế xác nhận chế độ và thời hạn.

Lifecycle không xoá được version đang khoá; replication cần role đọc được retention và legal hold, bucket đích cũng bật Object Lock. Để biết đang có gì, bật **S3 Inventory** (báo cáo định kỳ liệt kê object kèm storage class và trạng thái lock, query bằng Athena) và **S3 Storage Lens** (dashboard theo bucket và prefix), rẻ hơn nhiều so với `LIST` toàn bucket.

## Event notification: EventBridge hay gọi thẳng

Khi có ảnh mới, Tới Nơi muốn kiểm tra chất lượng ảnh và cập nhật chuyến. S3 báo theo hai đường.

| Tiêu chí | Trực tiếp (SQS, SNS, Lambda) | Qua EventBridge |
| --- | --- | --- |
| Cấu hình | Trên bucket | Bật một lần, phần còn lại ở rule |
| Lọc | Prefix và suffix; cùng loại event không được chồng prefix | Pattern theo key, kích thước, tên bucket... |
| Số đích | Một đích mỗi cấu hình | Nhiều rule, mỗi rule nhiều target |

Cả hai giao **ít nhất một lần**, nên consumer phải idempotent. Lab bật `eventbridge = true` và dùng rule sau để đẩy ảnh trong `proof/` vào một queue SQS:

```json
{
  "source": ["aws.s3"],
  "detail-type": ["Object Created"],
  "detail": {
    "bucket": { "name": ["toinoi-lab-delivery-photos-123456789012"] },
    "object": { "key": [{ "prefix": "proof/" }], "size": [{ "numeric": [">", 0] }] }
  }
}
```

Queue cần policy cho principal `events.amazonaws.com` với điều kiện `aws:SourceArn` là rule; thiếu bước này rule khớp nhưng tin nhắn không tới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 280" role="img" aria-label="Hình 2. Hai đường báo event của S3: cấu hình trực tiếp tới một đích SQS, SNS hoặc Lambda, hoặc qua EventBridge nơi rule lọc theo pattern rồi chuyển tới nhiều target."><defs><marker id="aws36-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws36-2-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="90" width="180" height="80" rx="10" class="f-phys-soft s-phys"/><text x="110" y="124" text-anchor="middle" class="t-b">Bucket ảnh</text><text x="110" y="144" text-anchor="middle" class="t-small t-mono">proof/&lt;trip&gt;/...</text><rect x="290" y="30" width="190" height="80" rx="10" class="f-pk-soft s-pk"/><text x="385" y="62" text-anchor="middle" class="t-b">EventBridge</text><text x="385" y="82" text-anchor="middle" class="t-small">rule lọc theo pattern</text><text x="385" y="98" text-anchor="middle" class="t-small">Object Created</text><rect x="600" y="20" width="250" height="44" rx="10" class="f-phys-soft s-phys"/><text x="725" y="48" text-anchor="middle" class="t-b">SQS: kiểm tra chất lượng ảnh</text><rect x="600" y="76" width="250" height="44" rx="10" class="f-phys-soft s-phys"/><text x="725" y="104" text-anchor="middle" class="t-b">Lambda: gắn tag chuyến</text><rect x="600" y="132" width="250" height="44" rx="10" class="f-phys-soft s-phys"/><text x="725" y="160" text-anchor="middle" class="t-b">Target khác thêm sau, không đụng bucket</text><rect x="290" y="190" width="190" height="64" rx="10" class="f-warn-soft s-warn dash"/><text x="385" y="216" text-anchor="middle" class="t-b t-warn">Cấu hình trực tiếp</text><text x="385" y="236" text-anchor="middle" class="t-small">một đích, lọc prefix hoặc suffix</text><path d="M200 120 L288 78" class="f-none s-pk" stroke-width="2" marker-end="url(#aws36-2-pk)"/><path d="M480 56 L598 42" class="f-none s-pk" stroke-width="2" marker-end="url(#aws36-2-pk)"/><path d="M480 72 L598 98" class="f-none s-pk" stroke-width="2" marker-end="url(#aws36-2-pk)"/><path d="M480 88 L598 152" class="f-none s-pk" stroke-width="2" marker-end="url(#aws36-2-pk)"/><path d="M200 140 L288 214" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws36-2-warn)"/><path d="M480 222 L598 222" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws36-2-warn)"/><text x="725" y="226" text-anchor="middle" class="t-small">SQS hoặc SNS hoặc Lambda</text></svg>
</div>
<figcaption>Hình 2. Đường trên (liền) linh hoạt khi cần nhiều consumer; đường dưới (đứt) đủ cho một consumer duy nhất.</figcaption>
</figure>

## Batch Operations, Access Point và S3 Tables

**S3 Batch Operations** chạy một thao tác trên danh sách object từ manifest (thường là báo cáo Inventory): sao chép, gắn tag, đặt retention, gọi Lambda, `restore` từ Glacier hay replicate object cũ. Một job `restore` mở được hàng chục nghìn ảnh khiếu nại đang ở Glacier Flexible; job tính phí theo số object.

**Access Point** là tên truy cập riêng cho một bucket, mỗi cái có policy riêng và có thể giới hạn **chỉ từ một VPC**. Thay một bucket policy liệt kê mọi team, ta tạo access point cho đội hỗ trợ (chỉ đọc `proof/*`) và cho billing (ghi `invoices/*`). Access point không thay IAM: cả hai lớp đều phải cho phép.

**S3 Tables** là loại bucket riêng (**table bucket**) lưu bảng Apache Iceberg, tự compaction và dọn snapshot. Tới Nơi dùng nó cho bảng sự kiện chuyến (`trip_events`, khoảng 16 triệu dòng mỗi ngày) vì cần upsert và time travel mà không tự vận hành compaction. Lab tạo table bucket và namespace `trips`; bảng tạo bằng `aws s3tables create-table --format ICEBERG`. Để Athena query trực tiếp cần bật tích hợp với dịch vụ phân tích trong region, bài 38 sẽ dùng tới.

## Cái giá phải trả

- **Phí chuyển lớp và phí lấy dữ liệu.** Đẩy hàng triệu object nhỏ, hoặc ảnh vẫn bị mở lại, xuống lớp lạnh có thể đắt hơn để nguyên Standard.
- **Replication nhân đôi tiền lưu** và cộng phí truyền giữa region. Chỉ nhân bản prefix cần như `invoices/`.
- **Object Lock COMPLIANCE không có đường lui.** Thử ở GOVERNANCE trước.
- **Event có thể trùng hoặc đến muộn.** Consumer phải idempotent; đừng coi S3 event là nguồn sự thật cho trạng thái chuyến.
- **Bucket có versioning và lock khó xoá:** phải xoá mọi version, delete marker và đợi hết retention.

**Bài tập:** Ảnh chứng từ khiếu nại cần mở được trong vài giây trong 2 năm, nhưng chỉ khoảng 1% ảnh bị mở lại. Glacier Deep Archive có phù hợp không?

<details><summary>Xem đáp án</summary><p>Không. Deep Archive lấy dữ liệu mất hàng giờ và lưu tối thiểu 180 ngày, trong khi hỗ trợ viên cần xem ngay. Glacier Instant Retrieval hợp hơn: lưu rẻ, lấy trong mili giây, phí lấy dữ liệu cao nhưng chấp nhận được vì chỉ khoảng 1% ảnh bị mở. Deep Archive hợp với bản sao pháp lý gần như không đọc, như log đã quá hạn khiếu nại.</p></details>

## Lab: dựng và thử

Thư mục `examples/toi-noi-aws/p04-36-s3-advanced` có Terraform (không có backend thật) dựng bucket ảnh với lifecycle và Intelligent-Tiering, bucket hoá đơn với Object Lock và replication sang `ap-northeast-1`, event qua EventBridge vào SQS, và S3 table bucket. Hai script trong `scripts/` thử Object Lock và in lệnh S3 Tables, Access Point.

```bash
cd examples/toi-noi-aws/p04-36-s3-advanced
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Terraform đã qua `fmt` và `validate` nhưng chưa `apply` trên tài khoản thật; quyền tối thiểu của role replication với bucket có Object Lock cần kiểm khi chạy. Tiền lab đến từ dung lượng ở hai region, request và truyền dữ liệu; bước dọn nằm trong README.

Bài trước: [bài 35](/articles/aws-kinesis-data-streams-firehose-gps). Bài tiếp: [bài 37](/articles/aws-backup-disaster-recovery-rpo-rto).

## Tài liệu gốc

- [Amazon S3 User Guide](https://docs.aws.amazon.com/AmazonS3/latest/userguide/Welcome.html).
- [Storage class của S3](https://docs.aws.amazon.com/AmazonS3/latest/userguide/storage-class-intro.html).
- [Replication trong S3](https://docs.aws.amazon.com/AmazonS3/latest/userguide/replication.html).
- [Khoá object bằng Object Lock](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock.html).
- [Event notification của S3 và EventBridge](https://docs.aws.amazon.com/AmazonS3/latest/userguide/EventBridge.html).
- [Amazon S3 Tables](https://docs.aws.amazon.com/AmazonS3/latest/userguide/s3-tables.html).
