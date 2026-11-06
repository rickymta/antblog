---
slug: azure-portal-cli-cloud-shell-resource-group-dau-tien
title: "Azure Portal, Azure CLI và Cloud Shell: tạo resource group đầu tiên"
summary: "Bài 3: dùng Portal để khám phá, Azure CLI để lặp lại được và Cloud Shell khi chưa cài gì. Tạo, lọc, xem rồi xoá resource group rg-toinoi-lab-dev-sea của Tới Nơi bằng az và JMESPath."
category: devops-azure
level: beginner
tags: [azure, cloud, devops, toi-noi, azure-cli, resource-group, cloud-shell]
language: vi
---

Đây là **bài 3** của loạt **Azure từ cơ bản tới nâng cao qua [Tới Nơi](/articles/du-an-mau-toi-noi)** · Phần 1 · Nền tảng Azure (bài 3/10 của phần).

Bài này đưa bạn từ "đã có subscription" tới "đã tạo, lọc và xoá một resource group bằng dòng lệnh". Lệnh và cấu hình trong bài được soạn theo tài liệu Microsoft Learn tại thời điểm viết (11/2026); hãy chạy trên subscription của bạn và xoá resource sau lab.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Tạo resource group trên Portal và tạo bằng lệnh <code>az group create</code> có tạo ra hai loại resource group khác nhau không?</p>
<details><summary>Xem đáp án</summary><p>Không. Cả hai cùng gửi một request tới Azure Resource Manager. Kết quả là cùng một loại resource, chịu cùng quyền, cùng chính sách, và thao tác tạo được ghi vào Activity Log như nhau.</p></details>
</div>

## Portal, CLI và Cloud Shell cùng gõ vào một cánh cửa

Azure có ba cách thao tác phổ biến. **Azure Portal** là giao diện web: bạn bấm chuột, thấy ngay cấu trúc, rất hợp để khám phá một dịch vụ lạ. **Azure CLI** là lệnh `az`: bạn gõ một lần, lưu vào file, chạy lại giống hệt vào tuần sau. **Cloud Shell** là một terminal chạy ngay trong trình duyệt, đã cài sẵn `az` và đã đăng nhập bằng tài khoản bạn đang dùng trên Portal.

Còn có module PowerShell Az làm cùng việc với cú pháp khác. Loạt bài này chỉ dùng `az`, vì lệnh `az` copy được vào tài liệu, vào pipeline và vào bài viết mà không phải đổi cú pháp.

Chọn công cụ theo việc cần làm, không theo sở thích:

| Công cụ | Mạnh ở | Yếu ở | Dùng khi |
| --- | --- | --- | --- |
| Portal | Nhìn thấy quan hệ giữa resource, form có gợi ý | Khó lặp lại, khó review, dễ quên đã bấm gì | Khám phá dịch vụ mới, xem log, kiểm tra nhanh |
| Azure CLI | Lặp lại được, lọc bằng `--query`, đưa vào script | Phải cài và đăng nhập trên máy bạn | Mọi thao tác bạn muốn làm lần thứ hai |
| Cloud Shell | Không cần cài, đăng nhập sẵn | Phiên hết hạn khi không dùng, cần storage để giữ file | Máy lạ, hướng dẫn nhanh, sửa gấp |

Cả ba đều đi qua cùng một đường. Hình 1 cho thấy lý do: Portal và CLI không "nói chuyện" trực tiếp với từng dịch vụ, chúng gửi request REST tới Azure Resource Manager.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Portal, Azure CLI và Cloud Shell đều gửi request tới cùng ARM REST API, nơi xác thực, kiểm quyền và ghi Activity Log."><defs><marker id="az03-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="15" width="250" height="75" rx="10" class="f-pk-soft s-pk"/><text x="34" y="46" class="t-b">Azure Portal</text><text x="34" y="72" class="t-small">Khám phá, bấm chuột</text><rect x="20" y="112" width="250" height="75" rx="10" class="f-pk-soft s-pk"/><text x="34" y="143" class="t-b">Azure CLI</text><text x="34" y="169" class="t-small">Lệnh az, lặp lại được</text><rect x="20" y="209" width="250" height="75" rx="10" class="f-pk-soft s-pk"/><text x="34" y="240" class="t-b">Cloud Shell</text><text x="34" y="266" class="t-small">az chạy trong trình duyệt</text><path d="M270 52 L335 130" class="f-none s-phys" stroke-width="2" marker-end="url(#az03-0-a)"/><path d="M270 150 L335 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az03-0-a)"/><path d="M270 247 L335 170" class="f-none s-phys" stroke-width="2" marker-end="url(#az03-0-a)"/><rect x="340" y="112" width="240" height="75" rx="10" class="f-phys-soft s-phys"/><text x="354" y="143" class="t-b">ARM REST API</text><text x="354" y="169" class="t-small">management.azure.com</text><path d="M580 150 L615 150" class="f-none s-phys" stroke-width="2" marker-end="url(#az03-0-a)"/><rect x="620" y="112" width="240" height="75" rx="10" class="f-phys-soft s-phys"/><text x="634" y="143" class="t-b">Xác thực + RBAC</text><text x="634" y="169" class="t-small">Activity Log ghi lại</text></svg>
</div>
<figcaption>Hình 1. Portal, Azure CLI và Cloud Shell đều gửi request tới cùng ARM REST API, nơi xác thực, kiểm quyền và ghi Activity Log.</figcaption>
</figure>

