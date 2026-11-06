---
slug: flutter-clean-architecture-domain-data-presentation-mobile
title: "Clean Architecture cho mobile: domain, data, presentation và repository"
summary: "Bài 21: chia luồng đặt chuyến thành domain, data và presentation với một chiều phụ thuộc; repository là interface do domain sở hữu, DTO và ApiFailure được đổi ở lớp data, use case chỉ giữ khi có quy tắc, provider nối lớp, test đọc source để khoá chiều import và phân tích khi nào kiến trúc quá tay."
category: programming
level: intermediate
tags: [flutter, dart, mobile, toi-noi, architecture, clean-architecture, riverpod, repository]
language: vi
---

Trip-service của Tới Nơi sắp tách trường `fare` thành tiền cước và phụ phí. Đội mở màn hình bài 20 xem phải sửa gì. `providers.dart` dựng `ToiNoiApiClient`, giữ khoá idempotency, gọi `guardApi` rồi ghép luôn state của màn hình; `failure_text.dart` thì `switch` trên `ApiFailure`. Đổi hợp đồng API chạm cả logic lẫn giao diện, và test nút Thử lại đòi dựng cả fake API.

Cách chữa là cho mỗi việc một chỗ ở và quy định ai được biết ai: ta chia luồng đặt chuyến thành domain, data và presentation. Đây là bài 21 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 20 về lỗi, retry và mất mạng](/articles/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong) trước, vì bài này dùng lại `ApiFailure`, `guardApi` và `Idempotency-Key`; thêm [bài 19 về dio và DTO](/articles/flutter-http-dio-json-dto-goi-api-bao-gia) và [bài 18 về Riverpod](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen). Phía backend, [bài về Clean Architecture của lộ trình .NET](/articles/dotnet10-clean-architecture-layers) có bốn tầng và đặt port `ITripRepository` ở Application; `domain` của bài này gộp Domain và Application của bên đó.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một màn hình Flutter muốn đặt chuyến mà không biết gì về dio. Ta khai báo <code>abstract interface class TripRepository</code> và để màn hình gọi nó. Interface này nên nằm cạnh code gọi API (lớp data), hay cạnh code dùng nó?</p>
<details><summary>Xem đáp án</summary><p>Cạnh code dùng nó, tức domain. Nếu interface nằm ở data, domain phải import data để biết hình dạng cổng, và mũi tên đi từ lõi ra ngoài. Khi domain sở hữu interface, data import domain để thực thi nó, và mũi tên quay vào lõi. Test <code>architecture_test.dart</code> khoá chiều này.</p></details>
</div>

## Ba lớp và một chiều phụ thuộc

| Lớp | Câu hỏi | Trong lab 21 |
|---|---|---|
| `domain` | Tới Nơi làm gì, và cần gì từ bên ngoài? | `QuoteRequest`, `TripFailure`, `TripRepository`, `RequestTrip` |
| `data` | Lấy và gửi dữ liệu bằng cách nào? | `TripRepositoryImpl`, mapper DTO và lỗi, `toi_noi_api` |
| `presentation` | Người dùng thấy và bấm gì? | provider, controller, màn hình, câu chữ lỗi |

