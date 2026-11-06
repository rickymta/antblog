---
slug: aws-s3-bucket-versioning-presigned-url
title: "S3: bucket, versioning, lifecycle và presigned URL cho ảnh chứng từ giao hàng"
summary: "Bài 06: thiết kế bucket ảnh chứng từ giao hàng của Tới Nơi với Block Public Access, mã hoá mặc định, versioning, lifecycle sang lớp lưu trữ rẻ hơn; tài xế upload bằng presigned URL và khách xem bằng URL có hạn."
category: devops-aws
level: beginner
tags: [aws, toi-noi, devops, s3, presigned-url, versioning, lifecycle, object-storage]
language: vi
---

Một tài xế giao bưu kiện ở Đà Nẵng chụp ảnh gói hàng trước cửa nhà khách rồi bấm "Hoàn thành". Tuần sau khách khiếu nại "tôi chưa nhận được hàng", và bộ phận hỗ trợ cần mở đúng tấm ảnh đó trong vài giây. Giả định của bài: chỉ chuyến `food` và `parcel` có ảnh, mỗi chuyến một ảnh khoảng 1 MB, tức 1,2 triệu ảnh, khoảng 1,2 TB mỗi ngày và hơn 400 TB mỗi năm. Đủ lớn để phải nghĩ tới chỗ lưu, chi phí và ai được xem.

Đây là **bài 06** của lộ trình **AWS qua Tới Nơi · Phần 01: Nền tảng AWS**. Bạn cần đã có tài khoản và AWS CLI chạy được ([bài 02](/articles/aws-tai-khoan-iam-identity-center-cli)), hiểu IAM role ở mức cơ bản ([bài 03](/articles/aws-iam-user-role-policy)). Bài này chưa cần VPC hay EC2.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bucket ảnh đã bật Block Public Access. Vậy ứng dụng tài xế có thể upload ảnh bằng một đường link mà không cần credential AWS nào không?</p>
<details><summary>Xem đáp án</summary><p><b>Có.</b> Presigned URL mang sẵn chữ ký của người tạo (ví dụ role của service `delivery-photo`), nên S3 vẫn xác thực được request. Block Public Access chặn quyền truy cập công khai ẩn danh, không chặn request đã ký đúng.</p></details>
</div>

## S3 là object storage, không phải ổ đĩa

Amazon S3 (Simple Storage Service) lưu **object** trong **bucket**. Một object gồm nội dung, metadata và một **key**, chính là tên đầy đủ của nó, ví dụ `proof/0b6f.../delivered.jpg`. Dấu `/` trong key chỉ là ký tự thường: "thư mục" trong console là cách hiển thị theo **prefix** chung. Hệ quả là không có lệnh "đổi tên thư mục" rẻ tiền; đổi prefix nghĩa là copy rồi xoá từng object.

| Điểm | File system (ổ đĩa, EFS) | S3 |
| --- | --- | --- |
| Đơn vị | File trong cây thư mục | Object trong bucket, định danh bằng key |
| Sửa một phần | Ghi đè vài byte được | Thay cả object bằng bản mới |
| Truy cập | Mount, đọc ghi qua hệ điều hành | HTTPS API (PUT, GET, DELETE, LIST) |
| Dung lượng | Cấp trước theo ổ | Không giới hạn tổng; mỗi object tối đa 50 TB (từ 12/2025, trước đó 5 TB) |
| Tính nhất quán | Tuỳ hệ thống | Strong read-after-write: PUT xong là GET và LIST thấy ngay |

Tên bucket phải **duy nhất toàn cầu** trong mọi tài khoản AWS, chỉ gồm chữ thường, số, dấu gạch ngang (và dấu chấm, nên tránh). Vì thế lab dùng `toinoi-lab-delivery-photos-<account-id>` để tránh trùng; nếu dùng lại policy ở bài 03, sửa ARN bucket trong đó cho khớp tên này. Bucket thuộc một region (ở đây `ap-southeast-1`) và dữ liệu không tự rời region đó.

## Cấu hình an toàn ngay từ đầu

