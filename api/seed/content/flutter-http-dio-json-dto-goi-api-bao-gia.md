---
slug: flutter-http-dio-json-dto-goi-api-bao-gia
title: "HTTP, dio và JSON: gọi API báo giá của trip-service"
summary: "Bài 19: dựng package toi_noi_api với dio, DTO viết tay kiểm kiểu từng trường và mapper sang domain; thêm tools/fake_api (shelf) mô phỏng trip-service, interceptor log không lộ dữ liệu nhạy cảm, adapter giả để test không socket và màn hình báo giá bằng Riverpod."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, dio, http, json, api, riverpod]
language: vi
---

Màn hình báo giá của Tới Nơi đến giờ tự tính cước ngay trên máy bằng `calculateFare`. Ứng dụng thật không làm vậy. Giá phải do server quyết (sau này có khuyến mãi, giờ cao điểm), và chuyến phải được tạo ở server thì tài xế mới thấy. Hôm nay đội nối app với API: `POST /quotes` để báo giá, `POST /trips` để đặt chuyến, `GET /trips/{id}` để xem trạng thái. Service .NET hiện chưa có endpoint báo giá, nên lab gọi một fake API mô phỏng trip-service.

Ba chuyện xảy ra ngay lần chạy đầu. Service .NET trả trạng thái `Requested`, còn enum Dart tên là `requested`. Một backend khác serialize cước thành `25500.0`, và một dòng `as int` làm app sập. Mạng chập chờn khiến một lần bấm "Đặt chuyến" có thể tạo hai chuyến. Cả ba nằm ở ranh giới giữa app và server, nơi dữ liệu không còn do ta kiểm soát.

Đây là bài 19 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 18 về Riverpod](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen), vì màn hình dùng `AsyncNotifier`, và [bài 06 về package](/articles/dart-package-pubspec-lint-test-toi-noi-core). Fake API bám theo service đặt chuyến của [bài tổng hợp .NET](/articles/dotnet10-tong-hop-service-dat-chuyen). Bảng giá là số minh hoạ của hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi). Retry, xác thực và realtime để các bài 20, 24 và 36.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Server trả <code>{"fare": 25500.0}</code> và app đọc bằng <code>final fare = json['fare'] as int;</code>. Trên iOS và Android, dòng này làm gì?</p>
<details><summary>Xem đáp án</summary><p>Nó ném <code>TypeError</code>. <code>jsonDecode</code> đọc <code>25500.0</code> thành <code>double</code>, còn <code>as int</code> không đổi kiểu mà chỉ khẳng định kiểu (đã chạy với Dart 3.13.3: <em>type 'double' is not a subtype of type 'int' in type cast</em>). Lỗi nổ ở chỗ cast, xa nơi dữ liệu đi vào, và không nói trường nào sai. Mục 3 viết bộ đọc báo đúng tên DTO và tên trường.</p></details>
</div>

## Client dio với baseUrl truyền vào

Gói `dio` có ba thứ ta cần: `BaseOptions` (địa chỉ gốc, timeout, header mặc định), `Interceptor` chạy trước và sau mỗi request, và `HttpClientAdapter`, lớp cuối cùng thật sự gửi byte. Ta chọn dio vì hai thứ sau: interceptor (log ở bài này, token ở bài 24) và adapter thay được khi test. Hình 1 vẽ đường đi từ widget tới server.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Đường đi của một request: widget và provider gọi ToiNoiApiClient, client dùng dio với interceptor, dio giao cho HttpClientAdapter. Trong test adapter là InProcessAdapter gọi thẳng handler của fake API; khi chạy trên máy adapter thật gửi qua socket tới bin/server.dart.">
<defs><marker id="fl19-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="10" y="30" width="170" height="104" rx="10" class="f-phys-soft s-phys"/><text x="26" y="62" class="t-b">Widget</text><text x="26" y="88" class="t-mono">QuoteController</text><text x="26" y="112" class="t-small">state: AsyncValue</text>
<rect x="240" y="30" width="170" height="104" rx="10" class="f-pk-soft s-pk"/><text x="256" y="62" class="t-b">Client</text><text x="256" y="88" class="t-mono">ToiNoiApiClient</text><text x="256" y="112" class="t-small">DTO &lt;-&gt; JSON</text>
<rect x="470" y="30" width="170" height="104" rx="10" class="f-pk-soft s-pk"/><text x="486" y="62" class="t-b">dio</text><text x="486" y="88" class="t-small">BaseOptions</text><text x="486" y="112" class="t-small">Interceptor (log)</text>
<rect x="700" y="30" width="170" height="104" rx="10" class="f-pk-soft s-pk"/><text x="716" y="62" class="t-b">Adapter</text><text x="716" y="88" class="t-mono">HttpClientAdapter</text><text x="716" y="112" class="t-small">gửi byte</text>
<path d="M182 82 L238 82" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-1-a)"/>
<path d="M412 82 L468 82" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-1-a)"/>
<path d="M642 82 L698 82" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-1-a)"/>
<rect x="360" y="206" width="240" height="100" rx="10" class="f-pk-soft s-pk"/><text x="376" y="238" class="t-b">Trong test</text><text x="376" y="264" class="t-mono">InProcessAdapter</text><text x="376" y="288" class="t-small">từ toi_noi_fake_api</text>
<rect x="640" y="206" width="230" height="100" rx="10" class="f-phys-soft s-phys"/><text x="656" y="238" class="t-b">Chạy trên máy</text><text x="656" y="264" class="t-mono">IOHttpClientAdapter</text><text x="656" y="288" class="t-small">socket tới server.dart</text>
<path d="M760 136 L500 204" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-1-a)"/>
<path d="M800 136 L780 204" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-1-a)"/>
<text x="10" y="236" class="t-small">Hai nhánh dùng chung client, DTO</text><text x="10" y="256" class="t-small">và interceptor: chỉ adapter khác.</text>
</svg>
</div>
<figcaption>Hình 1. Một request đi từ widget tới server. Client, DTO và interceptor giống hệt nhau ở hai nhánh; chỉ adapter cuối cùng được thay.</figcaption>
</figure>