Hệ quả thực tế: quyền của bạn không phụ thuộc công cụ. Nếu role của bạn cho phép tạo resource group trên Portal thì `az` cũng tạo được, và nếu bị từ chối thì cả hai cùng bị từ chối. Activity Log cũng ghi thao tác ghi (tạo, sửa, xoá) của cả hai theo cùng một dạng, nên không có chuyện "làm bằng tay thì không để lại dấu". Thao tác chỉ đọc như `az group list` thì không vào Activity Log.

Cloud Shell có một điều cần biết từ đầu: lần đầu mở, nó hỏi bạn chọn phiên tạm (không giữ file, mất khi phiên kết thúc) hay gắn một storage account để giữ thư mục home trong một file share. Bản thân Cloud Shell không tính phí, nhưng storage account là một resource thật, tính tiền theo mức dùng. Với lab ngắn, phiên tạm là đủ; nếu chọn gắn storage, hãy biết nó nằm ở resource group nào để còn dọn.

## Đăng nhập, chọn đúng subscription rồi mới tạo gì đó

Lệnh đầu tiên luôn là đăng nhập. Trên máy cá nhân, `az login` mở trình duyệt để bạn xác thực bằng Microsoft Entra ID. Nếu máy không có trình duyệt, dùng `--use-device-code` và nhập mã trên máy khác. Đăng nhập kiểu này là **danh tính của một người**: mọi lệnh sau đó chạy với quyền của bạn. Pipeline và ứng dụng thì không dùng cách này; chúng dùng service principal và managed identity, chủ đề của Phần 2 và Phần 5.

Một tài khoản có thể thấy nhiều subscription, và lệnh `az` chạy trên subscription đang là mặc định. Đây là nguồn nhầm lẫn phổ biến nhất của người mới: tạo resource ở một subscription không định tạo. Hãy xem và chọn rõ ràng.

```bash
az login
az account list \
  --query "[].{Name:name, Id:id, Default:isDefault}" \
  --output table

az account set --subscription "<subscription-id>"
az account show \
  --query "{Name:name, Id:id, Tenant:tenantId}" \
  --output table
```

Khối này đăng nhập, liệt kê subscription bạn thấy được, đặt một subscription làm mặc định rồi in lại để kiểm. Nó không chứng minh bạn có quyền tạo resource trong subscription đó; việc ấy chỉ biết khi lệnh tạo chạy. Nếu bạn thuộc nhiều tenant, thêm `--tenant <tenant-id>` vào `az login`.

Để bớt gõ lặp lại, `az configure` lưu giá trị mặc định cho các tham số hay dùng như `group` và `location`.

```bash
az configure --defaults \
  group=rg-toinoi-lab-dev-sea \
  location=southeastasia
az configure --list-defaults --output table

az config unset defaults.group
az config unset defaults.location
```

Hai dòng cuối gỡ default sau lab. Việc gỡ quan trọng hơn bạn nghĩ: default là **trạng thái ẩn** nằm trong file cấu hình trên máy, không hiện trong lệnh. Một tuần sau, bạn gõ `az vm create` mà quên `--resource-group`, và lệnh vẫn chạy, vào đúng resource group bạn đã quên. Với tài liệu và review, luôn ghi rõ tham số thay vì dựa vào default.

## Tạo resource group với tag ngay từ đầu

Tới Nơi dùng **Southeast Asia** (Singapore) làm region chính, mã trong CLI là `southeastasia`. Tên resource group theo quy ước `rg-toinoi-<workload>-<env>-sea`; ở lab này workload là `lab`, môi trường `dev`. Tag `env`, `owner` và `cost-center` gắn ngay lúc tạo để sau này lọc chi phí được (bài 4 và bài 5 quay lại).

```bash
az group create \
  --name rg-toinoi-lab-dev-sea \
  --location southeastasia \
  --tags env=dev owner=platform cost-center=toinoi \
  --output table

az group list --output table
az group show --name rg-toinoi-lab-dev-sea --output json
```

