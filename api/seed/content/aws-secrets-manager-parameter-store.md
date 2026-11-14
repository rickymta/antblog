---
slug: aws-secrets-manager-parameter-store
title: "Secrets Manager và Parameter Store: cấu hình, secret và rotation cho service Tới Nơi"
summary: "Bài 14: chia cấu hình thường vào Parameter Store (phân cấp /toinoi/lab/booking/...) và secret vào Secrets Manager; rotation mật khẩu RDS, cache secret trong ứng dụng, quyền đọc theo ARN, tag và KMS key riêng, chi phí mỗi secret và mỗi lần gọi API. Có lab CLI và Terraform."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, secrets-manager, parameter-store, rotation, configuration, kms]
language: vi
---

Cuối quý, đội bảo mật của Tới Nơi yêu cầu đổi mật khẩu database của service đặt chuyến. Mật khẩu đó đang nằm ở ba chỗ: file `.env` trên máy một dev, task definition revision 12, và một trang wiki. Đổi mật khẩu nghĩa là sửa cả ba rồi deploy, và nếu quên một chỗ thì service mất kết nối đúng giờ cao điểm 17:00. Vấn đề không phải là đổi mật khẩu khó, mà là mật khẩu được *chép* đi nhiều nơi. Bài này chuyển cấu hình và secret về một nguồn duy nhất, có quyền đọc, có lịch sử và có thể xoay vòng.

Đây là **bài 14** của lộ trình **AWS qua Tới Nơi · Phần 02: Container, serverless và dịch vụ ứng dụng**. Bài cần [bài 12](/articles/aws-ecs-fargate-task-definition-roles), nơi task definition nhận secret qua `valueFrom`, và bài 03 về IAM.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Số giây cache báo giá (<code>60</code>) và mật khẩu PostgreSQL đều là "cấu hình". Có nên cất chúng cùng một chỗ, cùng một cách không?</p>
<details><summary>Xem đáp án</summary><p><b>Không nhất thiết.</b> Số giây cache không bí mật, đổi tuỳ ý, đọc rất nhiều lần: hợp với Parameter Store loại Standard. Mật khẩu cần mã hoá, kiểm soát chặt, có phiên bản và nên xoay vòng định kỳ: hợp với Secrets Manager.</p></details>
</div>

## Hai kho, hai việc

| | Parameter Store (Systems Manager) | Secrets Manager |
|---|---|---|
| Kiểu giá trị | `String`, `StringList`, `SecureString` (mã hoá bằng KMS) | Chuỗi hoặc JSON, luôn mã hoá bằng KMS |
| Kích thước | 4 KB (Standard), 8 KB (Advanced) | Tới 64 KB |
| Cấu trúc | Cây phân cấp theo đường dẫn | Tên phẳng, quy ước dùng dấu `/` |
| Phiên bản | Có lịch sử, có nhãn | Nhãn `AWSCURRENT`, `AWSPREVIOUS`, `AWSPENDING` |
| Rotation tự động | Không có sẵn | Có, bằng Lambda hoặc managed rotation |
| Chi phí | Standard không tính phí lưu trữ | Tính theo secret mỗi tháng và mỗi 10.000 lần gọi API |

Con số quota và giá thay đổi, hãy xem trang giá hiện hành. Tới Nơi chốt quy ước: **cấu hình thường vào Parameter Store, thứ cho phép truy cập tài nguyên (mật khẩu database, khoá cổng thanh toán VíXanh) vào Secrets Manager**.

## Parameter Store: cấu hình theo đường dẫn

Đặt tên theo `/toinoi/<env>/<service>/<tên>`, ví dụ `/toinoi/lab/booking/quote-cache-ttl-seconds`. Đường dẫn cho phép đọc cả nhánh một lần và cấp quyền theo nhánh.

```bash
aws ssm put-parameter --name /toinoi/lab/booking/quote-cache-ttl-seconds \
  --type String --value 60 --tags Key=Project,Value=ToiNoi Key=Service,Value=booking
aws ssm put-parameter --name /toinoi/lab/booking/vixanh-api-key \
  --type SecureString --key-id alias/toinoi-lab-config --value file://vixanh-api-key.txt
```