`ToiNoiApiClient` đóng gói dio và đặt ranh giới: nhận DTO, trả DTO, không biết gì về widget hay Riverpod. Nó nằm trong package `toi_noi_api`, chỉ phụ thuộc `dio` và `toi_noi_core`; fake API và `shelf` chỉ là dev dependency của test:

### pubspec.yaml · packages/toi_noi_api/pubspec.yaml

```yaml
name: toi_noi_api
description: "Client HTTP của Tới Nơi: ToiNoiApiClient dựng trên dio, DTO viết tay tách khỏi domain, interceptor log an toàn (bài 19), lỗi phân loại ApiFailure, RetryInterceptor có backoff và FaultyAdapter để test (bài 20)."
version: 0.2.0
publish_to: none

environment:
  sdk: ^3.13.3

dependencies:
  clock: ^1.1.3
  dio: ^5.11.1
  toi_noi_core:
    path: ../toi_noi_core

dev_dependencies:
  fake_async: ^1.3.3
  lints: ^6.1.0
  shelf: ^1.4.3
  test: ^1.33.0
  toi_noi_fake_api:
    path: ../../tools/fake_api
```

Địa chỉ gốc do người tạo client truyền vào; bài 25 chọn giá trị theo flavor. Dio ghép `baseUrl` với `path` bằng cộng chuỗi (xem `RequestOptions.uri`), nên client từ chối địa chỉ có query (`http://x/api?y=1` ghép `/trips/z` thành `?y=1/trips/z`) và luôn bỏ `/` cuối của base, dùng path bắt đầu bằng `/`; không thì `http://x/api/v1` và `quotes` dính thành `v1quotes` (đã chạy thử).

### client.dart · packages/toi_noi_api/lib/src/client.dart (trích)

```dart
static String _normalize(Uri baseUrl) {

// ...

  if (baseUrl.hasQuery || baseUrl.hasFragment) {
    throw ArgumentError.value(
      baseUrl,
      'baseUrl',
      'không được kèm query hay fragment (dio nối path vào cuối chuỗi)',
    );
  }
  final text = baseUrl.toString();
  return text.endsWith('/') ? text.substring(0, text.length - 1) : text;
}

// ...

String _tripPath(String id) {
  if (id.isEmpty || id == '.' || id == '..' || id.contains('/')) {
    throw ArgumentError.value(
      id,
      'id',
      'mã chuyến không được rỗng, là "." hay "..", hoặc chứa "/"',
    );
  }
  return '/trips/${Uri.encodeComponent(id)}';
}
```

`Uri.encodeComponent` mã hoá `/`, `?`, `#` nhưng để nguyên `.` và `..`, và dio chuẩn hoá đường dẫn: `getTrip('..')` với base `/api/v1` gửi `GET /api/v1/`, `acceptTrip('..')` gửi `POST /api/v1/accept`, mã rỗng gửi `GET /trips/`. Ta từ chối các mã đó và mã chứa `/` bằng `ArgumentError` trước khi gửi (một test chứng minh việc chuẩn hoá).

Timeout: tài liệu của `BaseOptions` ghi `null` nghĩa là không giới hạn, nên một server treo giữ màn hình chờ mãi. Ta đặt tạm 10 giây kết nối, 15 giây nhận dữ liệu; số đúng theo loại request là việc của bài 20.

## Hợp đồng API: fake_api và service .NET

Backend của lab là package Dart thuần `toi_noi_fake_api`, dựng bằng `shelf`. Cốt lõi là một `Handler`, tức hàm `Request -> Response`: test gọi trực tiếp không cần socket, còn `bin/server.dart` bọc nó bằng `shelf_io.serve`:

### handler.dart · tools/fake_api/lib/src/handler.dart (trích)

```dart
Handler createFakeApiHandler({
  FaultConfig faults = const FaultConfig(),
  FakeApiStore? store,
  DateTime Function()? clock,
  String Function()? newId,
  Future<void> Function(Duration)? delay,
  List<Middleware> middleware = const [],
  List<FakeApiModule> modules = const [],
}) {

// ...

var pipeline = const Pipeline()
    .addMiddleware(_traceMiddleware())
    .addMiddleware(_problemMiddleware())
    .addMiddleware(faultMiddleware(faults, delay ?? Future<void>.delayed));
for (final extra in middleware) {
  pipeline = pipeline.addMiddleware(extra);
}
return pipeline.addHandler(router.call);
```

Lỗi ném trong route được bắt thành `application/problem+json` trước lớp tiêm lỗi và độ trễ (cờ `--fail-first`, `--latency-ms`, cho bài 20). Route nằm trong các module:

### module.dart · tools/fake_api/lib/src/module.dart (trích)

```dart
abstract interface class FakeApiModule {
  void mount(Router router, FakeApiContext context);
}
```

Bài 24 thêm module OTP và middleware kiểm token, bài 36 thêm WebSocket, không sửa `createFakeApiHandler`. Báo giá gọi chính `calculateFare` của `toi_noi_core`, nên không có công thức thứ hai để lệch: 5 km ra 25.500 VND, 2,001 km ra 12.005 VND, 100 km ra 453.000 VND (test kiểm cả ba).

Bốn endpoint: `POST /quotes`, `POST /trips`, `GET /trips/{id}`, `POST /trips/{id}/accept`, cùng các lỗi có chủ đích 400, 403, 404, 409, 422 và 500 tiêm bằng cờ (bảng đầy đủ ở README của `tools/fake_api`).

Đây là JSON thật từ một lần chạy curl:

### Một lần chạy thật: POST /trips rồi accept

```json
{"id":"7e57d6d4-a4fe-411a-8d24-74635ca1de5b","serviceKind":"ride","status":"requested","distanceMeters":5000,"fare":25500,"currency":"VND","driverId":null,"requestedAt":"2026-11-06T12:27:30.373143Z","version":0}

{"id":"7e57d6d4-a4fe-411a-8d24-74635ca1de5b","serviceKind":"ride","status":"accepted","distanceMeters":5000,"fare":25500,"currency":"VND","driverId":"D-1","requestedAt":"2026-11-06T12:27:30.373143Z","version":1}

{"title":"Request conflicts with current state","status":409,"traceId":"fake-000007"}
```