Khối này tạo resource group, liệt kê tất cả resource group của subscription rồi in đầy đủ một cái dưới dạng JSON. Nó chưa tạo resource nào bên trong, nên chưa phát sinh tiền cho tới khi bạn thêm resource.

Có hai điểm cơ chế nên nhớ. Thứ nhất, **location của resource group chỉ là nơi lưu metadata** của nhóm đó. Resource bên trong có thể nằm ở region khác. Nhưng Tới Nơi vẫn đặt resource group cùng region với phần lớn resource để dễ đọc và dễ phân tích sự cố. Thứ hai, location của resource group không đổi được sau khi tạo; chạy lại `az group create` cùng tên với location khác sẽ báo lỗi, vì request là PUT lên cùng một resource.

## --output và --query: ba định dạng cho ba người đọc

Mặc định `az` in JSON, rất đầy đủ nhưng dài. Bạn chọn định dạng bằng `--output` (viết tắt `-o`):

| Định dạng | Dành cho | Đặc điểm |
| --- | --- | --- |
| `table` | Người đọc trên terminal | Chỉ hiện các cột đơn giản, không hiện object lồng nhau |
| `json` | Xem đủ dữ liệu, đưa vào công cụ khác | Đầy đủ, giữ nguyên cấu trúc |
| `tsv` | Script shell | Không có tiêu đề, không có dấu nháy, dễ gán vào biến |

`--query` nhận biểu thức **JMESPath** chạy trên JSON trả về, trước khi chọn định dạng. `[?điều_kiện]` lọc phần tử của mảng, `.{Tên:thuộc_tính}` chọn và đặt lại tên cột. Vì `table` bỏ qua object lồng nhau, ta thường kết hợp: dùng `--query` để "làm phẳng" rồi `--output table` để in.

```bash
az group list \
  --query "[?tags.env=='dev'].{Name:name, Region:location, State:properties.provisioningState, Cost:tags.\"cost-center\"}" \
  --output table

SUB_ID=$(az account show --query id --output tsv)
echo "$SUB_ID"
```

Truy vấn đầu lọc resource group có tag `env=dev` và in bốn cột. Tag `cost-center` có dấu gạch nối nên tên phải đặt trong dấu nháy kép, và ta phải escape chúng bên trong chuỗi bash. Chuỗi cố định trong JMESPath dùng dấu nháy đơn. Phần sau lấy subscription ID ở dạng tsv để gán vào biến. Khối này không kiểm tra bạn có tag viết sai chính tả; lọc theo `tags.env` bỏ sót resource group nào gõ nhầm thành `Env` hay `environment`, một lý do để bắt buộc tag bằng policy ở bài sau.

## Mỗi lệnh az là một request tới ARM

Ta đã nói Portal và CLI chung đường đi. Có thể thấy điều đó bằng mắt: `--debug` in ra từng HTTP request mà `az` gửi, và `az rest` cho bạn gửi thẳng request mà không qua lệnh có sẵn.

```bash
az group show --name rg-toinoi-lab-dev-sea --debug 2>&1 | head -n 20

az rest --method get \
  --url "https://management.azure.com/subscriptions/<subscription-id>/resourcegroups/rg-toinoi-lab-dev-sea?api-version=2021-04-01"
```

Lệnh đầu in phần đầu của log debug; dòng request có URL tới `management.azure.com` và `api-version` có thể nằm sâu hơn 20 dòng, nên bỏ `head` hoặc lọc bằng `grep management.azure.com` nếu chưa thấy. Lệnh thứ hai lấy cùng resource group bằng một GET thuần. Nhìn vào log không chứng minh mọi dịch vụ đều dùng API version giống nhau; mỗi resource provider có phiên bản riêng, nội dung bài 4. Hãy cẩn thận khi dán log debug lên chat hay issue, vì nó chứa subscription ID, tên resource và có thể lộ thông tin nhạy cảm khác.

Hiểu điều này giúp bạn xử lý hai tình huống. Khi một thao tác Portal lỗi mơ hồ, chạy lại bằng CLI với `--debug` thường cho thông báo rõ hơn. Khi cần tự động hoá một việc mà `az` chưa có lệnh tiện, `az rest` là lối thoát chính thống.

## Lab: tạo, liệt kê, rồi xoá, và vì sao xoá là việc nặng