Ba lớp là ba thư mục dưới `lib/`. Quy tắc duy nhất: **mã nguồn chỉ import hướng vào domain**. Presentation và data import domain; domain chỉ import `toi_noi_core`, package entity dùng chung (`Trip`, `Money` từ bài 06). Hình 1 vẽ các mũi tên import chính.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 420" role="img" aria-label="Hình 1. Chiều import giữa ba lớp. Composition root ở trên cùng import presentation và data (không vẽ: nó còn import toi_noi_api). Presentation import domain. Data import domain. Domain chỉ import toi_noi_core. Presentation còn import flutter, flutter_riverpod và toi_noi_ui. Data còn import toi_noi_api, và toi_noi_api import dio. Không có mũi tên nào đi từ domain sang presentation, data, flutter hay dio.">
<defs><marker id="fl21-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl21-1-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="16" width="840" height="52" rx="10" class="f-surface s-muted"/><text x="36" y="40" class="t-b">Composition root: composition.dart, main.dart</text><text x="36" y="58" class="t-small">nơi duy nhất import cả data lẫn presentation, và cắm thực thi vào chỗ cắm</text>
<rect x="20" y="140" width="260" height="140" rx="10" class="f-phys-soft s-phys"/><text x="36" y="168" class="t-b">Presentation</text><text x="36" y="196" class="t-small">providers, controller</text><text x="36" y="214" class="t-small">màn hình, câu chữ lỗi</text><text x="36" y="232" class="t-small">chỗ cắm repository</text>
<rect x="310" y="140" width="260" height="140" rx="10" class="f-pk-soft s-pk"/><text x="326" y="168" class="t-b">Domain</text><text x="326" y="196" class="t-small">QuoteRequest, TripFailure</text><text x="326" y="214" class="t-small">RequestTrip (use case)</text><text x="326" y="232" class="t-small">TripRepository: interface</text>
<rect x="600" y="140" width="260" height="140" rx="10" class="f-phys-soft s-phys"/><text x="616" y="168" class="t-b">Data</text><text x="616" y="196" class="t-small">TripRepositoryImpl</text><text x="616" y="214" class="t-small">mapper: DTO, lỗi</text><text x="616" y="232" class="t-small">guardApi</text>
<rect x="20" y="340" width="260" height="56" rx="10" class="f-surface s-muted"/><text x="36" y="364" class="t-small">flutter, flutter_riverpod,</text><text x="36" y="382" class="t-small">toi_noi_ui</text>
<rect x="310" y="340" width="260" height="56" rx="10" class="f-surface s-muted"/><text x="326" y="364" class="t-small">toi_noi_core (entity:</text><text x="326" y="382" class="t-small">Trip, Money, ServiceKind)</text>
<rect x="600" y="340" width="260" height="56" rx="10" class="f-surface s-muted"/><text x="616" y="364" class="t-small">toi_noi_api, rồi dio</text><text x="616" y="382" class="t-small">(DTO, client, ApiFailure)</text>
<path d="M150 70 L150 138" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-1-a)"/>
<path d="M730 70 L730 138" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-1-a)"/>
<path d="M282 210 L308 210" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-1-a)"/>
<path d="M598 210 L572 210" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-1-a)"/>
<path d="M150 282 L150 338" class="f-none s-muted" stroke-width="2" marker-end="url(#fl21-1-m)"/>
<path d="M440 282 L440 338" class="f-none s-muted" stroke-width="2" marker-end="url(#fl21-1-m)"/>
<path d="M730 282 L730 338" class="f-none s-muted" stroke-width="2" marker-end="url(#fl21-1-m)"/>
<text x="20" y="410" class="t-small">Mũi tên A → B: A import B. Không vẽ: Presentation và Data import toi_noi_core, Composition import toi_noi_api.</text>
</svg>
</div>
<figcaption>Hình 1. Presentation và Data cùng trỏ vào Domain; Domain chỉ trỏ xuống package entity dùng chung.</figcaption>
</figure>

Có hai điểm đáng nói. Một: entity nằm ở `toi_noi_core` dùng chung cho hai app, nên "domain" của lab là phần riêng của luồng đặt chuyến cộng entity chung. Hai: presentation **không** import data. Provider ở presentation chỉ biết một chỗ cắm; ai cắm vào là việc của composition root, ở mục 5.