Đọc mã nguồn service .NET của bài 31 đến 38, ta thấy vẫn còn khác biệt (README của `tools/fake_api` có thêm dòng về body và độ chính xác của `distanceKm`):

| Điểm | Service .NET | Fake API |
|---|---|---|
| `POST /trips` trả | `{tripId, fare, currency}` | cả chuyến, khoá `id` |
| `GET /trips/{id}` | không có `serviceKind`, `distanceMeters`, `requestedAt`; có `customerId` | có ba trường đó |
| `accept` trả | `{tripId, status, version}` | cả chuyến |
| `status` | PascalCase, hai giá trị | camelCase, đủ 8 giá trị |
| Báo giá | không có endpoint | `POST /quotes` |
| Khoá `Idempotency-Key` sai | 400, tính theo customer | 422, một kho chung |
| Lỗi nhập liệu khác | 400 | 422 |
| Danh tính | JWT | header `x-fake-driver-id` |

Hai điểm giống nhau: `Idempotency-Key` (mục 6) và `expectedVersion` khi nhận chuyến (mục 4). Hình 2 cho thấy ai phụ thuộc ai.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 2. Chiều phụ thuộc giữa các package. Lab toi_noi_19 phụ thuộc toi_noi_api và toi_noi_core; toi_noi_api phụ thuộc toi_noi_core; toi_noi_fake_api phụ thuộc toi_noi_core. toi_noi_api và toi_noi_19 chỉ dùng toi_noi_fake_api làm dev dependency để test.">
<defs><marker id="fl19-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl19-2-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="340" y="20" width="200" height="70" rx="10" class="f-phys-soft s-phys"/><text x="356" y="50" class="t-b">toi_noi_19</text><text x="356" y="74" class="t-small">lab: màn hình báo giá</text>
<rect x="60" y="150" width="200" height="70" rx="10" class="f-pk-soft s-pk"/><text x="76" y="180" class="t-b">toi_noi_api</text><text x="76" y="204" class="t-small">client dio, DTO</text>
<rect x="620" y="150" width="220" height="70" rx="10" class="f-phys-soft s-phys"/><text x="636" y="180" class="t-b">toi_noi_fake_api</text><text x="636" y="204" class="t-small">server shelf giả</text>
<rect x="340" y="270" width="200" height="70" rx="10" class="f-pk-soft s-pk"/><text x="356" y="300" class="t-b">toi_noi_core</text><text x="356" y="324" class="t-small">domain: Trip, Money</text>
<path d="M380 90 L190 148" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-2-a)"/>
<path d="M440 90 L440 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-2-a)"/>
<path d="M500 90 L690 148" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl19-2-m)"/>
<path d="M200 222 L380 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-2-a)"/>
<path d="M680 222 L500 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-2-a)"/>
<path d="M262 190 L618 190" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl19-2-m)"/>
<text x="275" y="178" class="t-small">dev_dependencies (test)</text>
<text x="20" y="300" class="t-small">liền: dependencies</text><text x="20" y="322" class="t-small">đứt: dev_dependencies</text>
</svg>
</div>
<figcaption>Hình 2. Mũi tên chỉ từ package phụ thuộc sang package được dùng. Domain không biết HTTP; fake API chỉ vào gói của app khi test.</figcaption>
</figure>

## DTO viết tay: mỗi trường một lần kiểm

**DTO** (data transfer object) là đối tượng có hình dạng đúng như dữ liệu trên dây: tên trường, kiểu và đơn vị theo hợp đồng API, chưa mang quy tắc nghiệp vụ. Bài 26 sẽ sinh mã bằng `json_serializable`; ở đây ta viết tay để thấy từng quyết định.

Quy tắc của ta: `fromJson` nhận `Object?`, kiểm từng trường và ném `DtoFormatException(dto, field, problem)` khi sai. Phần kiểm nằm trong một lớp nhỏ tên `JsonReader`:

### json_reader.dart · packages/toi_noi_api/lib/src/json_reader.dart (trích)

```dart
int nonNegativeInt(String field) {
  final value = _json[field];
  if (value is! int) {
    throw _error(
      field,
      _json.containsKey(field) ? 'cần số nguyên' : 'thiếu trường',
    );
  }
  if (value < 0) {
    throw _error(field, 'không được âm');
  }
  if (value > maxSafeInteger) {
    throw _error(field, 'quá lớn (tối đa 2^53)');
  }
  return value;
}

// ...

int wholeVnd(String field) {
  final value = _json[field];
  if (value is int) {
    return nonNegativeInt(field);
  }
  if (value is double) {
    if (value.isFinite && value == value.truncateToDouble()) {
      if (value < 0) {
        throw _error(field, 'không được âm');
      }
      if (value > maxSafeInteger) {
        throw _error(field, 'quá lớn (tối đa 2^53)');
      }
      return value.toInt();
    }
    throw _error(field, 'số đồng phải là số nguyên');
  }
  throw _error(field, _json.containsKey(field) ? 'cần số' : 'thiếu trường');
}

// ...

T enumByName<T extends Enum>(String field, List<T> values) {
  final text = string(field).toLowerCase();
  for (final value in values) {
    if (value.name.toLowerCase() == text) {
      return value;
    }
  }
  throw _error(
    field,
    'giá trị "${_json[field]}" không thuộc {${values.map((v) => v.name).join(', ')}}',
  );
}
```

Mỗi hàm là một quyết định đã có test:

- **Số nguyên mét** không nhận `5000.0`: hợp đồng nói `int`.
- **Tiền** nhận `25500` và cả `25500.0` (service .NET dùng `decimal`), nhưng từ chối `25500.5` và số âm.
- **Trạng thái** đọc theo tên enum, không phân biệt hoa thường, nên `pickedUp` và `PickedUp` đều vào `TripStatus.pickedUp`. Giá trị lạ như `onHold` bị từ chối.
- **Trường lạ** được bỏ qua, `driverId` vắng mặt coi như `null`, để server thêm trường không làm app cũ hỏng.

Thời điểm cần cẩn thận hơn cả:

### json_reader.dart · packages/toi_noi_api/lib/src/json_reader.dart (trích)

