---
slug: azure-entra-id-tenant-user-group-app-registration
title: "Microsoft Entra ID: tenant, user, group và app registration"
summary: "Bài 11: phân biệt Entra ID với Azure RBAC, đặt quyền cho group thay vì user, đăng ký ứng dụng api-toinoi-trips của Tới Nơi với scope Trips.Read rồi lấy token và giải mã các claim aud, iss, scp, roles."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, entra-id, identity, oauth]
language: vi
---

Đây là **bài 11** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 2 · Danh tính và bảo mật (bài 1/9 của phần).

Phần 2 gồm 9 bài (11 đến 19) với hai mục tiêu cụ thể: không còn mật khẩu nằm trong cấu hình, và không còn database mở ra Internet. Mọi thứ bắt đầu từ câu hỏi nền tảng nhất: ai đang gọi? Bài này dựng nền danh tính của Tới Nơi bằng Microsoft Entra ID và kết thúc bằng một lab đăng ký API rồi giải mã token. *Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.*

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Bạn là Global Administrator của tenant. Vậy bạn có tự động đọc được mọi resource trong subscription của tenant đó không?</p>
<details><summary>Xem đáp án</summary><p>Không. Global Administrator quản lý danh tính, không mặc định có quyền trên resource Azure. Quyền trên resource đến từ Azure RBAC (bài 12), dù Global Administrator có thể tự nâng quyền (elevate access) thành User Access Administrator ở root scope để lấy nó.</p></details>
</div>

## Entra ID trả lời «bạn là ai», không trả lời «bạn được làm gì»

Một **tenant** là một thực thể Entra ID riêng của tổ chức: nó giữ danh sách user, group, ứng dụng và phát hành token. Tenant của Tới Nơi chứa nhân viên, đội platform và các ứng dụng nội bộ. Mỗi subscription Azure tin đúng một tenant. Khi bạn chạy `az login`, Entra xác thực bạn và cấp token; Azure Resource Manager nhận token đó rồi mới tra bảng RBAC xem bạn được làm gì.

Từ đó có ba ranh giới hay bị trộn lẫn:

- **Entra ID**: xác thực, quản lý danh tính, phát token. Entra ID trả lời câu hỏi «đây là ai».
- **Azure RBAC**: quyết định principal đó làm được gì trên resource nào. Đây là bài 12.
- **Entra External ID**: tenant dành cho khách hàng và tài xế của ứng dụng, tách khỏi tenant nhân viên. Đây là bài 19.

Hệ quả thực tế: nhân viên đăng nhập portal bằng tenant nhân viên, còn khách đặt xe trên app không bao giờ nên là user trong tenant đó. Trộn hai loại này làm danh sách nhân viên đầy người lạ, và chính sách bảo mật cho nhân viên bị áp lên khách hoặc ngược lại.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Tenant nhân viên cấp token, Azure RBAC trên subscription quyết định quyền; khách hàng nằm ở tenant External ID riêng."><defs><marker id="az11-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Tenant nhân viên</text><text x="34" y="123" class="t-small">User, group, app, token</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az11-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-pk-soft s-pk"/><text x="319" y="95" class="t-b">Subscription + RBAC</text><text x="319" y="123" class="t-small">Quyết định được làm gì</text><rect x="590" y="65" width="260" height="85" rx="10" class="f-warn-soft s-warn"/><text x="604" y="95" class="t-b">External ID (bài 19)</text><text x="604" y="123" class="t-small">Khách và tài xế, tách riêng</text></svg>
</div>
<figcaption>Hình 1. Tenant nhân viên cấp token, Azure RBAC trên subscription quyết định quyền; khách hàng nằm ở tenant External ID riêng.</figcaption>
</figure>

## Quyền đặt cho group, người đến và đi thì sửa thành viên

Entra có ba loại danh tính người cần phân biệt:

- **Member user**: nhân viên có tài khoản trong tenant.
- **Guest (B2B)**: người ngoài tổ chức được mời, đăng nhập bằng danh tính của họ nhưng xuất hiện trong tenant dưới dạng guest. Ví dụ một nhà thầu ngoài giúp đội platform vài tuần. Guest không phải khách hàng của ứng dụng Tới Nơi.
- **Group**: tập hợp các principal để cấp quyền một lần cho nhiều người.

Group có vài loại, chọn theo mục đích:

| Loại group | Cách xác định thành viên | Dùng khi nào |
| --- | --- | --- |
| Security group (assigned) | Admin thêm tay | Cấp quyền RBAC cho đội, như `grp-toinoi-platform` |
| Security group (dynamic) | Rule trên thuộc tính user | Tự động theo phòng ban hay vị trí; cần license phù hợp, kiểm tra tài liệu |
| Microsoft 365 group | Admin thêm tay, rule dynamic, hoặc user tự tham gia nếu group mở | Cộng tác (hộp thư, kênh chung); không dùng cho phân quyền hạ tầng |