Dòng đầu tạo parameter thường. Dòng sau tạo SecureString mã hoá bằng KMS key riêng, đọc giá trị từ tệp qua `file://` để giá trị bí mật không lọt vào lịch sử shell hay danh sách tiến trình. `--key-id` bỏ trống thì dùng key mặc định `alias/aws/ssm`. Lưu ý: không dùng `--tags` cùng `--overwrite`; muốn gắn tag cho parameter đã có thì dùng `add-tags-to-resource`.

```bash
aws ssm get-parameters-by-path --path /toinoi/lab/booking --recursive --with-decryption
```

Lệnh này trả về mọi parameter dưới nhánh đó; `--with-decryption` mới giải mã SecureString. Loại Advanced thêm dung lượng lớn hơn và policy như hết hạn (expiration), đổi lại tính phí theo parameter.

## Secrets Manager: secret JSON và nhãn phiên bản

```bash
aws secretsmanager create-secret --name toinoi/lab/booking/db \
  --kms-key-id alias/toinoi-lab-config --secret-string file://db-secret.json \
  --tags Key=Project,Value=ToiNoi Key=Service,Value=booking
```

Lệnh tạo secret mã hoá bằng key riêng; `db-secret.json` là tệp tạm không commit. Dạng JSON chuẩn `{"username","password","engine","host","port","dbname"}` đáng theo vì hàm rotation mẫu của AWS đọc đúng cấu trúc này. Mỗi lần ghi giá trị mới tạo một phiên bản; nhãn `AWSCURRENT` trỏ vào bản đang dùng, `AWSPREVIOUS` giữ bản liền trước. Rotation dùng thêm `AWSPENDING` cho bản đang được chuẩn bị.

Có một điểm không khớp với bài 36 của loạt .NET và task definition ở bài 12, nơi secret là *một chuỗi connection string đầy đủ* được inject nguyên vào `ConnectionStrings__Trips`. Chuỗi phẳng thì rotation mẫu không biết sửa chỗ nào; ngược lại, inject nguyên secret JSON vào biến đó thì Npgsql nhận một chuỗi JSON và không kết nối được. Lab bài này giữ secret JSON để còn bật rotation. Ba cách dung hoà: ECS inject từng trường qua hậu tố `:password::`, `:host::`… rồi app tự ghép chuỗi kết nối lúc khởi động; app tự đọc secret JSON bằng SDK lúc chạy; hoặc giữ một secret riêng chứa connection string và tự viết hàm rotation cho nó. Cách thứ nhất ít đổi code nhất.

## Ai đọc được gì

Cả hai kho dùng chung ba lớp kiểm soát: quyền IAM theo ARN, điều kiện theo tag, và quyền trên KMS key. Policy chỉ cho service booking đọc đúng thứ của nó:

```json
{
  "Effect": "Allow",
  "Action": "secretsmanager:GetSecretValue",
  "Resource": "arn:aws:secretsmanager:ap-southeast-1:123456789012:secret:toinoi/lab/booking/*",
  "Condition": { "StringEquals": { "secretsmanager:ResourceTag/Project": "ToiNoi" } }
}
```