```dart
static final _instant = RegExp(
  r'^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(\.\d+)?(Z|[+-]\d{2}:\d{2})$',
);

// ...

DateTime utcInstant(String field) {
  final text = string(field);
  final match = _instant.firstMatch(text);
  if (match != null) {
    final year = int.parse(match[1]!);
    final month = int.parse(match[2]!);
    final day = int.parse(match[3]!);
    final hour = int.parse(match[4]!);
    final minute = int.parse(match[5]!);
    final second = int.parse(match[6]!);
    final probe = DateTime.utc(year, month, day, hour, minute, second);
    final offset = match[8]!;
    final offsetOk =
        offset == 'Z' ||
        (int.parse(offset.substring(1, 3)) <= 23 &&
            int.parse(offset.substring(4, 6)) <= 59);
    final inRange =
        offsetOk &&
        probe.month == month &&
        probe.day == day &&
        probe.hour == hour &&
        probe.minute == minute &&
        probe.second == second;
    final parsed = inRange ? DateTime.tryParse(text) : null;
    if (parsed != null) {
      return parsed.toUtc();
    }
  }
  throw _error(field, 'cần ISO-8601 có múi giờ, hợp lệ, nhận "$text"');
}
```

Hai điều `DateTime.parse` tự làm mà ta không muốn. Chuỗi thiếu múi giờ bị hiểu là giờ địa phương của máy: cùng chuỗi `2026-11-06T03:30:00` cho 03:30 UTC trên máy múi giờ UTC, nhưng cho 20:30 UTC của ngày hôm trước trên máy `Asia/Ho_Chi_Minh` (đã chạy thử bằng biến môi trường `TZ`). Ngày giờ tràn không báo lỗi: `DateTime.parse('2026-13-45T00:00:00Z')` trả `2027-02-14`. Bộ đọc đòi múi giờ và kiểm lại từng thành phần nên từ chối cả hai. Chuỗi kiểu .NET như `2026-11-06T03:30:00.1234567+00:00` vẫn đọc được; kết quả luôn là UTC.

`TripDto.fromJson` chỉ còn là danh sách các lời gọi đó:

### dto.dart · packages/toi_noi_api/lib/src/dto.dart (trích)

```dart
factory TripDto.fromJson(Object? json) {
  final reader = JsonReader.of('TripDto', json)
    ..expectString('currency', 'VND');
  return TripDto(
    id: reader.string('id'),
    serviceKind: reader.enumByName('serviceKind', ServiceKind.values),
    status: reader.enumByName('status', TripStatus.values),
    distanceMeters: reader.nonNegativeInt('distanceMeters'),
    fareVnd: reader.wholeVnd('fare'),
    driverId: reader.stringOrNull('driverId'),
    requestedAt: reader.utcInstant('requestedAt'),
    version: reader.nonNegativeInt('version'),
  );
}
```

Đổi lại vài dòng, mọi lỗi payload có địa chỉ (`TripDto.fare: số đồng phải là số nguyên`) thay cho `TypeError` vô danh.

## Từ DTO sang domain, và ba loại lỗi

DTO đổi khi server đổi hợp đồng, `Trip` đổi khi nghiệp vụ đổi; giữa hai bên là một mapper (hồ sơ kiến trúc §12.3 cũng dặn không chia sẻ DTO backend với domain mobile). Hình 3 là bảng chuyển đổi của `TripDto`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 3. Bảng chuyển đổi của TripDto. Cột trái là JSON trên dây, cột giữa là TripDto sau fromJson, cột phải là Trip của domain sau toDomain: mét thành km, đồng thành Money, tên trạng thái thành enum, chuỗi thời gian thành DateTime UTC. Trạng thái accepted mà driverId null làm toDomain ném DtoFormatException.">
<defs><marker id="fl19-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl19-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="20" y="28" class="t-b">JSON trên dây</text><text x="315" y="28" class="t-b">TripDto</text><text x="610" y="28" class="t-b">Trip (domain)</text>
<rect x="20" y="40" width="250" height="38" rx="8" class="f-surface s-muted"/><text x="34" y="64" class="t-mono">"distanceMeters": 5000</text>
<rect x="315" y="40" width="250" height="38" rx="8" class="f-pk-soft s-pk"/><text x="329" y="64" class="t-mono">int distanceMeters</text>
<rect x="610" y="40" width="250" height="38" rx="8" class="f-phys-soft s-phys"/><text x="624" y="64" class="t-mono">double distanceKm = 5.0</text>
<rect x="20" y="88" width="250" height="38" rx="8" class="f-surface s-muted"/><text x="34" y="112" class="t-mono">"fare": 25500</text>
<rect x="315" y="88" width="250" height="38" rx="8" class="f-pk-soft s-pk"/><text x="329" y="112" class="t-mono">int fareVnd</text>
<rect x="610" y="88" width="250" height="38" rx="8" class="f-phys-soft s-phys"/><text x="624" y="112" class="t-mono">Money(25500)</text>
<rect x="20" y="136" width="250" height="38" rx="8" class="f-surface s-muted"/><text x="34" y="160" class="t-mono">"status": "PickedUp"</text>
<rect x="315" y="136" width="250" height="38" rx="8" class="f-pk-soft s-pk"/><text x="329" y="160" class="t-mono">TripStatus.pickedUp</text>
<rect x="610" y="136" width="250" height="38" rx="8" class="f-phys-soft s-phys"/><text x="624" y="160" class="t-mono">TripStatus.pickedUp</text>
<rect x="20" y="184" width="250" height="38" rx="8" class="f-surface s-muted"/><text x="34" y="208" class="t-mono">"requestedAt": "...Z"</text>
<rect x="315" y="184" width="250" height="38" rx="8" class="f-pk-soft s-pk"/><text x="329" y="208" class="t-mono">DateTime (UTC)</text>
<rect x="610" y="184" width="250" height="38" rx="8" class="f-phys-soft s-phys"/><text x="624" y="208" class="t-mono">DateTime requestedAt</text>
<rect x="20" y="232" width="250" height="38" rx="8" class="f-surface s-muted"/><text x="34" y="256" class="t-mono">"driverId": null</text>
<rect x="315" y="232" width="250" height="38" rx="8" class="f-pk-soft s-pk"/><text x="329" y="256" class="t-mono">String? driverId</text>
<rect x="610" y="232" width="250" height="38" rx="8" class="f-phys-soft s-phys"/><text x="624" y="256" class="t-mono">String? driverId</text>
<path d="M272 59 L313 59" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/><path d="M567 59 L608 59" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/>
<path d="M272 107 L313 107" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/><path d="M567 107 L608 107" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/>
<path d="M272 155 L313 155" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/><path d="M567 155 L608 155" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/>
<path d="M272 203 L313 203" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/><path d="M567 203 L608 203" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/>
<path d="M272 251 L313 251" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/><path d="M567 251 L608 251" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-3-a)"/>
<rect x="315" y="284" width="545" height="40" rx="8" class="f-warn-soft s-warn"/><text x="331" y="309" class="t-small t-warn">accepted mà driverId null: toDomain ném DtoFormatException</text>
<path d="M590 270 L590 282" class="f-none s-warn" stroke-width="2" marker-end="url(#fl19-3-w)"/>
<text x="20" y="352" class="t-small">fromJson: kiểm kiểu từng trường</text><text x="330" y="352" class="t-small">toDomain: đổi đơn vị, kiểm bất biến của Trip</text>
</svg>
</div>
<figcaption>Hình 3. DTO giữ hình dạng của dây dẫn; domain giữ hình dạng của nghiệp vụ. Mọi thay đổi hợp đồng dừng ở hai hàm chuyển đổi này.</figcaption>
</figure>

