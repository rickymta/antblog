---
slug: aws-appconfig-feature-flag-cau-hinh-dong
title: "AppConfig và feature flag: đổi bảng giá và bật tính năng mới không cần deploy"
summary: "Bài 29: AppConfig application, environment, profile feature flag và freeform, deployment strategy có bake time và rollback theo alarm, validator JSON Schema, AppConfig Agent cho ECS và Lambda; bật giá giờ cao điểm mới cho 5% khách ở một thành phố mà không cần deploy."
category: devops-aws
level: intermediate
tags: [aws, toi-noi, devops, appconfig, feature-flag, configuration, progressive-delivery, parameter-store]
language: vi
---

Đội sản phẩm của Tới Nơi muốn thử công thức giá giờ cao điểm mới: nhân 1,2 lần, chỉ cho 5% khách ở TP.HCM, bắt đầu từ 17:00. Hiện bảng giá nằm trong `appsettings.json` của service `quote`, nên mỗi lần đổi là một lần build, deploy và một lần cầu may. Tệ hơn, nếu ai đó gõ nhầm giá cơ bản thành số âm, khách sẽ thấy giá âm cho tới khi bản sửa lên production.

Đây là **bài 29** của lộ trình **AWS qua Tới Nơi · Phần 03: Infrastructure as Code và CI/CD**. Bài giả định bạn đã biết Parameter Store và Secrets Manager (bài 14) và có Lambda ở bài 18, cùng khái niệm canary ở [bài 27](/articles/aws-chien-luoc-deploy-rolling-canary-rollback).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Nếu service tự đọc một giá trị từ Parameter Store mỗi khi cần, ta đã có thể đổi cấu hình an toàn mà không deploy chưa?</p>
<details><summary>Xem đáp án</summary><p>Đổi được mà không deploy, nhưng chưa an toàn. Parameter Store không validate nội dung theo schema, không phát hành dần theo phần trăm client và không tự rollback khi lỗi tăng.</p></details>
</div>

## Ba kho, ba việc khác nhau

Ba dịch vụ cùng chứa "cấu hình" nhưng giải quyết ba bài toán khác nhau, và nhầm kho là lỗi thiết kế phổ biến.

| Dịch vụ | Chứa gì | Điểm mạnh | Không làm |
|---|---|---|---|
| Parameter Store | Giá trị đơn lẻ: endpoint, hằng số | Rẻ, đơn giản, phân cấp theo path | Không validate theo schema, không phát hành dần |
| Secrets Manager | Bí mật: mật khẩu DB, khoá cổng VíXanh | Mã hoá, rotation, kiểm soát truy cập | Không dành cho cấu hình thường |
| AppConfig | Cấu hình có vòng đời: validate, deploy dần, rollback | Strategy, bake time, alarm, flag | Không thay thế kho bí mật |

Cần AppConfig khi một thay đổi cấu hình có thể làm hỏng hành vi của hệ thống (bảng giá, ngưỡng rate limit, flag bật tính năng) và bạn muốn đối xử với nó như một lần release nhỏ. Nguồn của AppConfig có thể là kho hosted trong chính AppConfig, hoặc Parameter Store, Secrets Manager, S3 mà AppConfig đọc rồi phát hành. Bài này dùng hosted cho đơn giản.

## Feature flag và cấu hình tự do

AppConfig tổ chức theo **application** (trong lab là `toinoi-lab-quote`), **environment** (trong lab là `lab`) và **configuration profile**. Profile có hai loại chính. **Feature flag** có cấu trúc cố định: mỗi flag có `name`, các `attributes` kèm ràng buộc và phần `values` cho từng flag. **Freeform** là JSON hoặc YAML tuỳ ý, thích hợp cho bảng giá. Với flag, AppConfig tự kiểm cấu trúc và ràng buộc của attribute; với freeform, bạn gắn validator JSON Schema (AppConfig hỗ trợ draft-04) hoặc một hàm Lambda cho luật nghiệp vụ phức tạp hơn, ví dụ so sánh chéo hai trường. Validator chạy khi bạn bắt đầu deployment: một phiên bản sai có thể đã được lưu, nhưng `StartDeployment` bị từ chối nên nó không tới được client.