Đường dẫn `toinoi/lab/booking/*` ngăn service booking đọc secret của dispatch. Điều kiện tag thêm một lớp nữa. ARN secret có hậu tố ngẫu nhiên sáu ký tự, nên dùng ký tự đại diện `*` thay vì đoán hậu tố. KMS key riêng cho phép tách quyền: ai chỉ có quyền secret mà không có `kms:Decrypt` trên key thì không đọc được. Với key mặc định của AWS thì không làm được việc này và không dùng được khi chia sẻ qua tài khoản khác.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Task booking khởi động: ECS agent dùng execution role gọi Secrets Manager hoặc Parameter Store, kho này nhờ KMS key giải mã, giá trị được đặt thành biến môi trường của container. CloudTrail ghi lại mọi lần đọc và giải mã.">
<defs>
<marker id="aws14-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
<marker id="aws14-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker>
</defs>
<rect x="20" y="40" width="180" height="80" rx="10" class="f-pk-soft s-pk"/><text x="34" y="72" class="t-b">ECS agent</text><text x="34" y="94" class="t-small">execution role</text>
<rect x="260" y="40" width="200" height="80" rx="10" class="f-phys-soft s-phys"/><text x="274" y="72" class="t-b">Secrets Manager</text><text x="274" y="94" class="t-small">hoặc Parameter Store</text>
<rect x="520" y="40" width="160" height="80" rx="10" class="f-phys-soft s-phys"/><text x="534" y="72" class="t-b">KMS key riêng</text><text x="534" y="94" class="t-small">kms:Decrypt</text>
<rect x="720" y="40" width="140" height="80" rx="10" class="f-surface s-line"/><text x="734" y="72" class="t-b">Container</text><text x="734" y="94" class="t-small">biến môi trường</text>
<line x1="200" y1="80" x2="258" y2="80" class="f-none s-phys" stroke-width="1.8" marker-end="url(#aws14-1-a)"/>
<line x1="460" y1="80" x2="518" y2="80" class="f-none s-phys" stroke-width="1.8" marker-end="url(#aws14-1-a)"/>
<path d="M600 120 L600 150 L790 150 L790 122" class="f-none s-phys" stroke-width="1.8" marker-end="url(#aws14-1-a)"/>
<text x="610" y="143" class="t-small">giá trị đã giải mã</text>
<rect x="260" y="185" width="420" height="42" rx="8" class="f-warn-soft s-warn"/><text x="274" y="210" class="t-warn t-b">CloudTrail ghi GetSecretValue, GetParameters, Decrypt</text>
<path d="M360 120 L360 183" class="f-none s-warn dash" stroke-width="1.4" marker-end="url(#aws14-1-w)"/>
</svg>
</div>
<figcaption>Hình 1. Đường đi của một secret lúc task khởi động. Mỗi chặng cần quyền riêng, và mỗi lần đọc đều để lại dấu vết trong CloudTrail.</figcaption>
</figure>

CloudTrail ghi lại các lời gọi đọc như `GetSecretValue` và `Decrypt` (kèm danh tính và thời điểm), nhưng không ghi giá trị. Đó là bằng chứng cần có khi điều tra ai đã đọc secret nào.

## Ứng dụng đọc và cache secret

ECS đọc secret **một lần lúc task khởi động** (bài 12). Cách này rất đơn giản, nhưng giá trị bị cố định từ lúc đó. Có hai lựa chọn:

| Cách đọc | Khi giá trị đổi | Ghi chú |
|---|---|---|
| ECS inject (`secrets.valueFrom`) | Chỉ đổi khi có task mới (deploy, `--force-new-deployment`) | Không cần code AWS SDK |
| SDK lúc chạy kèm cache | Đổi sau thời gian sống của cache hoặc khi gặp lỗi xác thực | Cần task role có quyền đọc; thư viện caching có cho Java, Python, .NET, Go |
| Lambda extension | Cache cục bộ cho hàm Lambda | Dùng ở bài 18 |

Đừng gọi `GetSecretValue` ở mỗi request. Với 30.000 lượt đọc mỗi giây lúc cao điểm, đó là chừng ấy lời gọi API tính phí và có thể chạm quota; cache trong tiến trình giảm xuống còn vài lời gọi mỗi giờ. Với .NET, `Amazon.Extensions.Configuration.SystemsManager` nạp parameter vào `IConfiguration` kèm tự làm mới, và thư viện `AWSSDK.SecretsManager.Caching` cache secret.

## Rotation và cửa sổ hai mật khẩu