`toDomain` làm ba việc: đổi mét sang km, đổi đồng sang `Money.fromVnd` (luôn kiểm số âm), và gọi constructor `Trip`. Constructor đó giữ một bất biến: `accepted` phải có `driverId`, còn `searching` thì không. Nếu server vi phạm, ta đổi `ArgumentError` của domain thành `DtoFormatException` thay vì để nó lọt ra như lỗi lập trình:

### dto.dart · packages/toi_noi_api/lib/src/dto.dart (trích)

```dart
Trip toDomain() {
  try {
    return Trip(
      id: id,
      serviceKind: serviceKind,
      status: status,
      distanceKm: distanceMeters / 1000,
      fare: Money.fromVnd(fareVnd),
      driverId: driverId,
      requestedAt: requestedAt,
    );
  } on ArgumentError catch (e) {
    throw DtoFormatException('TripDto', e.name ?? r'$', '${e.message}');
  }
}
```

Test chạy vòng domain, DTO, JSON, DTO, domain cho cả 8 trạng thái và nhận lại đúng `Trip`. Một test khác chứng minh `accepted` không tài xế bị báo ở trường `driverId`:

### dto_test.dart · packages/toi_noi_api/test/dto_test.dart (trích)

```dart
test('accepted mà không có tài xế -> DtoFormatException, không phải ArgumentError', () {
  final dto = TripDto.fromJson(
    _tripJson(override: {'status': 'accepted', 'driverId': null}),
  );
  expect(
    dto.toDomain,
    throwsA(
      isA<DtoFormatException>()
          .having((e) => e.field, 'field', 'driverId')
          .having((e) => e.message, 'message', contains('accepted')),
    ),
  );
});
```

Một phát hiện thật: `TripDto.fromJson` từ chối view hiện tại của service .NET vì thiếu `serviceKind`, `distanceMeters` và `requestedAt` (test nêu đúng trường `serviceKind`). Muốn gọi service thật cần mở rộng nó hoặc đặt BFF ở giữa. `TripDto` còn `version`, thứ domain không có: `toSnapshot()` trả `TripSnapshot` gồm `Trip` và `version`, để app tài xế dùng làm `expectedVersion` khi nhận chuyến (bài 45 dùng lại). `acceptTrip` không có tham số tài xế vì danh tính là việc của header hoặc token; lab gắn `x-fake-driver-id` bằng một interceptor ở client của tài xế.

Client phân biệt ba loại lỗi, vì mỗi loại cần cách xử lý khác nhau:

| Lỗi | Khi nào |
|---|---|
| `ApiException` | server trả mã không phải 2xx, dù body là gì (kể cả chữ thường hay JSON cụt) |
| `DtoFormatException` | 2xx nhưng body không phải JSON hoặc sai hợp đồng (thiếu trường, trang HTML của captive portal) |
| `DioException` | không có response: hết thời gian chờ, mất kết nối, `SocketException` |

### client.dart · packages/toi_noi_api/lib/src/client.dart (trích)

```dart
Future<T> _send<T>(
  String method,
  String path, {
  Object? body,
  Map<String, String>? headers,
  CancelToken? cancelToken,
  required T Function(Object? json) parse,
  required String dto,
}) async {
  final Response<String> response;
  try {
    response = await _dio.request<String>(
      path,
      data: body,
      cancelToken: cancelToken,
      options: Options(
        method: method,
        headers: headers,
        responseType: ResponseType.plain,
      ),
    );
  } on DioException catch (e) {
    final failed = e.response;
    if (e.type == DioExceptionType.badResponse && failed != null) {
      throw ApiException.fromResponse(failed.statusCode ?? 0, failed.data);
    }
    rethrow;
  }
  final Object? json;
  try {
    json = jsonDecode(response.data ?? '');
  } on FormatException {
    throw DtoFormatException(dto, r'$', 'body không phải JSON hợp lệ');
  }
  return parse(json);
}
```

Client dùng `ResponseType.plain` rồi tự `jsonDecode`. Để dio giải JSON thì một body hỏng ở mã 200 ra `DioException` của bộ giải mã, không phải `DtoFormatException`. `ApiException` đọc body `problem+json` (xem [bài về Problem Details](/articles/dotnet10-validation-problem-details-openapi)): `title`, `fieldErrors`, `traceId`; body không phải JSON vẫn cho tiêu đề chung. `title` của server là tiếng Anh viết cho lập trình viên nên không lên giao diện; ta chỉ hiện `traceId` thành "mã hỗ trợ". Lỗi tầng mạng để nguyên `DioException`: gom chúng thành `sealed ApiFailure` có hành động là bài 20.

## Interceptor log không lộ dữ liệu nhạy cảm