```json
{
  "version": "1",
  "flags": {
    "peak-pricing-v2": {
      "name": "peak-pricing-v2",
      "attributes": {
        "multiplier": { "constraints": { "type": "number", "required": true, "minimum": 1, "maximum": 3 } },
        "city": { "constraints": { "type": "string", "required": true, "pattern": "^[a-z-]+$" } },
        "rolloutPercent": { "constraints": { "type": "number", "required": true, "minimum": 0, "maximum": 100 } }
      }
    }
  },
  "values": {
    "peak-pricing-v2": { "enabled": true, "multiplier": 1.2, "city": "ho-chi-minh", "rolloutPercent": 5 }
  }
}
```

Flag trên có hệ số từ 1 đến 3, nên một lần gõ nhầm 12 sẽ bị chặn trước khi tới khách. Còn bảng giá nằm ở profile freeform `pricing-table`, kèm schema yêu cầu `baseFare` và `pricePerExtraKm` tối thiểu 1.000, nên phiên bản có giá âm không deploy được:

```json
{ "baseFare": { "type": "number", "minimum": 1000 } }
```

Về **targeting**, có hai cách. Cách thứ nhất, từ 2024 AppConfig có **multi-variant feature flag**: một flag có nhiều variant, mỗi variant gắn một luật đánh giá trên ngữ cảnh (context) mà service gửi cho agent, ví dụ thành phố và mã khách. Bài này chỉ nhắc tới; cú pháp luật xem trong user guide. Cách thứ hai, dễ kiểm hơn và là cách lab dùng: cấu hình mang `city` và `rolloutPercent`, còn **service tự quyết định** bằng cách băm `customerId` vào bucket 0 đến 99. Cùng một khách luôn rơi vào cùng bucket, nên không bị giá nhảy qua lại giữa hai lần xem. Chi phí của cách này là logic targeting nằm trong code của bạn và phải được kiểm thử.

## Deployment strategy: phát hành cấu hình như phát hành code

Tạo một phiên bản cấu hình không làm gì cả; chỉ **deployment** mới đưa nó tới client. Một deployment cần ba thứ: environment, profile cùng phiên bản, và **deployment strategy**.

| Chiến lược có sẵn | Hành vi | Bake time |
|---|---|---|
| `AppConfig.AllAtOnce` | Mọi client nhận ngay | 10 phút |
| `AppConfig.Linear50PercentEvery30Seconds` | 50% mỗi 30 giây, chỉ để thử hoặc demo | 1 phút |
| `AppConfig.Canary10Percent20Minutes` | Tăng theo hàm mũ từ 10% trong 20 phút | 10 phút |
| `AppConfig.Linear20PercentEvery6Minutes` | 20% mỗi 6 phút, AWS khuyến nghị cho production | 30 phút |
| Tự tạo | Chọn duration, growth type (`LINEAR` hoặc `EXPONENTIAL`), growth factor, final bake time | Tuỳ chọn |

Phần trăm ở đây áp lên **client** (phiên cấu hình đang poll), không phải khách hàng cuối. **Final bake time** là khoảng AppConfig tiếp tục theo dõi alarm sau khi mọi client đã nhận. Gắn một CloudWatch alarm làm **monitor** của environment: nếu alarm vào trạng thái ALARM trong lúc deploy hoặc bake, AppConfig tự rollback về phiên bản trước. Mỗi environment chỉ chạy một deployment tại một thời điểm, nên bake time dài cũng là thời gian chờ trước khi đẩy được cấu hình tiếp theo.

```bash
aws appconfig create-deployment-strategy --name toinoi-linear20-bake10 \
  --deployment-duration-in-minutes 20 --final-bake-time-in-minutes 10 \
  --growth-factor 10 --growth-type LINEAR --replicate-to NONE
aws appconfig start-deployment --application-id "$APP_ID" --environment-id "$ENV_ID" \
  --deployment-strategy-id "$STRATEGY_ID" --configuration-profile-id "$PROFILE_ID" \
  --configuration-version 2
```