Rotation chạy bốn bước qua Lambda: `createSecret` (sinh mật khẩu mới, gắn `AWSPENDING`), `setSecret` (đặt mật khẩu mới trên database), `testSecret` (đăng nhập thử) và `finishSecret` (chuyển `AWSCURRENT` sang bản mới, bản cũ thành `AWSPREVIOUS`). Có hai chiến lược:

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Bốn bước rotation: createSecret, setSecret, testSecret, finishSecret. Với chiến lược một user, mật khẩu cũ ngừng hợp lệ ở bước setSecret nên có khoảng ngắn kết nối mới bị từ chối. Với chiến lược hai user luân phiên, user còn lại vẫn dùng được suốt quá trình.">
<defs>
<marker id="aws14-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker>
</defs>
<rect x="20" y="20" width="190" height="56" rx="8" class="f-phys-soft s-phys"/><text x="34" y="44" class="t-mono t-b">createSecret</text><text x="34" y="62" class="t-small">mật khẩu mới, AWSPENDING</text>
<rect x="240" y="20" width="190" height="56" rx="8" class="f-phys-soft s-phys"/><text x="254" y="44" class="t-mono t-b">setSecret</text><text x="254" y="62" class="t-small">đặt mật khẩu trên DB</text>
<rect x="460" y="20" width="190" height="56" rx="8" class="f-phys-soft s-phys"/><text x="474" y="44" class="t-mono t-b">testSecret</text><text x="474" y="62" class="t-small">đăng nhập thử bản mới</text>
<rect x="680" y="20" width="180" height="56" rx="8" class="f-pk-soft s-pk"/><text x="694" y="44" class="t-mono t-b">finishSecret</text><text x="694" y="62" class="t-small">AWSCURRENT chuyển sang bản mới</text>
<line x1="210" y1="48" x2="238" y2="48" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws14-2-a)"/>
<line x1="430" y1="48" x2="458" y2="48" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws14-2-a)"/>
<line x1="650" y1="48" x2="678" y2="48" class="f-none s-phys" stroke-width="1.6" marker-end="url(#aws14-2-a)"/>
<text x="20" y="116" class="t-b">Một user</text>
<rect x="20" y="126" width="420" height="30" rx="6" class="f-phys-soft s-phys"/><text x="30" y="146" class="t-small">mật khẩu cũ hợp lệ</text>
<rect x="440" y="126" width="140" height="30" rx="6" class="f-warn-soft s-warn"/><text x="450" y="146" class="t-small t-warn">cửa sổ rủi ro</text>
<rect x="580" y="126" width="280" height="30" rx="6" class="f-pk-soft s-pk"/><text x="590" y="146" class="t-small">mật khẩu mới hợp lệ</text>
<text x="20" y="196" class="t-b">Hai user luân phiên</text>
<rect x="20" y="206" width="560" height="30" rx="6" class="f-phys-soft s-phys"/><text x="30" y="226" class="t-small">user A hợp lệ, vẫn phục vụ</text>
<rect x="580" y="206" width="280" height="30" rx="6" class="f-pk-soft s-pk"/><text x="590" y="226" class="t-small">user B (mật khẩu mới) thay user A</text>
</svg>
</div>
<figcaption>Hình 2. Hai chiến lược rotation. Chiến lược hai user luân phiên không có cửa sổ lỗi, đổi lại cần hai user database và quyền tương ứng.</figcaption>
</figure>

Với một user, sau `setSecret` mật khẩu cũ ngừng hợp lệ trong khi ứng dụng có thể chưa đọc secret mới. Kết nối đang mở vẫn sống, nhưng kết nối mở mới sẽ bị từ chối cho tới khi app lấy bản mới. Với hai user luân phiên, ta có đúng "cửa sổ hai mật khẩu": hai user cùng hợp lệ trong lúc chuyển. Phía Tới Nơi, service nên bắt lỗi xác thực khi mở kết nối, đọc lại secret, dựng lại connection pool rồi thử lại một lần. Giới hạn tuổi kết nối trong pool (ví dụ `Connection Lifetime` của Npgsql) để pool tự thay kết nối cũ.