Log request giúp gỡ lỗi nhưng cũng hay làm rò `Authorization`, số điện thoại trong query, địa chỉ trong body. Danh sách chặn che header đã biết là nhạy cảm, nên header mới sẽ lọt ra; danh sách cho phép chỉ ghi giá trị header đã biết là an toàn, còn lại che. Ta chọn danh sách cho phép, vì khi bài 24 thêm interceptor token, logger không cần sửa.

### log_interceptor.dart · packages/toi_noi_api/lib/src/log_interceptor.dart (trích)

```dart
final class SafeLogInterceptor extends Interceptor {
  SafeLogInterceptor(this.sink, {Set<String> visibleHeaders = defaultHeaders})
    : visibleHeaders = {for (final name in visibleHeaders) name.toLowerCase()};

  /// Header mặc định được phép ghi giá trị.
  static const defaultHeaders = {'accept', 'content-type', 'content-length'};

// ...

void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
  options.extra[_stopwatchKey] = Stopwatch()..start();
  sink(
    '--> ${options.method} ${_target(options)} '
    '${_headers(options.headers.entries.map((e) => MapEntry(e.key, '${e.value}')))}',
  );
  handler.next(options);
}

// ...

String _headers(Iterable<MapEntry<String, String>> entries) {
  final parts = [
    for (final entry in entries)
      '${entry.key.toLowerCase()}: '
          '${visibleHeaders.contains(entry.key.toLowerCase()) ? entry.value : '[redacted]'}',
  ];
  return '{${parts.join(', ')}}';
}
```

Query chỉ ghi tên khoá, body không bao giờ được ghi, lỗi chỉ ghi loại và mã trạng thái (`DioException.toString()` in `message` và `error` do tầng dưới đặt, ta không kiểm soát nội dung). Đường dẫn vẫn có mã chuyến; đó là lựa chọn của lab, production cần quyết theo chính sách dữ liệu. Ở lab, nơi nhận log chỉ in khi `kDebugMode`, vì `debugPrint` vẫn chạy ở bản release. Đây là log thật của ba request, với một interceptor giả gắn `Authorization` và request đặt chuyến mang `Idempotency-Key`:

### Log thật (SafeLogInterceptor gắn sau interceptor token)

```text
--> POST /quotes {accept: application/json, content-type: application/json, authorization: [redacted]}
<-- 200 POST /quotes (43 ms) {content-type: application/json; charset=utf-8, content-length: 74}
--> POST /trips {accept: application/json, idempotency-key: [redacted], content-type: application/json, authorization: [redacted]}
<-- 201 POST /trips (5 ms) {content-type: application/json; charset=utf-8, location: [redacted], content-length: 211}
--> POST /quotes {accept: application/json, content-type: application/json, authorization: [redacted]}
<-- ERROR badResponse 422 POST /quotes (3 ms)
```

Logger chỉ thấy header đã có lúc nó chạy: đặt sau interceptor token thì thấy `authorization: [redacted]`, đặt trước thì không thấy header đó; cả hai đều không lộ giá trị. Test kiểm token, khoá idempotency, số điện thoại trong query và cookie đều không xuất hiện trong log. Một lưu ý khác: thông báo của `DtoFormatException` có thể chứa giá trị lấy từ payload (`nhận "..."`), nên đừng gửi nguyên văn lên crash report khi payload có dữ liệu cá nhân (bài 48).

## Test không cần socket, và nối vào Riverpod

Khi binding của `flutter_test` khởi tạo, nó cài một `HttpClient` giả: theo comment trong source, mọi request sau đó nhận mã 400 và không mở kết nối nào. Điều kiện là binding đã khởi tạo: `testWidgets` làm việc đó, còn `test()` thuần phải gọi `TestWidgetsFlutterBinding.ensureInitialized()`. Chưa khởi tạo thì `dio` vẫn mở socket thật (một test trong lab chứng minh với server loopback). Sau khi khởi tạo, quên override adapter ở provider thật cho `ApiException` 400 (cũng có test).

Lối ra là thay đường truyền, không thay client. `InProcessAdapter` (trong `package:toi_noi_fake_api/testing.dart`, vì cần shelf) đổi `RequestOptions` của dio thành `Request` của shelf, gọi handler, rồi đổi `Response` về `ResponseBody`. Nó ghi lại request, tôn trọng `receiveTimeout`, và đổi `SocketException` thành `DioException.connectionError` như socket thật:

### in_process_adapter.dart · tools/fake_api/lib/src/in_process_adapter.dart (trích)

