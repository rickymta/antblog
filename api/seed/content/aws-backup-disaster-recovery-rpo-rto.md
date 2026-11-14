---
slug: aws-backup-disaster-recovery-rpo-rto
title: "AWS Backup và disaster recovery: RPO, RTO, pilot light, warm standby và diễn tập restore"
summary: "Bài 37: xác định RPO và RTO cho từng loại dữ liệu của Tới Nơi (trips, ví, vị trí), dựng AWS Backup với plan, vault, copy cross-region và cross-account, Vault Lock; so sánh bốn chiến lược DR từ backup and restore tới multi-site, Elastic Disaster Recovery và game day restore có đo thời gian."
category: devops-aws
level: advanced
tags: [aws, toi-noi, devops, aws-backup, disaster-recovery, rpo-rto, pilot-light, warm-standby, restore]
language: vi
---

Một kỹ sư chạy nhầm lệnh xoá trên bảng `trips` của môi trường production lúc 14:20. Đội trực mở danh sách backup, tìm thấy bản chụp đêm qua và bắt đầu restore. Hai tiếng sau database lên lại, nhưng 14 tiếng dữ liệu, tức hàng triệu chuyến và giao dịch, đã mất. Không ai trả lời được câu "mất bao nhiêu thì chấp nhận được, và chờ bao lâu thì chấp nhận được" vì chưa ai hỏi trước khi sự cố xảy ra. Bài này đặt hai con số đó cho từng loại dữ liệu của Tới Nơi rồi chọn công cụ tương ứng.