Lab gồm ba bước bạn đã gặp: tạo `rg-toinoi-lab-dev-sea` với ba tag, liệt kê bằng `--query`, rồi xoá. Hình 2 mô tả vòng đời và chỗ nguy hiểm duy nhất của nó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Vòng đời resource group trong lab: tạo với tag, dùng, rồi xoá kéo theo mọi resource bên trong."><defs><marker id="az03-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="185" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">az group create</text><text x="34" y="123" class="t-small">Kèm 3 tag</text><path d="M205 108 L240 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az03-1-a)"/><rect x="245" y="65" width="185" height="85" rx="10" class="f-pk-soft s-pk"/><text x="259" y="95" class="t-b">Resource group</text><text x="259" y="123" class="t-small">Chứa resource</text><path d="M430 108 L465 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az03-1-a)"/><rect x="470" y="65" width="185" height="85" rx="10" class="f-warn-soft s-warn"/><text x="484" y="95" class="t-b">az group delete</text><text x="484" y="123" class="t-small">Không hỏi lại với --yes</text><path d="M655 108 L690 108" class="f-none s-phys" stroke-width="2" marker-end="url(#az03-1-a)"/><rect x="695" y="65" width="170" height="85" rx="10" class="f-surface s-dead"/><text x="709" y="95" class="t-b t-dead">Mọi resource</text><text x="709" y="123" class="t-small">Bị xoá theo</text></svg>
</div>
<figcaption>Hình 2. Vòng đời resource group trong lab: tạo với tag, dùng, rồi xoá kéo theo mọi resource bên trong.</figcaption>
</figure>

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Chỗ phải trả giá</span>
<p>Xoá resource group là xoá <strong>toàn bộ</strong> resource bên trong, kể cả database. Cờ <code>--yes</code> bỏ qua câu hỏi xác nhận, <code>--no-wait</code> trả quyền điều khiển về ngay trong khi việc xoá vẫn chạy phía sau. Trước khi xoá, đọc lại tên group và chạy <code>az resource list --resource-group &lt;tên&gt; --output table</code>. Lỗi gõ tên group trên subscription thật có thể mất dữ liệu mà không có đường quay lại.</p>
</div>

```bash
az resource list \
  --resource-group rg-toinoi-lab-dev-sea \
  --output table

az group delete --name rg-toinoi-lab-dev-sea --yes --no-wait
az group exists --name rg-toinoi-lab-dev-sea
```

Khối này liệt kê những gì sắp mất (lab trống sẽ không có dòng nào), ra lệnh xoá không chờ rồi kiểm tra xem group còn tồn tại không. `az group exists` có thể vẫn trả `true` trong chốc lát vì xoá chạy bất đồng bộ; lặp lại lệnh đến khi nhận `false`. Việc xoá cũng không chạy được nếu bên trong có **lock** (bài 4). Xoá xong thì nói chung không hoàn tác được; chỉ vài dịch vụ có soft delete riêng (như Key Vault) và đó không phải lưới an toàn cho cả resource group.

**Bài tập:** bạn chạy `az group create` hai lần với cùng tên và cùng location nhưng ở lần hai bỏ hết `--tags`. Resource group có còn ba tag của lần đầu không? Làm sao kiểm chứng mà không cần đoán?

<details><summary>Xem lời giải</summary><p>Lần hai không báo lỗi, vì resource group đã có cùng location. Nhưng <code>az group create</code> gửi một PUT mô tả toàn bộ trạng thái mong muốn, và request này không mang tag nào, nên hãy dự tính tag cũ bị thay bằng «không có tag» chứ không được giữ lại (cùng cơ chế khiến một template Bicep thiếu khối <code>tags</code> xoá tag đang có). Kiểm chứng bằng <code>az group show --name rg-toinoi-lab-dev-sea --query tags</code>. Muốn thêm hoặc sửa tag mà không đụng tag khác, dùng <code>az tag update --resource-id &lt;id-của-group&gt; --operation Merge --tags ...</code> thay vì chạy lại lệnh tạo.</p></details>

## Học tiếp từ đây

Bạn đã đăng nhập bằng danh tính người dùng, chọn đúng subscription, tạo resource group có tag, lọc bằng JMESPath, thấy request thật tới ARM và biết xoá là thao tác nặng. Những lệnh này là nền cho mọi lab về sau.

Bài tiếp theo là [Bài 4](/articles/azure-resource-manager-resource-id-tag-lock): nhìn vào bên trong ARM, đọc cấu trúc resource ID, hiểu resource provider, tag, lock và bảng quy ước đặt tên của Tới Nơi. Bài trước nói về tenant, subscription và resource group là [Bài 2](/articles/azure-tenant-subscription-resource-group).

## Tài liệu gốc

- [Azure CLI documentation](https://learn.microsoft.com/en-us/cli/azure/)
- [az group command reference](https://learn.microsoft.com/en-us/cli/azure/group)
- [Query Azure CLI command output using JMESPath](https://learn.microsoft.com/en-us/cli/azure/query-azure-cli)
- [Azure Cloud Shell overview](https://learn.microsoft.com/en-us/azure/cloud-shell/overview)
- [Manage Azure resource groups](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/manage-resource-groups-cli)