```dart
if (error is SocketException) {
  throw DioException.connectionError(
    requestOptions: options,
    reason: error.toString(),
    error: error,
    stackTrace: stack,
  );
}

// ...

final timeout = options.receiveTimeout;
if (timeout != null && timeout > Duration.zero) {
  pending = pending.timeout(
    timeout,
    onTimeout: () => throw DioException.receiveTimeout(
      timeout: timeout,
      requestOptions: options,
    ),
  );
}
final response = await pending;
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 4. Hai làn trong flutter test sau khi binding đã khởi tạo. Làn trên, quên override: client dùng adapter mặc định, flutter test thay HttpClient bằng bản giả, kết quả luôn là 400 không mở kết nối. Làn dưới, có override: client dùng InProcessAdapter gọi handler của fake API và nhận response đúng mã, header và JSON.">
<defs><marker id="fl19-4-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker><marker id="fl19-4-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="20" y="28" class="t-b">Quên override trong testWidgets</text>
<rect x="20" y="40" width="240" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="72" class="t-b">Client + dio</text><text x="36" y="100" class="t-small">adapter mặc định</text>
<rect x="320" y="40" width="240" height="90" rx="10" class="f-warn-soft s-warn"/><text x="336" y="72" class="t-b">HttpClient bị thay</text><text x="336" y="100" class="t-small">khi binding đã khởi tạo</text>
<rect x="620" y="40" width="240" height="90" rx="10" class="f-warn-soft s-warn"/><text x="636" y="72" class="t-b">Luôn trả 400</text><text x="636" y="100" class="t-small">không mở kết nối nào</text>
<path d="M262 85 L318 85" class="f-none s-warn" stroke-width="2" marker-end="url(#fl19-4-w)"/><path d="M562 85 L618 85" class="f-none s-warn" stroke-width="2" marker-end="url(#fl19-4-w)"/>
<text x="20" y="188" class="t-b">Có override: InProcessAdapter</text>
<rect x="20" y="200" width="240" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="232" class="t-b">Client + dio</text><text x="36" y="260" class="t-small">InProcessAdapter</text>
<rect x="320" y="200" width="240" height="90" rx="10" class="f-pk-soft s-pk"/><text x="336" y="232" class="t-b">Handler shelf</text><text x="336" y="260" class="t-small">createFakeApiHandler</text>
<rect x="620" y="200" width="240" height="90" rx="10" class="f-pk-soft s-pk"/><text x="636" y="232" class="t-b">Response của fake</text><text x="636" y="260" class="t-small">đúng mã, header, JSON</text>
<path d="M262 245 L318 245" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-4-a)"/><path d="M562 245 L618 245" class="f-none s-pk" stroke-width="2" marker-end="url(#fl19-4-a)"/>
<text x="20" y="322" class="t-small">Test loopback đối chiếu: cùng kịch bản qua socket thật cho cùng kết quả.</text>
</svg>
</div>
<figcaption>Hình 4. Sau khi binding của flutter_test khởi tạo (testWidgets hoặc ensureInitialized), adapter mặc định của dio nhận 400 và không chạm mạng. test() thuần chưa khởi tạo binding thì vẫn mở socket thật. InProcessAdapter không đi qua HttpClient nên chạy giống nhau ở cả hai.</figcaption>
</figure>

Một test của `toi_noi_api` chạy cùng kịch bản qua socket loopback thật rồi qua adapter trong tiến trình và so hai kết quả: giống nhau.

`apiClientProvider` của lab đọc `httpAdapterProvider` (mặc định `null`, tức adapter thật) và gắn `SafeLogInterceptor`. Test chỉ override adapter, nên mọi dây nối còn lại là mã thật:

### providers.dart · 19-http-dio-json/lib/providers.dart (trích)

```dart
final apiClientProvider = Provider<ToiNoiApiClient>((ref) {
  final client = ToiNoiApiClient(
    baseUrl: ref.watch(apiBaseUrlProvider),
    adapter: ref.watch(httpAdapterProvider),
    interceptors: [SafeLogInterceptor(ref.watch(logSinkProvider))],
  );
  ref.onDispose(client.close);
  return client;
});
```

### support.dart · 19-http-dio-json/test/support.dart (trích)

```dart
List<Override> get overrides => [
  httpAdapterProvider.overrideWithValue(adapter),
  logSinkProvider.overrideWithValue(logs.add),
  idempotencyKeyProvider.overrideWithValue(() => 'book-${++_keyCount.value}'),
];
```

`QuoteController` theo dõi form trong `build`: đổi form thì báo giá cũ bị bỏ, và bộ đếm `_generation` chặn response đến muộn đè lên kết quả mới.

### providers.dart · 19-http-dio-json/lib/providers.dart (trích)

```dart
class QuoteController extends AsyncNotifier<Money?> {
  int _generation = 0;

  @override
  Money? build() {
    ref.watch(quoteFormProvider);
    _generation++;
    return null;
  }

  Future<void> fetch() async {
    final generation = ++_generation;
    final request = _requestOf(ref.read(quoteFormProvider));
    final client = ref.read(apiClientProvider);
    state = const AsyncLoading();
    final result = await AsyncValue.guard(() async {
      final dto = await client.quote(request);
      return dto.toMoney();
    });
    if (!ref.mounted || generation != _generation) {
      return;
    }
    state = result;
  }
}
```

`BookingController` giữ `Idempotency-Key` gắn với đúng nội dung đã gửi. Bấm lại sau lỗi thì dùng lại khoá cũ, nên server nhận ra cùng một lần đặt; đổi quãng đường thì phải là khoá mới, nếu không server trả 409.

### providers.dart · 19-http-dio-json/lib/providers.dart (trích)

```dart
({String key, QuoteForm form})? _attempt;

// ...

Future<void> book() async {
  if (state.isLoading) {
    return;
  }
  if (state.value != null) {
    // Đã có chuyến: muốn đặt chuyến khác thì gọi reset() trước.
    return;
  }
  final form = ref.read(quoteFormProvider);
  var attempt = _attempt;
  if (attempt == null || attempt.form != form) {
    attempt = (key: ref.read(idempotencyKeyProvider)(), form: form);
    _attempt = attempt;
  }
  final client = ref.read(apiClientProvider);
  final key = attempt.key;
  state = const AsyncLoading();
  final result = await AsyncValue.guard(() async {
    final dto = await client.createTrip(
      _requestOf(form),
      idempotencyKey: key,
    );
    return dto.toDomain();
  });
  if (!ref.mounted) {
    return;
  }
  if (result is AsyncData<Trip?>) {
    _attempt = null;
  }
  state = result;
}
```

Test mạnh nhất mô phỏng ca xấu nhất: server đã tạo chuyến nhưng kết nối đứt trước khi khách nhận được response. Khách bấm lại, và vẫn chỉ có một chuyến. Giới hạn: khoá chỉ sống trong RAM và gắn với form đã gửi. Lỗi mập mờ rồi đổi quãng đường thì khoá mới ra đời và có thể có hai chuyến (có test ghi lại); tắt app thì mất khoá. Lưu khoá bền là việc của outbox, bài 46.

### controllers_test.dart · 19-http-dio-json/test/controllers_test.dart (trích)

```dart
test('response bị mất sau khi server đã tạo chuyến: gửi lại không tạo chuyến thứ hai', () async {
  var calls = 0;
  final harness = Harness(
    wrap: (fake) => (Request request) async {
      final response = await fake(request);
      if (++calls == 1) {
        // Server đã xử lý xong, nhưng kết nối đứt trước khi khách nhận được.
        throw const FormatException('kết nối đứt');
      }
      return response;
    },
  );
  final container = harness.container();
  final notifier = container.read(bookingControllerProvider.notifier);
  await notifier.book();
  expect(container.read(bookingControllerProvider).hasError, isTrue);
  expect(harness.store.trips, hasLength(1)); // server đã có chuyến
  await notifier.book();
  expect(container.read(bookingControllerProvider).value?.id, 'T-1');
  expect(harness.store.trips, hasLength(1)); // vẫn chỉ một
});
```

Riverpod 3 có một cái bẫy: theo `ProviderContainer.defaultRetry`, provider ném `Exception` được thử lại tự động tối đa 10 lần, từ 200 ms nhân đôi tới 6,4 giây (`Error` thì không). Đặt lời gọi API vào `build` của một `FutureProvider` thì cả một 422 do khách nhập sai cũng bị gọi lại (`retry_in_build_test.dart` chứng minh). Lab gọi API trong phương thức (`fetch`, `book`) nên 422 chỉ tạo một request. Retry có kiểm soát là bài 20.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi gọi API</span>
<ul>
<li><strong>Cast trần trong <code>fromJson</code>.</strong> Dấu hiệu: <code>type 'double' is not a subtype of type 'int'</code> nổ ở chỗ lạ. Sửa: bộ đọc kiểm từng trường.</li>
<li><strong>Sinh <code>Idempotency-Key</code> mới ở mỗi lần gửi.</strong> Dấu hiệu: bấm lại sau lỗi mạng ra hai chuyến. Sửa: một khoá cho mỗi nội dung đặt, giữ tới khi thành công.</li>
<li><strong>Quên override adapter trong test.</strong> Dấu hiệu: <code>ApiException</code> 400, hoặc (với <code>test()</code> chưa khởi tạo binding) chạm mạng thật. Sửa: override <code>httpAdapterProvider</code>.</li>
<li><strong>Log nguyên header và body.</strong> Dấu hiệu: token hay số điện thoại trong console. Sửa: danh sách cho phép, không ghi body.</li>
<li><strong>Gọi <code>localhost</code> từ Android emulator.</strong> Dấu hiệu: lỗi kết nối dù server chạy; <code>localhost</code> là chính emulator. Sửa: <code>--dart-define=API_BASE_URL=http://10.0.2.2:8787</code> (lab chưa kiểm trên emulator).</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository, chạy lần lượt hai package và lab:

```bash
cd examples/toi-noi-flutter/tools/fake_api
dart pub get
dart analyze
dart test

