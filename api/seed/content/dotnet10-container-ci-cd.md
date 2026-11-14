---
slug: dotnet10-container-ci-cd
title: "Container và CI/CD: build, test và publish service .NET 10"
summary: "Bài 35: Dockerfile nhiều stage, image non-root và publish OCI archive không cần Docker daemon; CI kiểm tra với PostgreSQL thật rồi lưu artifact, chưa tự deploy cloud."
category: csharp-advanced
level: advanced
tags: [csharp, dotnet, dotnet-10, toi-noi, architecture]
language: vi
---

Đây là **bài 35** của loạt **C# và .NET 10 qua [Tới Nơi](/articles/du-an-mau-toi-noi)**. Bài 31–38 cùng giải thích code cuối của service tại `examples/toi-noi-dotnet/31-38-trip-service`, target `net10.0`.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>dotnet publish thành công có chứng minh container đã chạy qua health check không?</p>
<details><summary>Xem đáp án</summary><p>Không. Publish/image archive chứng minh đóng gói; cần chạy container và kiểm tra cấu hình, network và probe ở môi trường đích.</p></details>
</div>

## Build một lần, đưa cùng artifact đi qua các bước

Một release cần truy vết được source, kết quả test, image digest và cấu hình môi trường. Không rebuild một image khác sau khi đã test staging rồi coi hai artifact là một chỉ vì có cùng tag latest.

Dockerfile có build stage dùng SDK và runtime stage chỉ nhận publish output. Runtime dùng ASP.NET image, port 8080 và user non-root do image .NET cung cấp. Không chép connection string hoặc token từ máy phát triển vào layer.

Tag 10.0 là dòng image phục vụ lab, có thể nhận bản cập nhật patch. Một release cụ thể nên ghi digest của base/output và quy trình cập nhật. Chạy non-root giảm một loại rủi ro, không thay được việc giới hạn quyền cloud, secret và mạng.

## Publish archive bằng SDK khi không có Docker

.NET SDK hỗ trợ PublishContainer với ContainerArchiveOutputPath để tạo archive image mà không cần daemon. Lab đã publish image Linux x64 từ service này và kiểm tra cấu hình user/entrypoint cũng như file Api.dll trong archive.

Dockerfile là một cách build riêng; SDK không tự thực thi nó khi gọi PublishContainer. Hai cách có cùng mục tiêu đóng gói nhưng không được dùng kết quả của cách này làm bằng chứng rằng cách kia đã chạy.