Quy tắc dễ giữ nhất: **không gán quyền cho từng user, chỉ gán cho group**. Khi một kỹ sư chuyển đội, bạn sửa thành viên một chỗ thay vì lần theo hàng chục role assignment rải rác. Danh sách assignment cũng ngắn và dễ review hơn nhiều.

Hai lớp bảo vệ nên có từ ngày đầu. **MFA** bắt user chứng minh thêm một yếu tố ngoài mật khẩu. **Conditional Access** là rule dạng «nếu user/ứng dụng/vị trí/rủi ro thoả điều kiện thì yêu cầu MFA, yêu cầu thiết bị tuân thủ hoặc chặn». Tenant mới thường có sẵn security defaults ở mức cơ bản; Conditional Access đầy đủ cần license riêng.

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Một policy Conditional Access viết sai (ví dụ chặn mọi người, kể cả admin) có thể khoá cả tenant. Luôn giữ hai tài khoản emergency access (break-glass) loại trừ khỏi policy, mật khẩu dài cất ngoại tuyến, và theo dõi mọi lần chúng đăng nhập. Dùng chế độ report-only để xem tác động trước khi bật policy thật.</p>
</div>

## App registration và enterprise application là hai mặt của một ứng dụng

Khi một ứng dụng cần nhận hoặc kiểm tra token, ứng dụng đó phải được Entra biết đến. Có hai object, hay bị gọi nhầm là một:

- **App registration** là bản định nghĩa ứng dụng, sống ở home tenant: client ID, redirect URI, scope mà API mở ra, app role, credential. Mỗi ứng dụng có đúng một registration.
- **Enterprise application** (service principal) là bản thể hiện của ứng dụng trong một tenant cụ thể. Đây là nơi gán user/group được phép dùng, ghi nhận consent và áp Conditional Access. Ứng dụng đa tenant có một registration nhưng mỗi tenant có một service principal.

Với Tới Nơi, `trip-service` là một API nên ta đăng ký nó thành `api-toinoi-trips` và để nó tự khai báo scope `Trips.Read`. Service Booking đóng vai client gọi API này. Portal tạo cả hai object cùng lúc; `az ad app create` chỉ tạo registration, bạn phải chạy thêm `az ad sp create`.

Client chứng minh danh tính với Entra bằng một credential. Ba loại có cái giá khác nhau:

| Credential | Bản chất | Rủi ro và cách dùng |
| --- | --- | --- |
| Client secret | Chuỗi bí mật có hạn dùng | Phải lưu ở đâu đó, phải xoay; lộ là mất. Chỉ cho lab hoặc khi không còn đường khác |
| Certificate | Cặp khoá, client ký assertion | Khoá riêng nên nằm trong Key Vault (bài 14); vẫn phải gia hạn |
| Federated credential | Tin token của issuer ngoài | Không có bí mật nào để lưu; hợp với GitHub Actions OIDC hay workload chạy trên Kubernetes |

Chạy trên Azure thì còn một lựa chọn tốt hơn cả ba: managed identity, nội dung bài 13. Đích của Phần 2 là bỏ client secret khỏi cấu hình, nên hãy coi secret là lựa chọn cuối.

## Token: audience và scope quyết định API có chấp nhận hay không

Entra phát hai loại token đáng nhớ. **ID token** nói cho client biết user là ai sau khi đăng nhập; audience của nó là chính client. **Access token** dùng để gọi API; audience của nó là API đó. Client không nên đọc nội dung bên trong access token của API khác, vì định dạng đó không thuộc hợp đồng với client.

Quyền trong access token xuất hiện ở hai dạng:

- **`scp`** (scope): quyền uỷ quyền đại diện cho user đã đăng nhập, ví dụ `Trips.Read`. Chỉ có khi có user.
- **`roles`** (app role): quyền gán cho chính ứng dụng, dùng khi không có user (service gọi service).

Một JWT gồm ba đoạn base64url nối bằng dấu chấm; đoạn giữa là payload JSON. API phải kiểm tra chữ ký bằng khoá công khai của tenant, rồi kiểm tra các claim sau:

| Claim | Ý nghĩa | API phải kiểm gì |
| --- | --- | --- |
| `iss` | Ai phát token | Đúng tenant của bạn, đúng phiên bản endpoint |
| `aud` | Token dành cho ai | Đúng API của bạn; token cho API khác phải bị từ chối |
| `scp` | Scope đã được uỷ quyền | Có scope cần cho thao tác |
| `roles` | App role của ứng dụng gọi | Có role cần cho thao tác app-only |
| `exp` | Hết hạn lúc nào | Chưa hết hạn |
| `tid`, `oid` | Tenant và object ID của principal | Dùng để định danh và audit, không dùng tên hiển thị |