Đây không phải đáp án duy nhất. [Hướng dẫn của Flutter](https://docs.flutter.dev/app-architecture/guide) đặt repository trong data layer, coi domain layer (use case) là tuỳ chọn, và tự nhận là khuyến nghị chứ không phải luật. Lab khác một điểm: interface nằm ở domain. Ta đổi sự gọn của UI import thẳng repository cụ thể lấy ranh giới mà test khoá được.

## Domain sở hữu cổng và lỗi của mình

Domain bắt đầu từ ngôn ngữ của khách. `QuoteRequest` là điều khách muốn, không phải JSON:

### quote_request.dart · 21-clean-architecture/lib/domain/quote_request.dart (trích)

```dart
final class QuoteRequest {
  const QuoteRequest({required this.serviceKind, required this.distanceKm});

  final ServiceKind serviceKind;

  /// Quãng đường theo km như khách nhập.
  final double distanceKm;

  /// Quãng đường nằm trong miền bảng giá phục vụ (0 đến [FarePolicy.maxKm] km).
  ///
  /// Viết dạng "điều kiện hợp lệ" để NaN cũng bị loại.
  bool get isWithinRange => distanceKm >= 0 && distanceKm <= FarePolicy.maxKm;

  /// Ném [Rejected] (`invalidDistance`) khi ngoài miền. Quy tắc nằm ở đây để
  /// báo giá và đặt chuyến dùng chung, không phải ở một use case.
  void requireWithinRange() {
    if (!isWithinRange) {
      throw const Rejected(RejectionKind.invalidDistance);
    }
  }

  // ...
}
```

Cổng ra thế giới là `TripRepository`. Nó nằm trong domain, và thân hàm không tồn tại:

### trip_repository.dart · 21-clean-architecture/lib/domain/trip_repository.dart (trích)

```dart
abstract interface class TripRepository {
  /// Cước dịch vụ báo cho [request].
  Future<Money> quote(QuoteRequest request);

  /// Đặt chuyến cho [request]. [intentId] nhận diện một ý định đặt chuyến:
  /// gửi lại cùng [intentId] và cùng nội dung thì không tạo thêm chuyến.
  Future<Trip> requestTrip(QuoteRequest request, {required String intentId});

  /// Đọc lại chuyến [id].
  Future<Trip> getTrip(String id);
}
```

Chữ ký chỉ dùng kiểu của domain: `QuoteRequest` vào, `Money` và `Trip` ra, không có DTO hay `Response`. Còn lỗi? Nếu interface ném `ApiFailure`, người dùng cổng phải import `toi_noi_api` để bắt nó. Vì thế domain có bộ lỗi riêng, `sealed` như `ApiFailure` nhưng nói bằng lời của sản phẩm: `Unreachable` (mất mạng, quá chậm, kết nối không an toàn), `ServiceFault`, `Rejected` (kèm `RejectionKind`) và `Unreadable`.

### trip_failure.dart · 21-clean-architecture/lib/domain/trip_failure.dart (trích)

```dart
sealed class TripFailure implements Exception {
  const TripFailure({this.supportCode, this.cause});

  /// Mã để người dùng đọc cho bộ phận hỗ trợ, nếu có.
  final String? supportCode;

  /// Lỗi gốc để ghi log. Là `Object?` nên domain không phải biết kiểu của nó.
  final Object? cause;

  /// Giao diện nên cho người dùng bấm Thử lại.
  ///
  /// Đây là quyết định của người dùng, không phải của máy: nó không nói máy
  /// được tự gửi lại an toàn (việc đó thuộc `RetryInterceptor` của lớp data).
  /// Với [Unreadable], thử lại tay là rẻ và đôi khi có ích (Wi-Fi công cộng
  /// trả trang đăng nhập), dù máy sẽ không tự thử.
  bool get canRetryByUser;
}
```

Hai chi tiết có chủ ý. `cause` là `Object?`: log vẫn thấy lỗi gốc mà domain không cần biết kiểu của nó. Và cờ tên `canRetryByUser`, không phải `retryable`: ở `toi_noi_api`, `ApiFailure.retryable` trả lời "máy có tự gửi lại được không", còn nút Thử lại là quyết định của người dùng. `Unreadable` cho bấm tay (Wi-Fi công cộng đôi khi trả trang đăng nhập thay JSON) dù máy sẽ không tự thử.

## Data thực thi cổng: DTO, mapper và dịch lỗi

`TripRepositoryImpl` implement interface của domain. Mỗi phương thức làm ba việc: đổi domain sang DTO, gọi client, đổi DTO về domain.

### trip_repository_impl.dart · 21-clean-architecture/lib/data/trip_repository_impl.dart (trích)

```dart
@override
Future<Trip> requestTrip(QuoteRequest request, {required String intentId}) =>
    _translate(() async {
      final dto = await _client.createTrip(
        request.toDto(),
        idempotencyKey: intentId,
      );
      return dto.toDomain();
    });

// ...

Future<T> _translate<T>(Future<T> Function() call) async {
  try {
    return await guardApi(call);
  } on ApiFailure catch (failure, stackTrace) {
    Error.throwWithStackTrace(failure.toTripFailure(), stackTrace);
  }
}
```

`intentId` của domain thành header `Idempotency-Key` ở đúng chỗ này, và chỉ ở đây. `_translate` chồng hai lớp: `guardApi` của bài 20 đổi lỗi nhận ra được thành `ApiFailure` (và ném lại lỗi lạ cùng việc huỷ request), rồi `catch` này đổi `ApiFailure` thành `TripFailure`. Lỗi không nhận ra không bị che thành `TripFailure`, nhưng cũng không giữ nguyên kiểu: một `StateError` trong handler lên tới nơi gọi dưới dạng `DioException` kiểu `unknown` bọc nó, vì dio gói lỗi lạ như vậy. Domain không giấu được kiểu đó mà không nuốt lỗi lập trình, nên interface ghi rõ: thứ gì không phải `TripFailure` là bug, đừng bắt. Hai test khoá điều đó: `intentId` sai định dạng ra `ArgumentError` nguyên vẹn, và `StateError` ra đúng `DioException(unknown)` bọc `StateError`.

Về mapper, lab chỉ viết phần chưa có. DTO sang `Trip` và `Money` đã nằm trong `toi_noi_api` (`TripDto.toDomain`, `QuoteDto.toMoney`) từ bài 19. Còn thiếu domain sang DTO, và lỗi sang lỗi; chiều lỗi đáng đọc:

### mappers.dart · 21-clean-architecture/lib/data/mappers.dart (trích)

```dart
extension ApiFailureMapper on ApiFailure {
  TripFailure toTripFailure() => switch (this) {
    NetworkFailure(reason: NetworkReason.badCertificate) => Unreachable(
      UnreachableReason.insecure,
      cause: this,
    ),
    NetworkFailure() => Unreachable(
      UnreachableReason.noConnection,
      cause: this,
    ),
    TimeoutFailure() => Unreachable(UnreachableReason.tooSlow, cause: this),
    ServerFailure(:final traceId) => ServiceFault(
      supportCode: traceId,
      cause: this,
    ),

    // ...

    ParseFailure() => Unreadable(cause: this),
  };
}
```

`switch` không có nhánh mặc định, nên thêm loại thứ sáu vào `ApiFailure` thì file không biên dịch cho tới khi ta quyết định tên của nó ở domain. Các nhánh `ClientFailure` (đã lược) đọc mã trạng thái: 422 trên `distanceKm` thành `invalidDistance`, 404 `notFound`, 409 `conflict`, còn lại `other`. Chỉ `traceId` đi qua, thành `supportCode`; `title` tiếng Anh của server thì không.

## Use case: chỉ giữ khi có quy tắc

Có nên tạo một use case cho mỗi thao tác? Lab trả lời bằng cách đếm quy tắc, và có hai. Quãng đường ngoài miền 0 đến 100 km bị từ chối ngay, không tốn request: quy tắc này nằm ở `QuoteRequest.requireWithinRange`, và cả báo giá lẫn đặt chuyến gọi nó, nên không cần lớp bọc. **Báo giá** vì thế không có use case: controller kiểm miền rồi gọi repository. Quy tắc thứ hai mới là lý do chính để `RequestTrip` tồn tại, vì nó cần giữ trạng thái giữa các lần gọi: một ý định đặt chuyến chỉ có một `intentId`. Gửi lại sau lỗi dùng lại `intentId` đó; đổi yêu cầu hoặc đặt thành công thì ý định kết thúc.

### request_trip.dart · 21-clean-architecture/lib/domain/request_trip.dart (trích)

```dart
Future<Trip> call(QuoteRequest request) async {
  request.requireWithinRange();
  final pending = _pending;
  final intent = pending != null && pending.request == request
      ? pending
      : (id: _newIntentId(), request: request);
  _pending = intent;
  final trip = await _repository.requestTrip(request, intentId: intent.id);
  if (identical(_pending, intent)) {
    _pending = null;
  }
  return trip;
}
```

Quy tắc thứ hai chuyển từ `BookingController` của bài 19 và 20 sang đây: giờ nó là lớp thuần Dart, test được bằng repository giả. Hình 2 là cảnh mở đầu bài 20, nhìn từ use case.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Hai lần bấm đặt chuyến khi mất response. Hàng trên, lần bấm một: bấm Đặt chuyến, RequestTrip sinh intent-1 và gửi, server tạo T-1 cho intent-1, response mất nên Unreachable lên màn hình. Hàng dưới, lần bấm hai: bấm Thử lại, RequestTrip dùng lại intent-1, server thấy đã có intent-1 và trả T-1, kết quả là một chuyến duy nhất. Dải dưới cùng cảnh báo: nếu lần hai sinh intent mới thì server tạo chuyến thứ hai.">
<defs><marker id="fl21-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl21-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="21" y="22" class="t-b">Lần bấm 1</text>
<rect x="21" y="32" width="190" height="64" rx="10" class="f-phys-soft s-phys"/><text x="35" y="58" class="t-b">Bấm Đặt chuyến</text><text x="35" y="78" class="t-small">ride, 5 km</text>
<rect x="237" y="32" width="190" height="64" rx="10" class="f-pk-soft s-pk"/><text x="251" y="58" class="t-b">RequestTrip</text><text x="251" y="78" class="t-small">sinh intent-1, gửi</text>
<rect x="453" y="32" width="190" height="64" rx="10" class="f-surface s-muted"/><text x="467" y="58" class="t-b">Server</text><text x="467" y="78" class="t-small">tạo T-1 cho intent-1</text>
<rect x="669" y="32" width="190" height="64" rx="10" class="f-warn-soft s-warn"/><text x="683" y="58" class="t-b">Response mất</text><text x="683" y="78" class="t-small">Unreachable lên UI</text>
<path d="M213 64 L235 64" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-2-a)"/>
<path d="M429 64 L451 64" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-2-a)"/>
<path d="M645 64 L667 64" class="f-none s-warn" stroke-width="2" marker-end="url(#fl21-2-w)"/>
<text x="21" y="128" class="t-b">Lần bấm 2</text>
<rect x="21" y="138" width="190" height="64" rx="10" class="f-phys-soft s-phys"/><text x="35" y="164" class="t-b">Bấm Thử lại</text><text x="35" y="184" class="t-small">cùng yêu cầu</text>
<rect x="237" y="138" width="190" height="64" rx="10" class="f-pk-soft s-pk"/><text x="251" y="164" class="t-b">RequestTrip</text><text x="251" y="184" class="t-small">dùng lại intent-1</text>
<rect x="453" y="138" width="190" height="64" rx="10" class="f-surface s-muted"/><text x="467" y="164" class="t-b">Server</text><text x="467" y="184" class="t-small">đã có intent-1: T-1</text>
<rect x="669" y="138" width="190" height="64" rx="10" class="f-pk-soft s-pk"/><text x="683" y="164" class="t-b">Trip T-1</text><text x="683" y="184" class="t-small">một chuyến duy nhất</text>
<path d="M213 170 L235 170" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-2-a)"/>
<path d="M429 170 L451 170" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-2-a)"/>
<path d="M645 170 L667 170" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-2-a)"/>
<rect x="21" y="232" width="838" height="44" rx="8" class="f-warn-soft s-warn"/><text x="35" y="259" class="t-small t-warn">Nếu RequestTrip sinh intent mới ở lần 2, server không nhận ra và tạo chuyến thứ hai</text>
</svg>
</div>
<figcaption>Hình 2. Một ý định, một intentId: gửi lại sau khi mất response không tạo thêm chuyến.</figcaption>
</figure>