Bucket mới tạo từ tháng 4/2023 có ba mặc định tốt: **Block Public Access** bật đủ bốn cờ, **ACL bị tắt** (Object Ownership là bucket owner enforced, mọi object thuộc chủ bucket) và mọi object ghi mới được mã hoá **SSE-S3** (khoá do S3 quản lý). Script vẫn khai báo tường minh để dễ đọc và kiểm.

```bash
aws s3api create-bucket --bucket "$BUCKET" \
  --create-bucket-configuration "LocationConstraint=ap-southeast-1"
aws s3api put-public-access-block --bucket "$BUCKET" \
  --public-access-block-configuration \
  BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true
aws s3api put-bucket-encryption --bucket "$BUCKET" \
  --server-side-encryption-configuration \
  'Rules=[{ApplyServerSideEncryptionByDefault={SSEAlgorithm=AES256}}]'
```

Ngoài `us-east-1`, `create-bucket` bắt buộc có `LocationConstraint`. Bốn cờ Block Public Access ghi đè mọi policy và ACL dại dột sau này: dù ai đó dán một bucket policy `Principal: "*"`, S3 vẫn từ chối. Lớp bảo vệ này không tốn tiền; cái giá là khi thật sự cần public, ta phải tắt có chủ đích.

## Versioning: xoá không còn là xoá

Khi bật **versioning**, mỗi lần PUT vào cùng key tạo một **version** mới có `VersionId` riêng. Lệnh DELETE không kèm version id không xoá gì cả: S3 thêm một **delete marker** làm bản mới nhất, nên GET trả 404 nhưng mọi version cũ vẫn còn và vẫn tính tiền lưu trữ. Chỉ lệnh DELETE chỉ rõ `--version-id` mới xoá vĩnh viễn một bản. Đã bật versioning thì chỉ có thể tạm dừng (suspended), không quay về trạng thái chưa từng bật.

Với Tới Nơi, tài xế lỡ upload đè ảnh sai thì bản đúng vẫn còn. Cái giá là dung lượng nhân lên, nên cần lifecycle đi kèm.

## Lifecycle: ảnh cũ chuyển sang lớp rẻ hơn

**Lifecycle rule** chạy ngầm, tự chuyển object sang storage class khác hoặc xoá object theo tuổi. Giả định nghiệp vụ: khiếu nại thường xảy ra trong 30 ngày đầu, sau đó hiếm khi ai mở lại, nhưng vẫn phải lưu để tra cứu pháp lý.

| Storage class | Hợp khi | Điều cần để ý |
| --- | --- | --- |
| Standard | Ảnh mới, truy cập thường xuyên | Không phí lấy dữ liệu |
| Standard-IA | Hiếm đọc nhưng cần đọc ngay | Tối thiểu 30 ngày; có phí lấy dữ liệu theo GB; object nhỏ hơn 128 KB tính như 128 KB |
| Glacier Instant Retrieval | Lưu lâu, đọc vài lần mỗi quý, vẫn cần mili giây | Tối thiểu 90 ngày; phí lấy dữ liệu cao hơn IA |
| Intelligent-Tiering | Không biết mẫu truy cập | S3 tự chuyển tầng; có phí giám sát theo số object, không có phí lấy dữ liệu |

Quy tắc trong lab (áp cho prefix `proof/`, như policy ở bài 03): sau 30 ngày chuyển Standard-IA, sau 90 ngày chuyển Glacier Instant Retrieval; version cũ giữ tối đa 3 bản mới nhất, các bản còn lại xoá sau 30 ngày không còn là bản hiện hành; dọn delete marker mồ côi và upload multipart dang dở sau 7 ngày.