Giải mã payload không đồng nghĩa với xác minh token. Ai cũng đọc được claim; chỉ chữ ký mới cho biết claim có thật. Cách kiểm tra trong ASP.NET Core nằm ở [bài 19 của loạt C#](/articles/dotnet10-authentication-authorization).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 2. Client xin token từ Entra cho audience api-toinoi-trips, API kiểm iss, aud, scp trước khi nhận; sai audience hoặc thiếu scope bị từ chối."><defs><marker id="az11-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Client</text><text x="34" y="123" class="t-small">Xin token cho API</text><path d="M200 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az11-1-a)"/><rect x="240" y="65" width="180" height="85" rx="10" class="f-phys-soft s-phys"/><text x="254" y="95" class="t-b">Entra ID</text><text x="254" y="123" class="t-small">Ký và phát token</text><path d="M420 108 L460 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az11-1-a)"/><rect x="460" y="65" width="180" height="85" rx="10" class="f-pk-soft s-pk"/><text x="474" y="95" class="t-b">api-toinoi-trips</text><text x="474" y="123" class="t-small">Kiểm iss, aud, scp</text><path d="M640 108 L680 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az11-1-a)"/><rect x="680" y="65" width="180" height="85" rx="10" class="f-warn-soft s-warn"/><text x="694" y="95" class="t-b">401 hoặc 403</text><text x="694" y="123" class="t-small">Sai aud, thiếu scope</text></svg>
</div>
<figcaption>Hình 2. Client xin token từ Entra cho audience api-toinoi-trips, API kiểm iss, aud, scp trước khi nhận; sai audience hoặc thiếu scope bị từ chối.</figcaption>
</figure>

## Lab: group, app registration và scope Trips.Read

Lab này tạo group, đăng ký API rồi mở một scope. Khối đầu chỉ dựng object; khối này không chứng minh bất kỳ client nào đã được phép gọi API.

```bash
az login --tenant <tenant-id>

az ad group create \
  --display-name grp-toinoi-platform \
  --mail-nickname grp-toinoi-platform
az ad group member add \
  --group grp-toinoi-platform \
  --member-id <user-object-id>

az ad app create \
  --display-name api-toinoi-trips \
  --sign-in-audience AzureADMyOrg \
  --query "{appId:appId, objectId:id}" -o json
az ad sp create --id <app-id>
```

Tiếp theo là mở scope. Tạo `api.json` với một GUID mới (chạy `uuidgen`) rồi PATCH vào object ứng dụng qua Microsoft Graph. Đặt `requestedAccessTokenVersion` bằng 2 để Entra phát access token phiên bản 2.0 cho API này. Thuộc tính này nằm ở phía API, nên client gọi endpoint nào thì định dạng token vẫn do thuộc tính này quyết định.

```json
{
  "identifierUris": ["api://<app-id>"],
  "api": {
    "requestedAccessTokenVersion": 2,
    "oauth2PermissionScopes": [{
      "id": "<scope-guid>",
      "value": "Trips.Read",
      "type": "User",
      "isEnabled": true,
      "adminConsentDisplayName": "Read Toi Noi trips",
      "adminConsentDescription": "Allows reading trips on behalf of the user.",
      "userConsentDisplayName": "Read your trips",
      "userConsentDescription": "Allows the app to read trips for you."
    }]
  }
}
```

```bash
az rest --method PATCH \
  --uri "https://graph.microsoft.com/v1.0/applications/<object-id>" \
  --headers "Content-Type=application/json" \
  --body @api.json
```

Lệnh `az rest` tự lấy token Microsoft Graph từ phiên `az login` và tự đặt header JSON khi body là JSON; user đăng nhập cần quyền sửa ứng dụng này (là owner của nó, hoặc có role như Application Administrator). Khối trên chỉ khai báo scope; nó chưa cho client nào quyền xin scope đó.

Azure CLI là một ứng dụng first-party của Microsoft, client ID `04b07795-8ddb-461a-bbee-02f9e1bf7b46`. Mặc định Azure CLI chưa được consent cho scope của API bạn vừa tạo, nên bước lấy token sẽ báo lỗi consent (AADSTS65001). Cách gọn cho lab là khai báo Azure CLI là client được uỷ quyền trước, trong một lần PATCH thứ hai sau khi scope đã tồn tại. Gửi lại đủ khối `api` (cả `requestedAccessTokenVersion` và `oauth2PermissionScopes` như trên) và thêm:

```json
"preAuthorizedApplications": [{
  "appId": "04b07795-8ddb-461a-bbee-02f9e1bf7b46",
  "delegatedPermissionIds": ["<scope-guid>"]
}]
```

`delegatedPermissionIds` phải trùng `id` của scope `Trips.Read`. Uỷ quyền trước nghĩa là user không còn thấy màn hình consent khi Azure CLI xin scope này; chỉ làm vậy với client bạn tin.

## Lấy token và giải mã claim

Lệnh dưới lấy access token cho scope `Trips.Read` rồi in vài claim của payload. `--scope` nhận scope dạng v2.0 (App ID URI cộng tên scope); nếu CLI báo cần tương tác, chạy lại `az login --scope "api://<app-id>/Trips.Read"`. Phần `awk` bù ký tự `=` vì base64url bỏ padding. Khối này chỉ đọc claim; nó không kiểm chữ ký.

```bash
TOKEN=$(az account get-access-token \
  --scope "api://<app-id>/Trips.Read" \
  --query accessToken -o tsv)

echo "$TOKEN" | cut -d. -f2 | tr '_-' '/+' \
  | awk '{ while (length($0) % 4) $0 = $0 "="; print }' \
  | base64 -d | jq '{aud, iss, scp, roles, tid, oid, ver}'
```

Kết quả kỳ vọng với token do user đăng nhập (không có `roles`, giá trị là placeholder):

```json
{
  "aud": "<app-id>",
  "iss": "https://login.microsoftonline.com/<tenant-id>/v2.0",
  "scp": "Trips.Read",
  "roles": null,
  "tid": "<tenant-id>",
  "oid": "<user-object-id>",
  "ver": "2.0"
}
```

Với token phiên bản 2.0, `aud` luôn là client ID của API và `iss` có hậu tố `/v2.0`; với phiên bản 1.0, `aud` thường là App ID URI dạng `api://<app-id>` và `iss` có dạng `https://sts.windows.net/<tenant-id>/`. API kiểm tra theo sai phiên bản sẽ từ chối token hợp lệ, nên bước PATCH ở trên đặt phiên bản rõ ràng. Nếu service Booking gọi bằng client credentials, token sẽ không có `scp` mà có `roles` (khi bạn đã định nghĩa app role và gán cho nó).

App registration và group không tính tiền, nhưng đừng để object lab nằm lại trong tenant. Dọn bằng `az ad app delete --id <app-id>` (service principal đi kèm bị xoá theo, registration nằm trong mục deleted items một thời gian trước khi mất hẳn) và `az ad group delete --group grp-toinoi-platform`.

**Bài tập:** service Booking gọi `api-toinoi-trips` bằng client credentials, không có user. API của bạn chỉ kiểm `aud` và `scp`. Chuyện gì xảy ra, và kiểm tra sai theo hướng ngược lại (chỉ kiểm tra `aud`) nguy hiểm ở đâu?

<details><summary>Xem lời giải</summary><p>Token app-only không có `scp` nên API từ chối mọi lời gọi của Booking, một lỗi theo hướng an toàn nhưng làm hỏng tính năng. Nếu API chỉ kiểm `aud` thì mọi ứng dụng trong tenant xin được token cho API này đều qua được, kể cả ứng dụng không liên quan. API cần kiểm `scp` cho luồng có user và `roles` cho luồng app-only, và nên bắt buộc gán app role cho client được phép.</p></details>

## Học tiếp từ đây

Bạn đã có group `grp-toinoi-platform`, registration `api-toinoi-trips` với scope `Trips.Read`, và biết đọc `aud`, `iss`, `scp`, `roles` trong một token. Nhưng token mới chứng minh danh tính; nó chưa cho ai quyền tạo hay xoá resource. Quyền đó là việc của Azure RBAC trong [Bài 12](/articles/azure-rbac-role-scope-assignment-pim), nơi các group như thế này nhận role ở đúng scope và quyền Owner production chuyển sang dạng just-in-time.

## Tài liệu gốc

- [Microsoft Entra ID documentation](https://learn.microsoft.com/en-us/entra/identity/)
- [Microsoft identity platform documentation](https://learn.microsoft.com/en-us/entra/identity-platform/)
- [Application and service principal objects in Microsoft Entra ID](https://learn.microsoft.com/en-us/entra/identity-platform/app-objects-and-service-principals)
- [Access tokens in the Microsoft identity platform](https://learn.microsoft.com/en-us/entra/identity-platform/access-tokens)
- [Conditional Access overview](https://learn.microsoft.com/en-us/entra/identity/conditional-access/overview)
- [Microsoft Entra External ID documentation](https://learn.microsoft.com/en-us/entra/external-id/)