Test `booking_flow_test.dart` dựng đúng Hình 2 với data thật: `FaultyAdapter` làm mất response lần đầu, server nhận hai request cùng khoá `intent-1`, `store.trips` chỉ có một chuyến. Test thuần của use case chỉ cần `FakeTripRepository`. Ta thử phá: cho use case luôn sinh `intentId` mới thì 5 test đỏ; bỏ kiểm miền thì 7 test đỏ; xoá ý định ngay khi lỗi thì 4 test đỏ.

## Presentation, chỗ cắm và composition root

Presentation cần repository nhưng không được import data, nên có một provider **chỗ cắm** không kèm thực thi:

### providers.dart · 21-clean-architecture/lib/presentation/providers.dart (trích)

```dart
final tripRepositoryProvider = Provider<TripRepository>(
  (ref) => throw UnimplementedError(
    'Ghi đè tripRepositoryProvider ở composition root hoặc trong test',
  ),
);
```

Quên ghi đè thì đọc provider này ném `ProviderException` bọc `UnimplementedError` ngay lần đầu (một test khoá hành vi đó), nên app không lặng lẽ chạy với thực thi sai. Phần còn lại của presentation chỉ thấy domain. `BookingController` gọi use case, gói kết quả bằng `AsyncValue.guard`, và lỗi trong state là `TripFailure`:

### providers.dart · 21-clean-architecture/lib/presentation/providers.dart (trích)

```dart
Future<void> book() async {
  if (state.isLoading) {
    return;
  }
  _lastOp = _BookingOp.book;
  final request = ref.read(quoteRequestProvider);
  final requestTrip = ref.read(requestTripProvider);
  state = const AsyncLoading();
  final result = await AsyncValue.guard<Trip?>(() => requestTrip(request));
  // ...
}
```

Composition root cắm thực thi thật vào chỗ cắm, gắn log và retry của bài 20 lên client:

### composition.dart · 21-clean-architecture/lib/composition.dart (trích)

```dart
Override productionTripRepository({Uri? baseUrl}) =>
    tripRepositoryProvider.overrideWith((ref) {
      final client = ToiNoiApiClient(
        baseUrl: baseUrl ?? apiBaseUrl,
        interceptors: [SafeLogInterceptor(debugPrint)],
      );
      client.dio.interceptors.add(RetryInterceptor(dio: client.dio));
      ref.onDispose(client.close);
      return TripRepositoryImpl(client);
    });
```

Nhóm test đầu của `quote_screen_test.dart` chạy màn hình với `FakeTripRepository`, nên không có `dio`, HTTP hay fake API (nhóm sau dùng data thật). Đó là lợi ích cụ thể nhất: ta kiểm nút Thử lại bằng cách cho repository giả ném `Unreachable`, thay vì dựng server giả làm rơi kết nối. Hình 3 vẽ đường đi lúc chạy của một lần đặt chuyến.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 412" role="img" aria-label="Hình 3. Đường đi lúc chạy của một lần đặt chuyến, từ trên xuống. BookingController.book thuộc presentation gọi RequestTrip. RequestTrip thuộc domain gọi TripRepository, một interface cũng thuộc domain. TripRepositoryImpl thuộc data thực thi interface đó và gọi ToiNoiApiClient của toi_noi_api. Mũi tên đi lên mang Trip hoặc TripFailure; từ client lên TripRepositoryImpl là TripDto hoặc lỗi thô. Giữa interface và phần thực thi có đường ranh giới, nơi mũi tên import đi ngược từ TripRepositoryImpl lên TripRepository.">
<defs><marker id="fl21-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl21-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="14" width="840" height="46" rx="8" class="f-phys-soft s-phys"/><text x="36" y="42" class="t-b">BookingController.book</text><text x="330" y="42" class="t-small">presentation: đọc QuoteRequest từ provider</text>
<rect x="20" y="92" width="840" height="46" rx="8" class="f-pk-soft s-pk"/><text x="36" y="120" class="t-b">RequestTrip.call</text><text x="330" y="120" class="t-small">domain: kiểm miền 0 đến 100 km, giữ intentId</text>
<rect x="20" y="170" width="840" height="46" rx="8" class="f-pk-soft s-pk"/><text x="36" y="198" class="t-b">TripRepository</text><text x="330" y="198" class="t-small">domain: interface, không có thân hàm</text>
<rect x="20" y="262" width="840" height="46" rx="8" class="f-phys-soft s-phys"/><text x="36" y="290" class="t-b">TripRepositoryImpl</text><text x="330" y="290" class="t-small">data: DTO, guardApi, đổi lỗi</text>
<rect x="20" y="340" width="840" height="46" rx="8" class="f-surface s-muted"/><text x="36" y="368" class="t-b">ToiNoiApiClient</text><text x="330" y="368" class="t-small">toi_noi_api: dio, JSON, interceptor</text>
<path d="M130 62 L130 90" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-3-a)"/><text x="142" y="80" class="t-small">gọi call(request)</text>
<path d="M130 140 L130 168" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-3-a)"/><text x="142" y="158" class="t-small">gọi requestTrip(request, intentId)</text>
<path d="M130 218 L130 260" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-3-a)"/><text x="142" y="232" class="t-small">gọi qua interface</text>
<path d="M130 310 L130 338" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-3-a)"/><text x="142" y="328" class="t-small">gọi createTrip</text>
<path d="M800 90 L800 62" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-3-a)"/><text x="790" y="80" text-anchor="end" class="t-small">Trip hoặc TripFailure</text>
<path d="M800 168 L800 140" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-3-a)"/><text x="790" y="158" text-anchor="end" class="t-small">Trip hoặc TripFailure</text>
<path d="M800 260 L800 218" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-3-a)"/><text x="790" y="256" text-anchor="end" class="t-small">Trip hoặc TripFailure</text>
<path d="M800 338 L800 310" class="f-none s-pk" stroke-width="2" marker-end="url(#fl21-3-a)"/><text x="790" y="328" text-anchor="end" class="t-small">TripDto hoặc lỗi thô</text>
<path d="M480 260 L480 218" class="f-none s-warn" stroke-width="2" marker-end="url(#fl21-3-w)"/><text x="492" y="232" class="t-small t-warn">import: Impl vào interface</text>
<path d="M20 239 L860 239" class="f-none s-warn" stroke-width="1" stroke-dasharray="6 4"/>
<text x="20" y="406" class="t-small">Chiều chạy đi xuống, chiều import qua đường đứt đi lên: đó là phụ thuộc bị đảo.</text>
</svg>
</div>
<figcaption>Hình 3. Lời gọi chạy từ trên xuống, nhưng ở đường ranh giới, import của TripRepositoryImpl trỏ ngược lên interface của domain.</figcaption>
</figure>