Đây là **bài 37** của lộ trình **AWS qua Tới Nơi · Phần 04: Dữ liệu, lưu trữ và messaging**. Bạn cần biết backup và Multi-AZ của RDS ([bài 07](/articles/aws-rds-postgresql-multi-az-backup)) và replication của S3 ([bài 36](/articles/aws-s3-nang-cao-storage-class-replication-object-lock)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Database bật Multi-AZ, nên đội hạ tầng nói "đã có backup rồi". Một nhân viên chạy nhầm <code>DELETE</code> không có <code>WHERE</code>. Multi-AZ có cứu được không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> Multi-AZ nhân bản đồng bộ sang standby, nên lệnh xoá cũng được nhân bản ngay. Multi-AZ chống hỏng phần cứng hoặc mất AZ, còn xoá nhầm cần backup hoặc point-in-time restore.</p></details>
</div>

## RPO, RTO và ma trận quyết định

**RPO** (Recovery Point Objective) là lượng dữ liệu tối đa chấp nhận mất, đo bằng thời gian: RPO 5 phút nghĩa là mất tối đa 5 phút ghi gần nhất. **RTO** (Recovery Time Objective) là thời gian tối đa từ lúc sự cố tới lúc dịch vụ hoạt động lại. Hai con số độc lập: backup mỗi giờ cho RPO một giờ nhưng restore vẫn có thể mất sáu tiếng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 1. Trục thời gian: backup gần nhất, sự cố, dịch vụ chạy lại. Khoảng từ backup gần nhất tới sự cố là dữ liệu mất, đối ứng RPO; khoảng từ sự cố tới dịch vụ chạy lại là thời gian gián đoạn, đối ứng RTO."><defs><marker id="aws37-1-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs><path d="M40 120 H840" class="f-none s-muted" stroke-width="2" marker-end="url(#aws37-1-pk)"/><circle cx="160" cy="120" r="8" class="f-phys"/><text x="160" y="95" text-anchor="middle" class="t-b">Backup gần nhất</text><text x="160" y="150" text-anchor="middle" class="t-small t-mono">00:00</text><circle cx="470" cy="120" r="8" class="f-warn"/><text x="470" y="95" text-anchor="middle" class="t-b t-warn">Sự cố</text><text x="470" y="150" text-anchor="middle" class="t-small t-mono">14:20</text><circle cx="720" cy="120" r="8" class="f-pk"/><text x="720" y="95" text-anchor="middle" class="t-b">Dịch vụ chạy lại</text><text x="720" y="150" text-anchor="middle" class="t-small t-mono">16:20</text><rect x="160" y="170" width="310" height="36" rx="10" class="f-warn-soft s-warn"/><text x="315" y="193" text-anchor="middle" class="t-small">dữ liệu mất, so với RPO</text><rect x="470" y="170" width="250" height="36" rx="10" class="f-pk-soft s-pk"/><text x="595" y="193" text-anchor="middle" class="t-small">gián đoạn, so với RTO</text></svg>
</div>
<figcaption>Hình 1. RPO đo ngược về quá khứ, RTO đo tiến về tương lai. Cả hai phải thấp hơn mức nghiệp vụ chấp nhận, và phải đo được bằng diễn tập chứ không chỉ khai báo.</figcaption>
</figure>

Mỗi loại dữ liệu có giá trị khác nhau, nên mức bảo vệ cũng khác. Các mục tiêu dưới đây là đề xuất để thảo luận với nghiệp vụ, không phải số liệu của Tới Nơi.

| Dữ liệu | RPO | RTO | Hướng bảo vệ |
| --- | --- | --- | --- |
| Ví và giao dịch (cổng giả lập VíXanh) | Gần 0 | Vài phút | Multi-AZ, replica hoặc Aurora Global ở region thứ hai |
| Bảng `trips` | Vài phút | Dưới 1 giờ | Continuous backup, point-in-time restore, copy sang region khác |
| Vị trí tài xế (6.250 toạ độ/giây) | Chấp nhận mất | Vài phút | Tái tạo sau 4 giây nên không backup |
| Ảnh chứng từ, hoá đơn | Vài phút | Vài giờ | Replication S3 (bài 36) và Object Lock |

Chi phí tăng gần như theo cấp số nhân khi RPO và RTO tiến về 0, vì phải giữ nhiều hạ tầng chạy sẵn. Vì vậy ta dành mức cao cho ví, không phải cho vị trí tài xế.

## AWS Backup: một chỗ để quản lý backup

**AWS Backup** quản lý backup tập trung cho RDS, Aurora, DynamoDB, EBS, EFS, S3 và nhiều dịch vụ khác. Các khái niệm chính:

- **Backup plan** chứa các **rule**: lịch (cron), cửa sổ chạy, vault đích, vòng đời (khi nào chuyển lớp lạnh, khi nào xoá).
- **Backup vault** là nơi chứa recovery point, có khoá mã hoá và policy riêng.
- **Selection** chọn tài nguyên, tốt nhất theo **tag**: thêm `Backup=daily` vào một database là nó tự vào kế hoạch, không phải sửa plan.
- **Copy action** sao chép recovery point sang vault khác: region khác (chống mất region) hoặc **tài khoản khác** (chống tài khoản chính bị xâm nhập). Copy cross-account cần hai tài khoản cùng AWS Organizations, bật tính năng ở management account và cho phép `backup:CopyIntoBackupVault` trên vault đích; với tài nguyên AWS Backup không quản lý hoàn toàn (như RDS, EBS), vault đích phải dùng khoá KMS customer managed và khoá của nguồn cũng phải là customer managed để chia sẻ được.
- **Continuous backup** (RDS, Aurora, S3...) cho phép point-in-time restore tới từng giây trong cửa sổ tối đa 35 ngày.

```hcl
rule {
  rule_name         = "daily-with-dr-copy"
  target_vault_name = aws_backup_vault.primary.name
  schedule          = "cron(0 17 * * ? *)"

  lifecycle {
    delete_after = 35
  }

  copy_action {
    destination_vault_arn = aws_backup_vault.dr.arn

    lifecycle {
      delete_after = 90
    }
  }
}
```

Rule này backup mỗi đêm lúc 17:00 UTC (00:00 giờ Việt Nam), giữ 35 ngày ở vault chính và copy sang vault ở `ap-northeast-1`, giữ 90 ngày. Cái giá là dung lượng lưu ở hai region cộng phí truyền dữ liệu giữa region.

**Vault Lock** biến vault thành WORM, có hai chế độ: governance (người có quyền đặc biệt vẫn gỡ được) và compliance. Ở compliance, bạn khai báo một thời gian **cooling-off** (tối thiểu 3 ngày) để còn sửa; hết thời gian này không ai, kể cả root, xoá được recovery point hay rút ngắn thời hạn giữ. Lab vì thế để Vault Lock tắt mặc định. Ngoài ra còn có **logically air-gapped vault**, một loại vault riêng có thể chia sẻ khi cần phục hồi; xem tài liệu để biết khác biệt và hỗ trợ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Tài nguyên gắn tag Backup=daily được backup plan chọn, ghi vào vault chính ở ap-southeast-1, copy sang vault ở ap-northeast-1; restore testing plan định kỳ restore thử từ vault chính."><defs><marker id="aws37-2-pk" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="aws37-2-phys" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="115" y="60" text-anchor="middle" class="t-b">RDS trips, DynamoDB, EFS</text><text x="115" y="82" text-anchor="middle" class="t-small t-mono">Backup=daily</text><text x="115" y="102" text-anchor="middle" class="t-small">chọn theo tag</text><rect x="290" y="30" width="210" height="90" rx="10" class="f-pk-soft s-pk"/><text x="395" y="60" text-anchor="middle" class="t-b">Vault chính</text><text x="395" y="82" text-anchor="middle" class="t-small">ap-southeast-1</text><text x="395" y="102" text-anchor="middle" class="t-small">35 ngày, có thể Vault Lock</text><rect x="620" y="30" width="240" height="90" rx="10" class="f-pk-soft s-pk"/><text x="740" y="60" text-anchor="middle" class="t-b">Vault DR</text><text x="740" y="82" text-anchor="middle" class="t-small">ap-northeast-1 (hoặc tài khoản khác)</text><text x="740" y="102" text-anchor="middle" class="t-small">90 ngày</text><path d="M210 75 H288" class="f-none s-phys" stroke-width="2" marker-end="url(#aws37-2-phys)"/><text x="249" y="65" text-anchor="middle" class="t-small">plan</text><path d="M500 75 H618" class="f-none s-pk" stroke-width="2" marker-end="url(#aws37-2-pk)"/><text x="559" y="65" text-anchor="middle" class="t-small">copy</text><rect x="290" y="180" width="210" height="80" rx="10" class="f-warn-soft s-warn dash"/><text x="395" y="208" text-anchor="middle" class="t-b t-warn">Restore testing</text><text x="395" y="228" text-anchor="middle" class="t-small">restore thử hằng tuần,</text><text x="395" y="244" text-anchor="middle" class="t-small">đo thời gian, rồi xoá</text><path d="M395 120 V178" class="f-none s-warn dash" stroke-width="2" marker-end="url(#aws37-2-pk)"/></svg>
</div>
<figcaption>Hình 2. Dòng chảy backup của lab. Đường đứt là kiểm chứng định kỳ: bản backup chưa từng restore thử vẫn chỉ là giả định.</figcaption>
</figure>

## Bốn chiến lược DR

Backup bảo vệ dữ liệu; **disaster recovery** còn phải dựng lại cả hệ thống ở nơi khác. Bốn chiến lược tăng dần theo chi phí và giảm dần theo RTO:

| Chiến lược | Ở region thứ hai | RPO và RTO cỡ | Cái giá |
| --- | --- | --- | --- |
| Backup and restore | Chỉ có backup được copy; hạ tầng dựng bằng IaC khi cần | Giờ | Rẻ nhất, RTO dài |
| Pilot light | Database luôn được nhân bản; compute tắt hoặc tối thiểu | Vài chục phút | Trả tiền nhân bản dữ liệu; còn phải bật compute |
| Warm standby | Cả hệ thống chạy ở quy mô nhỏ | Vài phút | Trả tiền một bản thu nhỏ chạy liên tục |
| Multi-site active/active | Hai region cùng phục vụ | Gần 0 | Đắt nhất, phức tạp nhất về ghi dữ liệu |

Region thứ hai không chỉ cần dữ liệu. Danh sách tối thiểu: hạ tầng dưới dạng code để dựng lại trong vài chục phút (bài 22 và 23), image trong ECR có replication, secret được nhân bản (Secrets Manager có replica theo region), khoá KMS dùng được ở region đó (bài 52), DNS trỏ được bằng Route 53 (failover và ARC ở bài 56), và quota đủ cho số instance cần bật. Thiếu một món là RTO thực tế dài hơn kế hoạch.

Với Tới Nơi, giả định hợp lý: ví và `trips` dùng pilot light hoặc warm standby, còn phần còn lại backup and restore. Vị trí tài xế tự khôi phục khi tài xế gửi toạ độ mới nên không đáng giữ ở region thứ hai.

## Elastic Disaster Recovery và diễn tập restore

Nếu còn máy EC2 hoặc server on-premises (như PostgreSQL tự vận hành trước khi chuyển sang Aurora ở [bài 40](/articles/aws-dms-migration-postgresql-zero-downtime)), **AWS Elastic Disaster Recovery** nhân bản liên tục ở mức block sang một vùng staging dùng instance nhỏ và ổ đĩa rẻ; khi sự cố, dịch vụ này bật các máy đầy đủ từ bản sao. Đổi lại ta trả tiền cho vùng staging và cho các bước cấu hình agent và mạng.

Bước quan trọng nhất là **diễn tập**. Backup chưa restore thử thì chưa biết nó có dùng được hay không. **Restore testing plan** của AWS Backup tự restore thử một recovery point theo lịch, đo thời gian rồi xoá. Ngoài ra, mỗi quý nên chạy một **game day**: một người chỉ huy, một người restore, một người kiểm tra dữ liệu, có đồng hồ bấm giờ.

| Việc đo | Cách làm | Ghi lại |
| --- | --- | --- |
| Thời gian restore | Script đo từ lúc `start-restore-job` tới trạng thái `COMPLETED` | Giây, kèm kích thước DB |
| Toàn vẹn | Đếm dòng, đối soát theo ngày và thành phố, checksum | Khớp hay lệch |
| Mốc dữ liệu | `max(updated_at)` trên DB restore | Khoảng thời gian dữ liệu bị mất thực tế, so với RPO |
| Bước thủ công | Runbook cho từng việc: đổi secret, đổi DNS | Ai làm, mất bao lâu |

Kết quả đo là RTO thật; nếu nó vượt mục tiêu thì phải đổi chiến lược hoặc kiến trúc, không phải đổi con số trên giấy.

## Cái giá phải trả

- **Backup chưa kiểm tra là chưa có backup.** Recovery point lỗi quyền hoặc khoá KMS chỉ lộ ra lúc cần.
- **Copy cross-region tốn tiền hai lần**: lưu ở hai nơi và truyền dữ liệu giữa region.
- **Vault Lock là quyết định một chiều.** Sau cooling-off không rút ngắn được, nên thử bằng retention ngắn trong tài khoản lab.
- **Backup không bảo vệ khỏi lỗi logic đã được backup.** Dữ liệu hỏng từ tuần trước nằm trong cả 35 bản; point-in-time restore giúp tìm mốc trước khi hỏng.
- **RTO phụ thuộc kích thước DB và quota**, không chỉ cấu hình. Restore DB lớn có thể mất hàng giờ.

**Bài tập:** Bạn có snapshot RDS tự động trong cùng region. Mất cả region `ap-southeast-1` thì snapshot đó có cứu được không?

<details><summary>Xem đáp án</summary><p>Không. Snapshot tự động nằm trong cùng region với instance nên mất region là mất cả hai. Cần copy snapshot hoặc recovery point sang region thứ hai (copy action của AWS Backup hoặc copy snapshot thủ công), đảm bảo khoá KMS dùng được ở đó, và có IaC để dựng lại mạng, instance và secret.</p></details>

## Lab: backup plan và diễn tập

Thư mục `examples/toi-noi-aws/p04-37-backup` có Terraform dựng hai vault, backup plan với copy sang `ap-northeast-1`, continuous backup, chọn theo tag, cảnh báo SNS, restore testing plan và tuỳ chọn Vault Lock (tắt mặc định), cùng `scripts/restore-drill.sh` đo thời gian restore và `scripts/verify-restore.sql` kiểm tra tính toàn vẹn.

```bash
cd examples/toi-noi-aws/p04-37-backup
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 init -backend=false -input=false
docker run --rm -v "$PWD:/w" -w /w hashicorp/terraform:1.10 validate
```

Terraform đã qua `fmt` và `validate` nhưng chưa `apply` hay restore trên tài khoản thật. Cần kiểm tra khi chạy: metadata restore của RDS, định dạng điều kiện tag của restore testing và hành vi copy với khoá KMS riêng. Tiền lab đến từ dung lượng recovery point ở hai region, truyền dữ liệu và DB instance restore thử; bước dọn nằm trong README.

Bài trước: [bài 36](/articles/aws-s3-nang-cao-storage-class-replication-object-lock). Bài tiếp: [bài 38](/articles/aws-glue-athena-redshift-lakehouse).

## Tài liệu gốc

- [AWS Backup Developer Guide](https://docs.aws.amazon.com/aws-backup/latest/devguide/whatisbackup.html).
- [AWS Backup Vault Lock](https://docs.aws.amazon.com/aws-backup/latest/devguide/vault-lock.html).
- [Restore testing trong AWS Backup](https://docs.aws.amazon.com/aws-backup/latest/devguide/restore-testing.html).
- [Disaster recovery of workloads on AWS (whitepaper)](https://docs.aws.amazon.com/whitepapers/latest/disaster-recovery-workloads-on-aws/disaster-recovery-workloads-on-aws.html).
- [AWS Elastic Disaster Recovery](https://docs.aws.amazon.com/drs/latest/userguide/what-is-drs.html).