Lệnh đầu tạo chiến lược 20 phút, bake 10 phút; lệnh sau bắt đầu deploy phiên bản 2. Trên môi trường dev dùng `AppConfig.AllAtOnce` cho nhanh, còn prod dùng chiến lược có bake time. Alarm phải gắn với một metric có nghĩa, ví dụ tỉ lệ 5xx của `quote` hay tỉ lệ báo giá bị từ chối, chứ không phải CPU.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Hình 1. Một phiên bản cấu hình đi qua validator, deployment strategy phát hành dần tới client, rồi bake time; alarm CloudWatch kêu thì AppConfig rollback về phiên bản trước."><defs><marker id="aws29-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="aws29-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="10" y="40" width="150" height="80" rx="10" class="f-phys-soft s-phys"/><text x="22" y="68" class="t-b">Phiên bản mới</text><text x="22" y="90" class="t-small">Hosted version,</text><text x="22" y="106" class="t-small">flag hoặc freeform</text><path d="M160 80 L190 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws29-1-a)"/><rect x="190" y="40" width="150" height="80" rx="10" class="f-pk-soft s-pk"/><text x="202" y="68" class="t-b">Validator</text><text x="202" y="90" class="t-small">Ràng buộc, JSON</text><text x="202" y="106" class="t-small">Schema hoặc Lambda</text><path d="M340 80 L370 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws29-1-a)"/><rect x="370" y="40" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="382" y="68" class="t-b">Deployment</text><text x="382" y="90" class="t-small">Tăng dần % client</text><text x="382" y="106" class="t-small">theo strategy</text><path d="M540 80 L570 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws29-1-a)"/><rect x="570" y="40" width="140" height="80" rx="10" class="f-phys-soft s-phys"/><text x="582" y="68" class="t-b">Bake time</text><text x="582" y="90" class="t-small">Canh chừng alarm</text><path d="M710 80 L740 80" class="f-none s-phys" stroke-width="2" marker-end="url(#aws29-1-a)"/><rect x="740" y="40" width="130" height="80" rx="10" class="f-phys-soft s-phys"/><text x="752" y="68" class="t-b">Hoàn tất</text><text x="752" y="90" class="t-small">100% client</text><rect x="370" y="160" width="340" height="50" rx="10" class="f-warn-soft s-warn"/><text x="382" y="182" class="t-b t-warn">CloudWatch alarm = ALARM</text><text x="382" y="199" class="t-small">AppConfig rollback về phiên bản trước</text><path d="M540 160 L540 122" class="f-none s-warn dash" stroke-width="1.6" marker-end="url(#aws29-1-w)"/></svg>
</div>
<figcaption>Hình 1. Một deployment AppConfig đi qua validator, phát hành dần và có bake time; alarm là đường lùi tự động.</figcaption>
</figure>

## Lấy cấu hình trong service: dùng Agent, đừng gọi API mỗi request

Service không nên gọi API AppConfig trong luồng xử lý mỗi báo giá. Với 30.000 lượt đọc mỗi giây lúc cao điểm, đó vừa là độ trễ, vừa là chi phí theo từng lượt yêu cầu. Thay vào đó dùng **AppConfig Agent**: một tiến trình chạy cạnh ứng dụng, poll AppConfig theo chu kỳ, giữ cấu hình trong bộ nhớ và phục vụ qua HTTP trên `localhost:2772`. Với Lambda, agent là extension (một layer); với ECS, agent là container sidecar trong cùng task. Chu kỳ poll mặc định của extension khoảng 45 giây, nên đổi cấu hình có độ trễ tới client cỡ chu kỳ đó, cộng thêm thời gian của strategy.

```python
def fetch(profile: str) -> dict:
    url = f"http://localhost:2772/applications/{APP}/environments/{ENV}/configurations/{profile}"
    with urllib.request.urlopen(url, timeout=2) as response:
        return json.loads(response.read())
```

Đoạn này nằm trong `reader/handler.py` của lab. Agent gọi `StartConfigurationSession` rồi `GetLatestConfiguration` thay bạn, vì vậy role của Lambda hoặc task chỉ cần hai quyền đó trên đúng ARN profile, không cần quyền ghi. Nếu AppConfig tạm không truy cập được, agent tiếp tục phục vụ bản đã lưu, và service nên có giá trị mặc định an toàn khi chưa đọc được gì.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 2. Service quote đọc cấu hình từ AppConfig Agent trên localhost, agent poll AppConfig theo chu kỳ và giữ cache trong bộ nhớ."><defs><marker id="aws29-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="10" y="50" width="200" height="90" rx="10" class="f-phys-soft s-phys"/><text x="22" y="78" class="t-b">Service quote</text><text x="22" y="100" class="t-small">Container hoặc Lambda</text><text x="22" y="116" class="t-small">đọc cache mỗi request</text><path d="M210 95 L270 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws29-2-a)"/><text x="215" y="84" class="t-small t-mono">:2772</text><rect x="270" y="50" width="250" height="90" rx="10" class="f-pk-soft s-pk"/><text x="282" y="78" class="t-b">AppConfig Agent</text><text x="282" y="100" class="t-small">Sidecar (ECS) hoặc extension</text><text x="282" y="116" class="t-small">Cache trong bộ nhớ, poll định kỳ</text><path d="M520 95 L580 95" class="f-none s-phys" stroke-width="2" marker-end="url(#aws29-2-a)"/><text x="530" y="84" class="t-small">~45 giây</text><rect x="580" y="50" width="290" height="90" rx="10" class="f-phys-soft s-phys"/><text x="592" y="78" class="t-b">AppConfig</text><text x="592" y="100" class="t-small">StartConfigurationSession,</text><text x="592" y="116" class="t-small">GetLatestConfiguration</text><text x="10" y="175" class="t-small">Không gọi API AppConfig trong luồng xử lý từng yêu cầu báo giá.</text></svg>
</div>
<figcaption>Hình 2. Agent đứng giữa: service đọc cache cục bộ, agent mới là bên nói chuyện với AppConfig.</figcaption>
</figure>

## Lỗi thường gặp và cái giá phải trả

- **Flag để mãi không dọn**: mỗi flag là một nhánh code phải kiểm thử. Ghi ngày dự kiến gỡ ngay khi tạo.
- **Alarm vô nghĩa**: gắn alarm CPU thì deployment vẫn "xanh" dù giá sai. Dùng metric nghiệp vụ.
- **Gọi API mỗi request**: tốn chi phí theo lượt và làm chậm báo giá; dùng Agent.
- **Quên độ trễ của poll**: tắt flag không có hiệu lực tức thì; chu kỳ poll cộng thời gian strategy là độ trễ thật của "nút tắt khẩn cấp".
- **Không có giá trị mặc định**: service phải chạy được với bảng giá nội trang khi chưa đọc được cấu hình.
- **Chưa thử trên tài khoản thật**: validator, rollback theo alarm và targeting trong lab chỉ được kiểm trên máy, cần xác nhận trong tài khoản lab.

**Bài tập:** feature flag bật cho 5% khách khác một thử nghiệm A/B ở điểm nào?

<details><summary>Xem đáp án</summary><p>Flag là cơ chế <em>phân phối</em>: bật hoặc tắt một hành vi cho một nhóm khách và cho phép rollback nhanh. A/B test là phương pháp <em>đo lường</em>: gán ngẫu nhiên có kiểm soát vào nhóm thử và nhóm đối chứng, định trước metric và cỡ mẫu rồi mới kết luận. Flag có thể là công cụ để chạy A/B test, nhưng AppConfig không tự đo hiệu quả hay tính ý nghĩa thống kê.</p></details>

## Lab và kiểm chứng

Mã nằm ở `examples/toi-noi-aws/p03-29-appconfig`: Terraform dựng application, environment, hai profile, strategy, alarm monitor và một Lambda đọc cấu hình tuỳ chọn; ngoài ra có script AWS CLI làm cùng việc. Trên máy, kiểm cú pháp Terraform và chạy unit test logic báo giá:

```bash
cd examples/toi-noi-aws/p03-29-appconfig
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
python3 -m unittest discover -s reader
```

Lệnh đầu cần `init -backend=false` trước, xem README của lab. `validate` không chứng minh quyền IAM hay hành vi rollback. README ghi tài nguyên nào tính tiền và lệnh dọn.

Bài trước: [bài 28](/articles/aws-organizations-scp-control-tower-landing-zone). Bài tiếp: [bài 30](/articles/aws-policy-as-code-checkov-infracost-drift).

## Tài liệu gốc

- [AWS AppConfig User Guide](https://docs.aws.amazon.com/appconfig/latest/userguide/what-is-appconfig.html)
- [AWS Systems Manager Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-parameter-store.html)
- [AWS Secrets Manager User Guide](https://docs.aws.amazon.com/secretsmanager/latest/userguide/intro.html)
- [Terraform: aws_appconfig_configuration_profile](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/appconfig_configuration_profile)
- [Terraform: aws_appconfig_deployment_strategy](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/appconfig_deployment_strategy)