cd ../../packages/toi_noi_api
dart pub get
dart analyze
dart test

cd ../../19-http-dio-json
flutter pub get
flutter analyze
flutter test
```

Chạy app với server thật theo README của lab (thêm `--fail-first 2` để thấy lỗi 500). Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: ba nơi đều `No issues found!`; `toi_noi_fake_api` có 63 test xanh, lab có 45 test xanh, và `toi_noi_api` có 86 test của bài này (thư mục `test/` còn có test của bài 20). JSON ở trên lấy từ `bin/server.dart` chạy thật. Mức: **Kiểm chứng tự động**, không cần thiết bị hay mạng ngoài.

Ta thử phá để chắc test bắt được lỗi: bỏ dòng tăng `_generation`, luôn sinh khoá mới, hay cho logger ghi mọi header đều làm test đỏ.

**Chưa làm**: retry, backoff, phân loại lỗi mạng và `connectivity_plus` (bài 20); xác thực, token và refresh khi 401 (bài 24); cấu hình flavor cho `baseUrl` (bài 25); sinh DTO bằng `json_serializable` (bài 26); realtime (bài 36); chạy trên simulator, emulator hay thiết bị thật, kể cả cấu hình mạng native để app gọi được máy dev; gọi service .NET thật, vì view hiện tại của nó thiếu ba trường (xem mục 4).

## Bài tập

1. Server thêm trạng thái `"onHold"`. App cũ gọi `TripDto.fromJson` sẽ ra sao, và đó có phải hành vi ta muốn?
2. Khách bấm "Đặt chuyến" ở 5 km, lỗi mạng, rồi kéo sang 6 km và bấm lại. `BookingController` dùng khoá nào, và có rủi ro gì?
3. `SafeLogInterceptor` đứng trước một interceptor thêm header `X-Session`. Dòng log có hiện `x-session` không?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Ném <code>DtoFormatException</code> ở trường <code>status</code>. Các <code>switch</code> vét cạn dựa vào enum đóng, nên chấp nhận bừa trạng thái lạ là phá đảm bảo ấy. Muốn app cũ không sập cần quyết định nghiệp vụ (hiện "Hãy cập nhật ứng dụng"), việc của bài 59.</p>
<p><strong>2.</strong> Khoá mới, vì <code>_attempt</code> lưu cả khoá lẫn form (khoá cũ với 6 km sẽ ra 409). Giá phải trả: nếu lần đầu đã tới server mà response mất, khách có thể có hai chuyến; khoá lại chỉ ở RAM nên tắt app là mất. Cách làm thật là outbox, bài 46.</p>
<p><strong>3.</strong> Không: logger chạy trước nên chưa thấy header đó, và không có giá trị nào để lộ. Đặt logger sau thì ra <code>x-session: [redacted]</code>, trừ khi thêm vào <code>visibleHeaders</code>.</p>
</details>

Bài trước: [Riverpod: Provider, Notifier và AsyncNotifier cho luồng đặt chuyến](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen). Bài tiếp: [Xử lý lỗi, retry, timeout và mất mạng: mạng di động không ổn định](/articles/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong).

## Tài liệu gốc

- [Fetch data from the internet](https://docs.flutter.dev/cookbook/networking/fetch-data) và [JSON and serialization](https://docs.flutter.dev/data-and-backend/serialization/json) (docs.flutter.dev).
- [dio](https://pub.dev/packages/dio) và [tài liệu API của dio](https://pub.dev/documentation/dio/latest/dio/dio-library.html) (pub.dev), gồm `Interceptor`, `HttpClientAdapter`, `BaseOptions`.
- [shelf](https://pub.dev/packages/shelf) và [shelf_router](https://pub.dev/packages/shelf_router) (pub.dev).
- [flutter_riverpod](https://pub.dev/packages/flutter_riverpod) và [Automatic Retry](https://riverpod.dev/docs/concepts2/retry) (pub.dev, riverpod.dev).
- [HttpOverrides](https://api.flutter.dev/flutter/dart-io/HttpOverrides-class.html) và [DateTime.parse](https://api.dart.dev/stable/dart-core/DateTime/parse.html) (api.flutter.dev, dart.dev).
- [Serialization in Dart](https://dart.dev/libraries/serialization/json) (dart.dev).