Máy dùng để làm bài này không chạy Docker engine, nên chưa có test khởi động container qua Docker/Kestrel network thật. Bộ test HTTP dùng TestServer vẫn có giá trị cho pipeline, còn smoke test cho container cần chạy ở CI hoặc staging có runtime phù hợp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 1. Các gate giữ cùng một artifact; publish không tự chứng minh runtime deploy."><defs><marker id="cs35-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Source + test</text><text x="34" y="123" class="t-small">PostgreSQL trong CI</text><path d="M280 108 L305 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs35-0-a)"/><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">Publish image</text><text x="319" y="123" class="t-small">Non-root + artifact</text><path d="M565 108 L590 108" class="f-none s-phys" stroke-width="2" marker-end="url(#cs35-0-a)"/><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Release gate</text><text x="604" y="123" class="t-small">Scan + smoke + digest</text></svg>
</div>
<figcaption>Hình 1. Các gate giữ cùng một artifact; publish không tự chứng minh runtime deploy.</figcaption>
</figure>

## CI có quyền tối thiểu và không tự xuất bản

Workflow dùng PostgreSQL service container riêng với dữ liệu và credential CI có thể vứt bỏ, SDK .NET 10, test Release rồi publish archive thành artifact. Credential CI trong YAML chỉ dùng cho container của job, không phải mật khẩu database của website.

Workflow chỉ có contents:read, không login registry, không xin quyền cloud và không tự deploy. Artifact test được giữ kể cả khi bước trước lỗi. Một test bị bỏ qua vì thiếu DB không được coi là xanh; fixture yêu cầu cấu hình và kết nối thật.

Bài ghi tag của các action theo đúng release đã kiểm tra lúc soạn. Khi đưa vào pipeline tổ chức, pin SHA và cơ chế cập nhật theo chính sách của đội; không bỏ qua việc kiểm tra dependency chỉ vì file YAML parse thành công.

## Migration là bước release riêng

API không tự gọi Migrate khi startup. Migrator có tệp thực thi riêng để chạy bằng role có quyền thay đổi schema trong một bước kiểm soát, sau khi review SQL và phương án phục hồi. Runtime role có thể chỉ giữ quyền đọc/ghi cần thiết.

Một migration có thể cần expand/contract để code cũ và mới cùng tồn tại trong lúc rollout. Rollback image không tự khôi phục schema hay dữ liệu đã đổi. Không đưa bước Down có thể phá dữ liệu vào lệnh rollback tự động khi chưa đánh giá.

**Bài tập:** nếu copy toàn thư mục build có .env vào runtime image thì secret có mất khi xoá file ở layer sau không? Nếu CI chỉ chạy unit test, điều gì về migration/SQL chưa được chứng minh?

<details><summary>Xem đáp án</summary><p>Secret có thể còn trong layer/history đã xuất; phải chặn không cho đưa vào image ngay từ đầu. Unit test không chạy PostgreSQL không chứng minh migration, constraint hay SQL provider trên database thật.</p></details>

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 220" role="img" aria-label="Hình 2. Hai cách đóng gói và một bước chạy thực tế là các bằng chứng khác nhau."><defs><marker id="cs35-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="34" y="95" class="t-b">Dockerfile</text><text x="34" y="123" class="t-small">Build nhiều stage</text><rect x="305" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="319" y="95" class="t-b">SDK PublishContainer</text><text x="319" y="123" class="t-small">Tạo OCI archive</text><rect x="590" y="65" width="260" height="85" rx="10" class="f-phys-soft s-phys"/><text x="604" y="95" class="t-b">Smoke test container</text><text x="604" y="123" class="t-small">Kiểm chạy và probe</text></svg>
</div>
<figcaption>Hình 2. Hai cách đóng gói và một bước chạy thực tế là các bằng chứng khác nhau.</figcaption>
</figure>

## Code và kiểm chứng

Thiết lập `TOI_NOI_PG` cho database local riêng `toi_noi_dotnet_lab` theo README bài 21, rồi từ gốc repository:

```bash
dotnet test examples/toi-noi-dotnet/31-38-trip-service/Tests
```

Fixture dùng schema GUID riêng, không xoá database; JWT dùng khoá ngẫu nhiên trong test và vẫn đi qua handler xác thực thật. Test không tự bỏ qua khi thiếu PostgreSQL. Docker/CI/cloud spec xem README service và cloud/README.md để biết phạm vi, điều kiện tiên quyết và phần chưa triển khai thật.

### Dockerfile — 31-38-trip-service/Dockerfile

```dockerfile
FROM mcr.microsoft.com/dotnet/sdk:10.0 AS build
WORKDIR /src
COPY . .
RUN dotnet restore Api/Api.csproj
RUN dotnet publish Api/Api.csproj -c Release --no-restore -p:UseAppHost=false -o /out

FROM mcr.microsoft.com/dotnet/aspnet:10.0 AS runtime
WORKDIR /app
COPY --from=build /out .
ENV ASPNETCORE_HTTP_PORTS=8080
USER $APP_UID
EXPOSE 8080
ENTRYPOINT ["dotnet", "Api.dll"]
```

Bài trước: [bài 34](/articles/dotnet10-use-case-dat-chuyen-end-to-end). Bài tiếp: [bài 36](/articles/dotnet10-aws-azure-deployment).

## Tài liệu gốc

- [SDK container publish](https://learn.microsoft.com/en-us/dotnet/core/containers/sdk-publish).
- [Container configuration](https://learn.microsoft.com/en-us/dotnet/core/containers/publish-configuration).