## Chiều phụ thuộc là thứ test được

Quy ước chỉ trên giấy sẽ bị phá trong lần sửa gấp đầu tiên: ai đó cần một `DioException` ở màn hình và thêm một dòng import. Vì thế lab có test đọc source. Mỗi lớp khai báo những gì nó được phép import:

### architecture_test.dart · 21-clean-architecture/test/architecture_test.dart (trích)

```dart
const layers = [
  Layer(
    'domain',
    packages: {'toi_noi_core'},
    layers: {'domain'},
    dartLibraries: {'async', 'collection', 'core', 'math'},
  ),
  Layer(
    'data',
    packages: {'toi_noi_core', 'toi_noi_api'},
    layers: {'domain', 'data'},
  ),
  Layer(
    'presentation',
    packages: {'flutter', 'flutter_riverpod', 'toi_noi_core', 'toi_noi_ui'},
    layers: {'domain', 'presentation'},
  ),
];
```

Test đọc từng file `.dart` dưới `lib/<lớp>`, lấy mọi dòng `import` và `export`, rồi so với bảng. Domain có thêm ba test nêu đích danh `package:flutter`, `package:dio` và `package:toi_noi_api`. Import tương đối cũng bị chặn, vì `../data/x.dart` trong domain trèo sang lớp khác mà không có chữ `package:`. Data không import Flutter và presentation không import data: cả hai là hệ quả của bảng.