```bash
aws secretsmanager rotate-secret --secret-id toinoi/lab/booking/db \
  --rotation-lambda-arn arn:aws:lambda:ap-southeast-1:123456789012:function:toinoi-lab-rotate-db \
  --rotation-rules AutomaticallyAfterDays=30
```

Lệnh bật rotation mỗi 30 ngày và chạy một lần ngay. Hàm Lambda và đường mạng từ hàm tới database phải có sẵn, bài này không dựng chúng. Với mật khẩu master do RDS quản lý (`manage_master_user_password`), RDS tự tạo secret trong Secrets Manager và lo rotation, không cần Lambda của bạn. Cả hai cơ chế chưa được kiểm chứng trên tài khoản thật trong lúc soạn bài.

## Lỗi thường gặp

- **Secret trong biến môi trường dạng plain.** Hiện trong `DescribeTaskDefinition`, console và lịch sử revision. Dùng `secrets`.
- **Gọi `GetSecretValue` mỗi request.** Tốn phí, chạm throttle. Cache trong tiến trình.
- **Quên rằng ECS inject chỉ một lần.** Rotation xong mà task cũ giữ mật khẩu cũ. Dùng SDK kèm cache hoặc deploy lại sau rotation.
- **Xoá secret rồi tạo lại cùng tên.** Secret nằm trong khoảng chờ xoá 7–30 ngày nên tên bị giữ. Chỉ bỏ khoảng chờ trong lab.
- **Cấp `secretsmanager:*` trên `*`.** Một service bị xâm phạm sẽ làm lộ mọi secret. Giới hạn theo ARN, tag và KMS key.

**Bài tập:** vì sao không đặt mật khẩu vào `environment` của task definition dạng plain?

<details><summary>Xem đáp án</summary><p>Mọi người có quyền <code>ecs:DescribeTaskDefinition</code> đều đọc được giá trị; nó nằm trong mọi revision cũ, lịch sử console và thường cả trong repository nếu task definition được commit. Đổi mật khẩu thì phải đăng ký revision mới và các revision cũ vẫn chứa mật khẩu cũ. Với <code>secrets.valueFrom</code>, task definition chỉ chứa ARN, giá trị nằm ở Secrets Manager (mã hoá, kiểm soát quyền, có CloudTrail, xoay vòng được).</p></details>

## Lab: tạo parameter và secret, đọc bằng role

Mã ở `examples/toi-noi-aws/p02-14-secrets-parameters`: Terraform tạo KMS key, bốn parameter, "vỏ" secret DB, policy đọc theo ARN, tag và KMS, cùng một role chỉ đọc; script `seed-values.sh` đặt giá trị thật bằng CLI để không nằm trong state, `read-as-role.sh` đọc lại bằng role và `versions-demo.sh` mô phỏng hai nhãn phiên bản. README ghi tài nguyên tính tiền (mỗi secret, mỗi KMS key, mỗi lần gọi API) và lệnh dọn.

Kiểm trên máy: `terraform fmt`, `init -backend=false`, `validate` qua Docker; parse JSON của policy; `bash -n` cho script. **Chưa kiểm chứng trên tài khoản thật:** điều kiện tag và `kms:ViaService` trong policy, việc ECS inject đúng khoá JSON, và toàn bộ rotation.

Bài trước: [bài 13](/articles/aws-ecs-alb-rolling-deploy-autoscaling). Bài tiếp: [bài 15](/articles/aws-elasticache-redis-cache-rate-limit).

## Tài liệu gốc

- [AWS Secrets Manager User Guide](https://docs.aws.amazon.com/secretsmanager/latest/userguide/intro.html).
- [Rotate AWS Secrets Manager secrets](https://docs.aws.amazon.com/secretsmanager/latest/userguide/rotating-secrets.html).
- [AWS Systems Manager Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html).
- [Pass sensitive data to an Amazon ECS container](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/specifying-sensitive-data.html).
- [AWS KMS Developer Guide](https://docs.aws.amazon.com/kms/latest/developerguide/overview.html).