```json
{
  "ID": "delivery-photos-tiering",
  "Status": "Enabled",
  "Filter": { "Prefix": "proof/" },
  "Transitions": [
    { "Days": 30, "StorageClass": "STANDARD_IA" },
    { "Days": 90, "StorageClass": "GLACIER_IR" }
  ],
  "NoncurrentVersionExpiration": { "NoncurrentDays": 30, "NewerNoncurrentVersions": 3 },
  "Expiration": { "ExpiredObjectDeleteMarker": true },
  "AbortIncompleteMultipartUpload": { "DaysAfterInitiation": 7 }
}
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Vòng đời một ảnh chứng từ: Standard từ ngày 0, Standard-IA từ ngày 30, Glacier Instant Retrieval từ ngày 90; phiên bản cũ chỉ giữ ba bản mới nhất và bị xoá sau 30 ngày."><rect x="20" y="40" width="270" height="70" rx="10" class="f-phys-soft s-phys"/><text x="34" y="70" class="t-b">Standard</text><text x="34" y="92" class="t-small">ngày 0 đến 30: khách và hỗ trợ hay xem</text><rect x="305" y="40" width="270" height="70" rx="10" class="f-pk-soft s-pk"/><text x="319" y="70" class="t-b">Standard-IA</text><text x="319" y="92" class="t-small">ngày 30 đến 90: hiếm đọc, có phí lấy dữ liệu</text><rect x="590" y="40" width="270" height="70" rx="10" class="f-pk-soft s-pk"/><text x="604" y="70" class="t-b">Glacier Instant Retrieval</text><text x="604" y="92" class="t-small">từ ngày 90: lưu lâu, vẫn đọc trong mili giây</text><text x="20" y="30" class="t-small t-mono">ngày 0</text><text x="305" y="30" class="t-small t-mono">ngày 30</text><text x="590" y="30" class="t-small t-mono">ngày 90</text><text x="20" y="150" class="t-b">Phiên bản cũ (noncurrent)</text><rect x="20" y="162" width="570" height="56" rx="10" class="f-warn-soft s-warn dash"/><text x="34" y="187" class="t-small">Giữ 3 bản cũ mới nhất; bản cũ hơn bị xoá sau 30 ngày không còn là bản hiện hành</text><text x="34" y="206" class="t-small">Delete marker mồ côi cũng được dọn</text><rect x="605" y="162" width="255" height="56" rx="10" class="f-surface s-line"/><text x="619" y="187" class="t-small">Thời gian chỉ minh hoạ,</text><text x="619" y="206" class="t-small">thanh không theo đúng tỉ lệ</text></svg>
</div>
<figcaption>Hình 1. Lifecycle của prefix <code>proof/</code> trong lab. Lifecycle chạy bất đồng bộ nên ngày chuyển lớp không chính xác đến từng giờ.</figcaption>
</figure>

Mỗi lần chuyển lớp là một request tính phí. Với 1,2 triệu ảnh mỗi ngày và hai lần chuyển, đó là hàng triệu request mỗi ngày; hãy dùng Pricing Calculator để chắc tiền tiết kiệm lưu trữ lớn hơn phí request. Nếu chưa biết mẫu truy cập, Intelligent-Tiering tự quyết thay bạn. Lifecycle mặc định cũng bỏ qua object nhỏ hơn 128 KB.

## Presigned URL: cho phép mà không trao credential

Ứng dụng tài xế không nên cầm credential AWS. Cách thường dùng: app tài xế hỏi service `delivery-photo`, service này dùng **role** của mình (`toinoi-lab-delivery-photo` ở bài 03) để **ký** một URL cho đúng một thao tác trên đúng một key trong thời gian ngắn, rồi tài xế gọi thẳng S3. Ảnh không đi qua server của Tới Nơi, nên không tốn băng thông và CPU của service.

Presigned URL là URL thường kèm chữ ký (SigV4) trong query string. Chữ ký phủ phương thức (GET hay PUT), bucket, key, thời điểm hết hạn và các header đã ký như `Content-Type`. URL không chứa secret của role, nhưng **ai giữ URL thì dùng được tới lúc hết hạn**, với đúng quyền của người ký tại thời điểm dùng. Một ràng buộc hay bị quên: nếu ký bằng credential tạm (role, SSO) thì URL chết cùng phiên, kể cả khi `Expires` còn dài.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Tài xế xin URL PUT từ service delivery-photo rồi upload thẳng lên S3; khách xin URL GET và tải thẳng từ S3."><defs><marker id="aws06-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws06-2-phys" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="15" width="180" height="50" rx="10" class="f-phys-soft s-phys"/><text x="110" y="45" text-anchor="middle" class="t-b">App tài xế</text><rect x="250" y="15" width="180" height="50" rx="10" class="f-pk-soft s-pk"/><text x="340" y="45" text-anchor="middle" class="t-b">delivery-photo</text><rect x="480" y="15" width="180" height="50" rx="10" class="f-phys-soft s-phys"/><text x="570" y="45" text-anchor="middle" class="t-b">Bucket S3</text><rect x="710" y="15" width="150" height="50" rx="10" class="f-phys-soft s-phys"/><text x="785" y="45" text-anchor="middle" class="t-b">App khách</text><path d="M110 65 V315 M340 65 V315 M570 65 V315 M785 65 V315" class="f-none s-line dash"/><path d="M110 108 H338" class="f-none s-pk" stroke-width="2" marker-end="url(#aws06-2-pk)"/><text x="224" y="100" text-anchor="middle" class="t-small">① xin URL upload cho chuyến</text><path d="M340 142 H112" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws06-2-pk)"/><text x="224" y="134" text-anchor="middle" class="t-small">② URL PUT, hạn 5 phút</text><path d="M110 184 H568" class="f-none s-phys" stroke-width="2" marker-end="url(#aws06-2-phys)"/><text x="340" y="176" text-anchor="middle" class="t-small">③ PUT ảnh trực tiếp, Content-Type phải khớp chữ ký</text><path d="M785 226 H342" class="f-none s-pk" stroke-width="2" marker-end="url(#aws06-2-pk)"/><text x="563" y="218" text-anchor="middle" class="t-small">④ khách xem ảnh chuyến của mình</text><path d="M340 260 H783" class="f-none s-pk dash" stroke-width="2" marker-end="url(#aws06-2-pk)"/><text x="563" y="252" text-anchor="middle" class="t-small">⑤ URL GET, hạn 15 phút</text><path d="M785 296 H572" class="f-none s-phys" stroke-width="2" marker-end="url(#aws06-2-phys)"/><text x="678" y="288" text-anchor="middle" class="t-small">⑥ GET ảnh trực tiếp</text></svg>
</div>
<figcaption>Hình 2. Hai lane của presigned URL. Mũi tên liền là request, mũi tên đứt là phản hồi. Service <code>delivery-photo</code> chỉ ký URL (tính cục bộ bằng credential của role), không đứng giữa luồng ảnh. Thời hạn 5 và 15 phút là lựa chọn của lab.</figcaption>
</figure>

`aws s3 presign` trên CLI **chỉ tạo URL GET**. URL PUT cần SDK; trong .NET (AWSSDK.S3 v4), service `delivery-photo` có thể làm như sau (đoạn mã minh hoạ, chưa được biên dịch hay chạy trên tài khoản thật):

```csharp
var request = new GetPreSignedUrlRequest
{
    BucketName = bucket,
    Key = $"proof/{tripId}/delivered.jpg",
    Verb = HttpVerb.PUT,
    ContentType = "image/jpeg",               // client phải gửi đúng Content-Type này
    Expires = DateTime.UtcNow.AddMinutes(5)
};
string url = await s3Client.GetPreSignedURLAsync(request); // ký cục bộ, không gọi mạng tới S3
```

Key luôn do **server** quyết định (`proof/<tripId>/...`), không bao giờ lấy từ client, để tài xế không ghi được vào chuyến của người khác. Thư mục lab có bản Python tương đương dùng `boto3` để thử trên máy.

## Bucket policy, CORS và những thứ chưa dùng

Quyền truy cập đúng cách gồm hai lớp. Role của `delivery-photo` có identity policy cho `s3:GetObject` và `s3:PutObject` trên `proof/*` (bài 03); bucket policy cho phép đúng role đó và chặn mọi request không dùng HTTPS bằng điều kiện `aws:SecureTransport`. Trong cùng tài khoản, chỉ cần một trong hai chỗ cho phép là đủ. Muốn khoá chặt hơn ("chỉ role này") phải dùng `Deny` kèm `aws:PrincipalArn`, nhưng viết sai sẽ tự khoá cả quản trị viên; hãy thử trên bucket trống và luôn chừa role break-glass.

**CORS** chỉ cần khi trình duyệt (web admin, web khách) gọi thẳng S3 bằng JavaScript; app di động không cần. Lab cho phép đúng origin `https://app.toinoi.example` với `GET`, `PUT` và header `Content-Type`.

S3 còn có **static website hosting**, nhưng endpoint website không hỗ trợ HTTPS; muốn phục vụ web tĩnh hoặc ảnh công khai an toàn, đặt CloudFront phía trước (bài 20). Ảnh chứng từ chứa địa chỉ nhà khách, nên không bao giờ để public.

## Lab: dựng và thử trên máy của bạn

Thư mục `examples/toi-noi-aws/p01-06-s3` có script AWS CLI v2 theo đúng thứ tự: tạo bucket (`01`), tạo presigned URL GET và xem 403 khi thiếu chữ ký hoặc hết hạn (`02`), presigned PUT bằng boto3 (`02b`), quan sát versioning (`03`) và dọn dẹp (`99`).

```bash
cd examples/toi-noi-aws/p01-06-s3
cp env.sh.example env.sh        # sửa profile; account ID được lấy từ sts
./01-create-bucket.sh
./02-presign.sh
./03-versioning-demo.sh
./99-cleanup.sh                 # xoá mọi version rồi mới xoá được bucket
```

Script đã qua kiểm tra cú pháp và shellcheck, JSON đã parse được, nhưng **chưa chạy trên tài khoản thật**: hành vi lifecycle và chữ ký presigned cần bạn xác nhận trong tài khoản lab. Với vài object test, chi phí gần như bằng không; tiền thật đến từ dung lượng, số request và phí lấy dữ liệu của lớp IA.

## Cái giá phải trả

- **Version cũ vẫn tính tiền.** Bật versioning mà không có lifecycle cho `NoncurrentVersionExpiration` thì dung lượng chỉ tăng, không bao giờ giảm.
- **Presigned URL là chìa khoá tạm.** URL lọt vào log, ảnh chụp màn hình hoặc chat thì ai nhặt được cũng dùng được tới khi hết hạn. Giữ hạn ngắn, không ghi URL đầy đủ vào log.
- **Xoá bucket không đơn giản.** Bucket có versioning phải xoá hết version và delete marker mới xoá được bucket; `aws s3 rb --force` không đủ.
- **Chuyển lớp có phí request và phí lấy dữ liệu.** Object nhỏ hoặc hay đọc lại có thể rẻ hơn nếu để ở Standard.
- **Tên bucket là tài nguyên chung toàn cầu.** Tên đẹp có thể đã bị người khác dùng, và đừng nhúng thông tin nhạy cảm vào tên.

**Bài tập:** trong bucket đã bật versioning, `aws s3api delete-object --bucket B --key proof/100123/delivered.jpg` (không có `--version-id`) làm gì? Làm sao lấy lại ảnh, và làm sao xoá ảnh vĩnh viễn?

<details><summary>Xem đáp án</summary><p>Lệnh chỉ thêm một delete marker nên GET key đó trả 404, các version cũ vẫn còn và vẫn tính tiền. Để lấy lại ảnh, xoá chính delete marker bằng <code>delete-object</code> kèm <code>--version-id</code> của marker (xem version bằng <code>list-object-versions</code>). Để xoá vĩnh viễn, xoá từng version bằng <code>--version-id</code> hoặc để lifecycle <code>NoncurrentVersionExpiration</code> làm việc đó.</p></details>

Bài trước: [bài 05](/articles/aws-ec2-ami-ebs-user-data). Bài tiếp: [bài 07](/articles/aws-rds-postgresql-multi-az-backup).

## Tài liệu gốc

- [Amazon S3 User Guide](https://docs.aws.amazon.com/AmazonS3/latest/userguide/Welcome.html).
- [Versioning trong S3](https://docs.aws.amazon.com/AmazonS3/latest/userguide/Versioning.html).
- [Quản lý vòng đời object (lifecycle)](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lifecycle-mgmt.html).
- [Chia sẻ object bằng presigned URL](https://docs.aws.amazon.com/AmazonS3/latest/userguide/using-presigned-url.html).
- [S3 Block Public Access](https://docs.aws.amazon.com/AmazonS3/latest/userguide/access-control-block-public-access.html).
- [Storage class của S3](https://docs.aws.amazon.com/AmazonS3/latest/userguide/storage-class-intro.html).