Test không thể đỏ thì không đáng tin, nên tám test nữa kiểm chính bộ kiểm: nó bắt `import` bị cấm, `export`, import tương đối sang lớp khác, `dart:io` trong domain, và bỏ qua dòng chú thích. Một test chặn thư mục rỗng cho kết quả xanh giả. Ta phá thật: thêm `import 'package:dio/dio.dart'` vào `trip_repository.dart` thì hai test đỏ và thông báo chỉ ra file; thêm import `data/mappers.dart` vào một file presentation thì test lớp presentation đỏ; thêm import `toi_noi_api` vào `quote_request.dart` thì hai test đỏ.

Giới hạn cần nói: bộ kiểm dùng biểu thức chính quy trên dòng `import` và `export`, không dùng analyzer. Một directive viết URI ở dòng khác với từ khoá, hoặc import có điều kiện kiểu `if (dart.library.io) 'package:dio/dio.dart'`, sẽ lọt. Lint chuyên dụng thuộc bài 32.

### Khi nào kiến trúc là quá tay

Ba lớp có giá: lab 21 có 14 file trong `lib/` (742 dòng), lab 20 có 8 file. Giá đó chỉ đáng trả khi mua được thứ cụ thể. Ở Tới Nơi nó mua ba thứ: đổi hợp đồng API chỉ chạm lớp data, đổi cách hiển thị không chạm domain, và màn hình test được không cần server giả. Ta nên giảm kiến trúc khi:

- Use case chỉ gọi một hàm repository rồi trả kết quả. Bỏ nó, gọi repository thẳng. Lab làm đúng thế với báo giá và làm mới chuyến.
- App chỉ có một màn hình đọc dữ liệu và không có quy tắc nào ngoài hiển thị: một provider gọi client là đủ.
- Mapper chép từng trường sang một lớp giống hệt. Mapper chỉ có giá trị khi hai bên khác nhau: km và mét, `Money` và `int`, lỗi kỹ thuật và lỗi sản phẩm.

Lab bỏ có chủ ý: lớp cơ sở `UseCase<T, P>`, kiểu `Either`/`Result` (lỗi là `TripFailure` được ném, như `AsyncValue.guard` dùng), entity riêng cho báo giá (dùng `Money`), interface riêng cho từng nguồn dữ liệu (chỉ có một nguồn tới bài 22), và cách chia theo feature (bài 44). Thêm lớp khi có một lý do thay đổi riêng, không thêm vì sơ đồ trông đầy đủ.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp khi chia lớp</span>
<ul>
<li><strong>Interface repository đặt ở data.</strong> Dấu hiệu: domain import <code>data/</code> chỉ để biết kiểu của cổng. Nguyên nhân: đặt interface cạnh thực thi cho tiện. Cách sửa: chuyển interface sang domain; test import bắt khi quay lại.</li>
<li><strong>Lỗi kỹ thuật lọt lên giao diện.</strong> Dấu hiệu: <code>failure_text.dart</code> import <code>toi_noi_api</code>. Nguyên nhân: repository ném lại lỗi của client. Cách sửa: đổi sang lỗi domain ở biên data, giữ lỗi gốc trong <code>cause</code>.</li>
<li><strong>Che lỗi lập trình thành lỗi miền.</strong> Dấu hiệu: một <code>StateError</code> hiện thành "Không tới được máy chủ". Nguyên nhân: <code>catch (e)</code> đổi mọi thứ. Cách sửa: chỉ đổi <code>ApiFailure</code>, như <code>_translate</code> làm.</li>
<li><strong>Dùng cờ của máy cho nút của người.</strong> Dấu hiệu: nút Thử lại ẩn với <code>ParseFailure</code> dù Wi-Fi công cộng chỉ cần đăng nhập. Nguyên nhân: lấy <code>ApiFailure.retryable</code> (máy tự gửi lại có an toàn không) làm điều kiện cho nút. Cách sửa: cờ riêng <code>canRetryByUser</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/21-clean-architecture
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `No issues found!` và 69 test xanh. Mức kiểm chứng: **Kiểm chứng tự động**, không cần thiết bị hay mạng ngoài. Phiên bản dùng: `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `dio` 5.11.1, `flutter_lints` 6.0.0.

| Test | Chứng minh điều gì |
|---|---|
| `architecture_test` (15) | domain không import flutter, dio, `toi_noi_api`; mỗi lớp chỉ import đúng bảng; bộ kiểm bắt được vi phạm |
| `request_trip_test` (12) | repository giả: biên 0 và 100 km, NaN, vô cực; `intentId` dùng lại sau lỗi, đổi theo yêu cầu, chung cho hai lần bấm đồng thời |
| `trip_repository_impl_test` (19) | DTO sang `Trip` và `Money` (5 km là 25.500, 2,001 km là 12.005, 100 km là 453.000 VND); từng loại lỗi sang `TripFailure`; lỗi lạ không bị che |
| `booking_flow_test` (4) | use case + data + fake API: mất response vẫn một chuyến; ghi lại giới hạn khi đổi yêu cầu |
| `controllers_test`, `failure_text_test`, `quote_screen_test` (19) | chỗ cắm chưa ghi đè thì lỗi; retry dùng lại `intentId`; nút Thử lại theo `canRetryByUser`; màn hình với repository giả và với data thật |

**Chưa làm**: chạy trên simulator, emulator hay thiết bị thật; không test nào chạy `productionTripRepository` (composition root chưa được kiểm); lint chiều import bằng analyzer (bài 32); lưu cục bộ sau repository (bài 22); connectivity và nút Mở cài đặt của bài 20.

Giới hạn của use case: ý định chỉ sống trong bộ nhớ và chỉ nhớ một yêu cầu. Mất response rồi đổi yêu cầu thì server có hai chuyến, quay lại yêu cầu đầu thành ba (một test ghi lại điều này), tắt app thì mất khoá. Lưu khoá bền là việc của bài 46.

Ở Phần 6, khi chuyển sang feature-first, chỗ cắm provider và bộ lỗi dùng chung sẽ phải tách theo feature.

## Bài tập

1. Tới Nơi thêm thao tác "Huỷ chuyến". Nó cần use case riêng không? Nếu có, quy tắc nào của `toi_noi_core` nằm trong đó?
2. Trip-service đổi `fare` thành hai trường `baseFare` và `surcharge`. Theo lab, những file nào phải sửa, và file nào chắc chắn không?
3. Một đồng nghiệp thêm `import 'package:dio/dio.dart';` vào `quote_screen.dart` để bắt `DioException`. Test nào đỏ, và đáng ra lỗi đó phải được xử lý ở đâu?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Có, nếu có quy tắc: huỷ chỉ được ở <code>searching</code>, <code>accepted</code> và <code>arriving</code>. Use case gọi <code>Trip.apply(Cancelled(...))</code> (ném <code>StateError</code> với cặp không hợp lệ) rồi gọi repository. Nếu chỉ chuyển tiếp một lời gọi thì bỏ use case.</p>
<p><strong>2.</strong> Sửa DTO trong <code>toi_noi_api</code> (<code>TripDto</code>, <code>QuoteDto</code>) và <code>toDomain</code> nếu cách tính tổng đổi. Domain và presentation chắc chắn không đổi, vì chỉ thấy <code>Money</code> và <code>Trip</code>.</p>
<p><strong>3.</strong> Test lớp presentation đỏ, báo <code>package dio không được phép</code>. Lỗi mạng phải đổi ở biên data thành <code>TripFailure</code> (<code>_translate</code>), và màn hình chỉ đọc <code>TripFailure</code>.</p>
</details>

Bài trước: [Xử lý lỗi, retry, timeout và mất mạng: mạng di động không ổn định](/articles/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong). Bài tiếp: [Lưu trữ cục bộ: drift, shared_preferences và secure storage](/articles/flutter-local-persistence-drift-secure-storage-lich-su-offline).

## Tài liệu gốc

- [Architecture recommendations and resources](https://docs.flutter.dev/app-architecture) và [Guide to app architecture](https://docs.flutter.dev/app-architecture/guide) (docs.flutter.dev).
- [Architecture case study](https://docs.flutter.dev/app-architecture/case-study) (docs.flutter.dev): ví dụ thư mục có `data/repositories` và `data/services`, để so với cách đặt interface ở domain của bài này.
- [Class modifiers: sealed, interface](https://dart.dev/language/class-modifiers) và [Patterns](https://dart.dev/language/patterns) (dart.dev).
- [Providers và override](https://riverpod.dev/docs/concepts2/overrides) và [flutter_riverpod](https://pub.dev/packages/flutter_riverpod) (riverpod.dev, pub.dev).
- [dio](https://pub.dev/packages/dio) (pub.dev).
