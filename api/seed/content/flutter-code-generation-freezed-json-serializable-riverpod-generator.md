---
slug: flutter-code-generation-freezed-json-serializable-riverpod-generator
title: "Code generation: freezed, json_serializable, riverpod_generator và build_runner"
summary: "Bài 26: viết lại TripDto và QuoteDto bằng freezed 4 và json_serializable, chứng minh trên cùng một JSON rằng hành vi giữ nguyên, chỉ ra chỗ code sinh lỏng hơn bản viết tay và siết bằng JsonConverter. Thêm union sealed, @riverpod, commit file sinh. 134 test."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, code-generation, freezed, json-serializable, riverpod, build-runner]
language: vi
---

Một buổi sáng, PM xin thêm trường `note` (ghi chú của khách cho tài xế) vào chuyến. Với `TripDto` viết tay ở bài 19, ta phải sửa ít nhất năm chỗ: tham số của constructor, khai báo field, `fromJson`, `toJson` và `fromDomain`. Quên `toJson` thì code vẫn biên dịch và ghi chú rơi mất giữa đường.

Code generation giao phần lặp đó cho máy: ta khai báo trường một lần, `build_runner` sinh phần còn lại. Nhưng máy sinh theo mặc định của nó, không theo hợp đồng của ta. Bài này viết lại `TripDto` và `QuoteDto` bằng `freezed` và `json_serializable`, chứng minh trên cùng một JSON rằng hành vi giữ nguyên, chỉ ra chỗ bản sinh mã lỏng hơn bản viết tay và siết lại bằng `JsonConverter`.

Đây là bài 26 của lộ trình **Flutter qua Tới Nơi**, bài cuối của Phần 3. Ta nên đọc [bài 25 về cấu hình môi trường](/articles/flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod) trước. Lab dùng lại DTO viết tay của [bài 19](/articles/flutter-http-dio-json-dto-goi-api-bao-gia), `ApiFailure` của [bài 20](/articles/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong) và Riverpod của [bài 18](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen). Bài dựa trên Flutter 3.47.4, Dart 3.13.3, `freezed` 4.0.3, `json_serializable` 6.15.2, `riverpod_generator` 4.0.10 và `build_runner` 2.17.1. Số tiền là minh hoạ theo bảng giá của loạt bài.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Server trả <code>"distanceMeters": 5000.5</code>. Một DTO sinh bằng <code>json_serializable</code> có trường <code>int distanceMeters</code>, không converter, không tuỳ chọn nào khác. <code>fromJson</code> cho kết quả gì?</p>
<details><summary>Xem đáp án</summary><p>Trường nhận <strong>5000</strong>, không có lỗi nào. Code sinh đọc <code>int</code> bằng <code>(json['distanceMeters'] as num).toInt()</code> nên phần lẻ bị cắt thầm lặng, còn <code>TripDto</code> viết tay của bài 19 ném <code>DtoFormatException</code>. Test <code>mét lẻ 5000.5 bị cắt thành 5000 mà không báo</code> chứng minh điều này. Mục 3 liệt kê các chỗ tương tự và cách siết.</p></details>
</div>

## 1. Khi nào sinh mã đáng giá

`build_runner` đọc code của ta bằng analyzer, chạy các generator và ghi file mới ngay cạnh file nguồn (Hình 1). `freezed` sinh `==`, `hashCode`, `toString`, `copyWith` và các nhánh của union vào `*.freezed.dart`. `json_serializable` sinh `fromJson` và `toJson`, `riverpod_generator` sinh provider; cả hai ghi vào `*.g.dart`. Mỗi file sinh khai `part of` file nguồn, nên cùng một library: nguồn gọi được `_$TripDtoFromJson` dù tên bắt đầu bằng dấu gạch dưới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Một lần chạy build_runner. Ba file nguồn là trip_dto.dart với @freezed và fromJson, quote_state.dart với @freezed sealed, và providers.dart với @riverpod. Cả ba vào build_runner, nơi chạy ba generator freezed, json_serializable và riverpod_generator. Đầu ra là trip_dto.freezed.dart cùng trip_dto.g.dart, quote_state.freezed.dart và providers.g.dart, nằm cạnh file nguồn.">
<defs><marker id="fl26-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="16" y="30" width="230" height="64" rx="10" class="f-phys-soft s-phys"/><text x="30" y="58" class="t-mono">trip_dto.dart</text><text x="30" y="80" class="t-small">@freezed, fromJson, part</text>
<rect x="16" y="130" width="230" height="64" rx="10" class="f-phys-soft s-phys"/><text x="30" y="158" class="t-mono">quote_state.dart</text><text x="30" y="180" class="t-small">@freezed sealed, part</text>
<rect x="16" y="230" width="230" height="64" rx="10" class="f-phys-soft s-phys"/><text x="30" y="258" class="t-mono">providers.dart</text><text x="30" y="280" class="t-small">@riverpod, part</text>
<rect x="330" y="90" width="220" height="150" rx="10" class="f-surface s-line"/><text x="344" y="120" class="t-b">build_runner</text><text x="344" y="144" class="t-small">đọc code qua analyzer</text><text x="344" y="164" class="t-small">chạy ba generator:</text><text x="344" y="188" class="t-mono">freezed</text><text x="344" y="208" class="t-mono">json_serializable</text><text x="344" y="228" class="t-mono">riverpod_generator</text>
<rect x="604" y="30" width="260" height="64" rx="10" class="f-pk-soft s-pk"/><text x="618" y="58" class="t-mono">trip_dto.freezed.dart</text><text x="618" y="80" class="t-mono">trip_dto.g.dart</text>
<rect x="604" y="130" width="260" height="64" rx="10" class="f-pk-soft s-pk"/><text x="618" y="158" class="t-mono">quote_state.freezed.dart</text><text x="618" y="180" class="t-small">union sealed và copyWith</text>
<rect x="604" y="230" width="260" height="64" rx="10" class="f-pk-soft s-pk"/><text x="618" y="258" class="t-mono">providers.g.dart</text><text x="618" y="280" class="t-small">provider sinh từ @riverpod</text>
<path d="M248 62 L328 140" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-1-a)"/>
<path d="M248 162 L328 165" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-1-a)"/>
<path d="M248 262 L328 190" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-1-a)"/>
<path d="M552 140 L602 62" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-1-a)"/>
<path d="M552 165 L602 162" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-1-a)"/>
<path d="M552 190 L602 262" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-1-a)"/>
</svg>
</div>
<figcaption>Hình 1. Một lần chạy <code>build_runner</code>. Mũi tên chỉ chiều dữ liệu: file nguồn của ta đi vào, file sinh ra nằm cạnh nguồn và là <code>part of</code> của nó.</figcaption>
</figure>

Sinh mã không làm code ngắn đi. Số dòng thật trong repo, cho `QuoteDto` và `TripDto`:

| | Viết tay (bài 19) | Sinh mã (lab này) |
|---|---|---|
| Khai báo `QuoteDto` và `TripDto` | 130 dòng (`dto.dart`) | 108 dòng (`quote_dto.dart` 34, `trip_dto.dart` 74) |
| Quy tắc kiểm JSON | 166 dòng (`json_reader.dart`) | 206 dòng (`converters.dart` 179, `decode.dart` 27) |
| Dòng do máy viết | 0 | 708 dòng (`*.freezed.dart` và `*.g.dart` của hai DTO) |
| `==`, `hashCode`, `copyWith`, `toString` | không có | có |

Khai báo ngắn hơn một chút, nhưng quy tắc kiểm JSON vẫn phải viết (giờ nằm trong converter) và máy viết thêm 708 dòng. Thứ ta mua được là một chỗ khai báo trường thay vì năm, bốn thành viên miễn phí và union `sealed`. Theo nhận định của ta, sinh mã đáng giá khi có nhiều model đổi thường xuyên hoặc cần `copyWith`, union; với vài DTO ổn định, một `JsonReader` viết tay như bài 19 dễ đọc hơn và không cần bước build.

## 2. freezed: data class, copyWith và union sealed

Lớp của `freezed` khai báo bằng một `factory` chuyển hướng tới lớp sinh. Với `freezed` 4.0.3, lớp một biến thể là `abstract class` và union là `sealed class`. Ta đã thử bỏ từng từ khoá: bỏ `abstract` thì analyzer báo `non_abstract_class_inherits_abstract_member`; đổi `sealed` thành `abstract` ở union thì `switch` trên state báo `non_exhaustive_switch_expression`, tức là mất đúng thứ ta cần.

### trip_dto.dart · 26-codegen/lib/dto/trip_dto.dart (trích)

```dart
@freezed
abstract class TripDto with _$TripDto {
  const TripDto._();

  const factory TripDto({
    required String id,
    @ServiceKindConverter() required ServiceKind serviceKind,
    @TripStatusConverter() required TripStatus status,
    @NonNegativeIntConverter() required int distanceMeters,
    @JsonKey(name: 'fare') @WholeVndConverter() required int fareVnd,
    @VndCurrencyConverter() required String currency,
    String? driverId,

    /// Thời điểm đặt, luôn UTC.
    @UtcInstantConverter() required DateTime requestedAt,

    /// Số phiên bản của chuyến trên server, dùng làm `expectedVersion`.
    @NonNegativeIntConverter() required int version,
  }) = _TripDto;

  factory TripDto.fromJson(Object? json) =>
      decodeDto('TripDto', json, _$TripDtoFromJson);
```

`const TripDto._()` cho phép thêm method như `toDomain`. `@JsonKey(name: 'fare')` ánh xạ field `fareVnd` vào khoá JSON `fare`. `fromJson` nhận `Object?` vì ta bọc nó trong `decodeDto` ở mục 3.

Hai `TripDto` sinh mã cùng JSON thì `==`; hai `TripDto` viết tay thì không, vì lớp đó không định nghĩa `==`. `copyWith(version: 5)` giữ `driverId`, còn `copyWith(driverId: null)` xoá nó.

Chỗ `freezed` đáng tiền nhất là union. Màn hình báo giá có bốn trạng thái loại trừ nhau, mỗi trạng thái chỉ mang dữ liệu có nghĩa ở đó:

### quote_state.dart · 26-codegen/lib/quote_state.dart (trích)

```dart
@freezed
sealed class QuoteScreenState with _$QuoteScreenState {
  /// Chưa hỏi gì.
  const factory QuoteScreenState.idle() = QuoteIdle;

  /// Đang chờ server.
  const factory QuoteScreenState.loading({required QuoteInput input}) =
      QuoteLoading;

  /// Đã có giá.
  const factory QuoteScreenState.ready({
    required QuoteInput input,
    required Money fare,
  }) = QuoteReady;

  /// Thất bại, đã phân loại.
  const factory QuoteScreenState.failed({
    required QuoteInput input,
    required ApiFailure failure,
  }) = QuoteFailed;
}
```

Màn hình vét cạn bốn nhánh bằng pattern matching:

### quote_screen.dart · 26-codegen/lib/quote_screen.dart (trích)

```dart
            // Vét cạn bốn nhánh: thiếu một nhánh là lỗi biên dịch.
            switch (state) {
              QuoteIdle() => const Text('Chọn dịch vụ rồi bấm báo giá.'),
              QuoteLoading() => const LinearProgressIndicator(),
              QuoteReady(:final fare) => Text(
                fare.toString(),
                style: Theme.of(context).textTheme.headlineMedium,
              ),
              QuoteFailed(:final failure) => Text(_describe(failure)),
            },
```

Ta đã xoá nhánh `QuoteLoading()` khỏi `switch` này và analyzer báo `non_exhaustive_switch_expression`.

`freezed` 4.0.3 vẫn sinh `when` và `map`, nhưng lab dùng pattern matching. `ApiFailure` không định nghĩa `==`, nên hai nhánh `failed` chỉ bằng nhau khi cùng trỏ tới một failure. Dart 3.13 có primary constructor và `freezed` 4 nhận cú pháp `class X({required final A a}) with _$X;` (ta đã thử trong một file tạm, chạy được); lab giữ `factory` vì union cần nhiều `factory`.

## 3. json_serializable: chỗ lỏng hơn bản viết tay và cách siết

Đây là `fromJson` mà `json_serializable` sinh cho bản mặc định (`NaiveTripDto`, cùng chín trường, không converter; lab giữ nó chỉ để so sánh).

### naive_trip_dto.g.dart · 26-codegen/lib/dto/naive_trip_dto.g.dart (trích)

```dart
_NaiveTripDto _$NaiveTripDtoFromJson(Map<String, dynamic> json) =>
    _NaiveTripDto(
      id: json['id'] as String,
      serviceKind: $enumDecode(_$ServiceKindEnumMap, json['serviceKind']),
      status: $enumDecode(_$TripStatusEnumMap, json['status']),
      distanceMeters: (json['distanceMeters'] as num).toInt(),
      fareVnd: (json['fare'] as num).toInt(),
      currency: json['currency'] as String,
      driverId: json['driverId'] as String?,
      requestedAt: DateTime.parse(json['requestedAt'] as String),
      version: (json['version'] as num).toInt(),
    );
```

Mỗi dòng là một quyết định mà `JsonReader` của bài 19 quyết định khác. Kết quả test trên cùng một JSON:

| JSON đầu vào | Bản viết tay | Bản mặc định |
|---|---|---|
| `distanceMeters: 5000.5` | `DtoFormatException` | 5000, không báo |
| `fare: 25500.5` | `DtoFormatException` | 25500, sai tiền |
| `fare`, `distanceMeters`, `version` âm | từ chối | nhận |
| `currency: "USD"` | từ chối | nhận |
| `requestedAt: "2026-13-45T03:30:00Z"` | từ chối | 2027-02-14 03:30 UTC |
| `requestedAt` thiếu múi giờ | từ chối | giờ địa phương, `isUtc` là `false` |
| `id` là số, hoặc thiếu `id` | `DtoFormatException` | `TypeError` |
| `serviceKind: "RIDE"` | nhận | `ArgumentError` (chặt hơn) |

Trên cả bảng ca, bản viết tay từ chối 35 ca JSON object sai; bản mặc định nhận thầm lặng 16 ca, ném `TypeError` ở 13 ca và lỗi khác ở 6 ca. Ví dụ cho ngày tràn:

### naive_dto_test.dart · 26-codegen/test/naive_dto_test.dart (trích)

```dart
    test('ngày tràn bị cuộn sang ngày khác thay vì báo lỗi', () {
      DateTime at(String text) =>
          naive(changed(validTripJson(), {'requestedAt': text})).requestedAt;
      expect(at('2026-13-45T03:30:00Z'), DateTime.utc(2027, 2, 14, 3, 30));
      expect(at('2026-02-30T03:30:00Z'), DateTime.utc(2026, 3, 2, 3, 30));
      expect(at('2026-11-06T25:61:61Z'), DateTime.utc(2026, 11, 7, 2, 2, 1));
      expect(
        at('2026-11-06T03:30:00+99:99'),
        DateTime.utc(2026, 11, 1, 22, 51),
      );
    });
```

`TypeError` là `Error`, không phải `Exception`: `toApiFailure` của bài 20 ném lại nguyên, nên body thiếu `id` làm app lỗi thay vì thành `ParseFailure`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 2. Cùng một JSON có fare bằng 25500.5. Cột trái, bản mặc định: biểu thức (json['fare'] as num).toInt() cho fareVnd bằng 25500, không có lỗi nào được ném, tiền sai đi tiếp vào app. Cột phải, bản có converter: WholeVndConverter ném FormatException, decodeDto đổi thành DtoFormatException ở trường fare, guardApi đổi thành ParseFailure, màn hình báo dữ liệu từ máy chủ không đọc được.">
<defs><marker id="fl26-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker><marker id="fl26-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="16" y="24" class="t-b">Bản mặc định</text><text x="454" y="24" class="t-b">Bản có converter và decodeDto</text>
<rect x="16" y="40" width="410" height="64" rx="10" class="f-warn-soft s-warn"/><text x="30" y="78" class="t-mono">JSON: fare = 25500.5</text>
<rect x="16" y="128" width="410" height="64" rx="10" class="f-warn-soft s-warn"/><text x="30" y="166" class="t-mono">(json['fare'] as num).toInt()</text>
<rect x="16" y="216" width="410" height="64" rx="10" class="f-warn-soft s-warn"/><text x="30" y="244" class="t-mono">fareVnd = 25500</text><text x="30" y="266" class="t-small">phần .5 biến mất</text>
<rect x="16" y="304" width="410" height="64" rx="10" class="f-warn-soft s-warn"/><text x="30" y="332" class="t-b">Không có lỗi nào được ném</text><text x="30" y="354" class="t-small">tiền sai đi tiếp vào app</text>
<path d="M221 106 L221 126" class="f-none s-warn" stroke-width="2" marker-end="url(#fl26-2-w)"/>
<path d="M221 194 L221 214" class="f-none s-warn" stroke-width="2" marker-end="url(#fl26-2-w)"/>
<path d="M221 282 L221 302" class="f-none s-warn" stroke-width="2" marker-end="url(#fl26-2-w)"/>
<rect x="454" y="40" width="410" height="64" rx="10" class="f-pk-soft s-pk"/><text x="468" y="78" class="t-mono">JSON: fare = 25500.5</text>
<rect x="454" y="128" width="410" height="64" rx="10" class="f-pk-soft s-pk"/><text x="468" y="156" class="t-mono">WholeVndConverter.fromJson</text><text x="468" y="178" class="t-small">ném FormatException</text>
<rect x="454" y="216" width="410" height="64" rx="10" class="f-pk-soft s-pk"/><text x="468" y="244" class="t-mono">decodeDto</text><text x="468" y="266" class="t-small">thành DtoFormatException, trường fare</text>
<rect x="454" y="304" width="410" height="64" rx="10" class="f-pk-soft s-pk"/><text x="468" y="332" class="t-b">ParseFailure</text><text x="468" y="354" class="t-small">màn hình: Dữ liệu từ máy chủ không đọc được</text>
<path d="M659 106 L659 126" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-2-a)"/>
<path d="M659 194 L659 214" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-2-a)"/>
<path d="M659 282 L659 302" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Cùng một JSON đi qua hai bản. Mũi tên chỉ chiều xử lý, từ trên xuống. Cột trái kết thúc bằng một con số sai mà không ai biết; cột phải kết thúc bằng một lỗi có tên DTO, tên trường và nơi hiển thị.</figcaption>
</figure>

Hai công cụ để siết. `checked: true` trong `build.yaml` làm lỗi `fromJson` mang tên khoá (`CheckedFromJsonException`) thay vì `TypeError` trần. `JsonConverter` là lớp có `const` constructor, gắn vào field bằng annotation. Converter cho mét và số phiên bản:

### converters.dart · 26-codegen/lib/dto/converters.dart (trích)

```dart
/// Mét hoặc số phiên bản: `int` không âm, tối đa 2^53.
///
/// `json_serializable` mặc định đọc `int` bằng `(json as num).toInt()`, nên
/// `5000.5` thành `5000` và `-1` đi qua. Converter này từ chối cả hai, và từ
/// chối `5000.0` như `JsonReader.nonNegativeInt` của bản viết tay.
final class NonNegativeIntConverter implements JsonConverter<int, Object?> {
  const NonNegativeIntConverter();

  @override
  int fromJson(Object? json) {
    if (json is! int) {
      throw const FormatException('cần số nguyên');
    }
    if (json < 0) {
      throw const FormatException('không được âm');
    }
    if (json > maxSafeInteger) {
      throw const FormatException('quá lớn (tối đa 2^53)');
    }
    return json;
  }

  @override
  Object? toJson(int object) => object;
}
```

`toApiFailure` không hiểu `CheckedFromJsonException`, nên `decodeDto` dịch nó về `DtoFormatException`; phần còn lại của app không biết DTO nào do tay viết, DTO nào do máy sinh:

### decode.dart · 26-codegen/lib/dto/decode.dart (trích)

```dart
T decodeDto<T>(
  String dto,
  Object? json,
  T Function(Map<String, dynamic> json) build,
) {
  if (json is! Map<String, Object?>) {
    throw DtoFormatException(dto, r'$', 'cần một JSON object');
  }
  try {
    return build(json);
  } on CheckedFromJsonException catch (e) {
    throw DtoFormatException(
      dto,
      e.key ?? r'$',
      e.message ?? e.innerError.toString(),
    );
  }
}
```

Converter chỉ canh **biên JSON**. `copyWith(fareVnd: -1)` lọt qua constructor nhưng bị `toDomain` chặn. `copyWith(distanceMeters: -5)` lọt cả `toDomain`: ra `Trip` dài -0,005 km, chỉ `fromJson(toJson())` mới bắt được. `copyWith(currency: 'USD')` đi qua cả `toJson` lẫn `toDomain`, trong khi bản viết tay luôn ghi `VND`. Mỗi điều có một test riêng.

## 4. Chứng minh hai bản cùng hành vi trên cùng một JSON

Siết xong, ta cần bằng chứng. Test đặt cùng một JSON vào hai `fromJson` rồi quy kết quả về dạng so sánh được: `toJson()` nếu thành công, `DtoFormatException <DTO>.<trường>` nếu lỗi. Mọi loại lỗi khác làm test hỏng, vì chúng không thuộc hợp đồng lỗi.

### dto_parity_test.dart · 26-codegen/test/dto_parity_test.dart (trích)

```dart
Object outcome(Object? Function() decode) {
  try {
    final dto = decode()! as dynamic;
    return dto.toJson() as Object;
  } on hand.DtoFormatException catch (e) {
    return 'DtoFormatException ${e.dto}.${e.field}';
  }
}
```

### dto_parity_test.dart · 26-codegen/test/dto_parity_test.dart (trích)

```dart
    for (final entry in cases.entries) {
      test(entry.key, () {
        final fromHand = outcome(() => hand.TripDto.fromJson(entry.value));
        final fromGen = outcome(() => gen.TripDto.fromJson(entry.value));
        expect(fromGen, fromHand);
      });
    }
```

Bảng ca có 47 ca cho `TripDto`: 8 hợp lệ (cước `25500.0`, tên enum hoa thường, độ lệch `+07:00`, khoá thừa...) và 39 sai (thiếu khoá, sai kiểu, âm, lẻ, ngày tràn, không phải object), thêm 13 ca cho `QuoteDto`. Một test riêng đếm đúng 8 ca được nhận, để bảng không thể thành toàn ca lỗi mà vẫn xanh.

Hai DTO cũng đọc body thật của `tools/fake_api` và cho cùng kết quả: 5 km là 25.500 VND, 2,001 km là 12.005 VND, tính bằng `calculateFare`. `DioQuoteSource` đổi lỗi thành `ApiFailure`: 404 thành `ClientFailure`, 500 thành `ServerFailure`, cước lẻ thành `ParseFailure` ở trường `fare`.

Giới hạn. Bằng chứng chỉ phủ các ca đã liệt kê, và test chỉ so tên DTO và trường chứ không so câu chữ. Khi một JSON có hai lỗi cùng lúc, tên trường có thể khác: bản viết tay luôn kiểm `currency` trước, bản sinh mã báo trường đầu theo thứ tự khai báo, nên JSON thiếu `currency` và `id` được báo là `currency` và `id` (một test ghi lại điều này). Và converter lặp lại quy tắc của `JsonReader` vì `JsonReader` không nằm trong API công khai của `toi_noi_api`: một quy tắc, hai nơi giữ, và test so sánh là thứ báo động khi hai nơi lệch nhau.

## 5. riverpod_generator: viết hàm, sinh provider

Bài 18 khai báo provider bằng tay. Với `riverpod_generator`, ta viết hàm hoặc class thường và máy sinh provider. Lab có đủ ba dạng:

### providers.dart · 26-codegen/lib/providers.dart (trích)

```dart
/// Cổng tới backend. Cố ý không có giá trị mặc định: `main()` và test ghi đè
/// bằng `quoteSourceProvider.overrideWithValue(...)`.
@Riverpod(keepAlive: true)
QuoteSource quoteSource(Ref ref) => throw StateError(
  'quoteSourceProvider chưa được ghi đè: thêm '
  'quoteSourceProvider.overrideWithValue(source) vào ProviderScope.',
);

/// State của màn hình báo giá. Không có `keepAlive` nên là auto-dispose:
/// rời màn hình thì state bị bỏ.
@riverpod
class QuoteController extends _$QuoteController {
  @override
  QuoteScreenState build() => const QuoteScreenState.idle();

  /// Hỏi giá cho [input]. Lỗi đã phân loại thành [ApiFailure] nằm trong state.
  /// Lỗi khác (một bug) vẫn ném ra để không bị che, nhưng state quay về `idle`
  /// để màn hình không kẹt ở trạng thái đang tải.
  Future<void> fetch(QuoteInput input) async {
    state = QuoteScreenState.loading(input: input);
    final source = ref.read(quoteSourceProvider);
    try {
      final quote = await source.quote(
        serviceKind: input.serviceKind,
        distanceKm: input.distanceKm,
      );
      if (!ref.mounted) return;
      state = QuoteScreenState.ready(input: input, fare: quote.toMoney());
    } on ApiFailure catch (failure) {
      if (!ref.mounted) return;
      state = QuoteScreenState.failed(input: input, failure: failure);
    } catch (_) {
      if (ref.mounted) state = const QuoteScreenState.idle();
      rethrow;
    }
  }
}

/// Một chuyến theo mã. Tham số của hàm (`id`) trở thành tham số của provider:
/// `tripSnapshotProvider('T-1')` và `tripSnapshotProvider('T-2')` có state
/// riêng (family).
@riverpod
Future<TripSnapshot> tripSnapshot(Ref ref, String id) async {
  final dto = await ref.watch(quoteSourceProvider).trip(id);
  return dto.toSnapshot();
}
```

Tên provider sinh từ tên hàm hoặc class, và hậu tố `Notifier` bị bỏ (ta đã thử: class `FooNotifier` sinh `fooProvider`, hàm `barNotifier` sinh `barProvider`). Ở lab: `quoteSourceProvider`, `quoteControllerProvider`, `tripSnapshotProvider`. Mặc định provider là auto-dispose; `@Riverpod(keepAlive: true)` tắt nó. Hàm có tham số ngoài `Ref` thành family: `tripSnapshotProvider('T-1')` và `('T-2')` có state riêng. Test đọc các cờ do generator sinh:

### providers_test.dart · 26-codegen/test/providers_test.dart (trích)

```dart
    test('cờ do generator sinh: keepAlive thì không auto-dispose', () {
      expect(quoteSourceProvider.isAutoDispose, isFalse);
      expect(quoteControllerProvider.isAutoDispose, isTrue);
      expect(tripSnapshotProvider('T-1').isAutoDispose, isTrue);
    });
```

Generator không đổi ngữ nghĩa Riverpod 3 của bài 18: vẫn một loại `Ref`, vẫn `ref.mounted` sau `await`, thường cần `retry: (_, _) => null` trong `ProviderContainer.test` khi provider ném `Exception`. Test dưới đây kiểm thứ tự state của `QuoteController`, so sánh bằng `==` do `freezed` sinh:

### providers_test.dart · 26-codegen/test/providers_test.dart (trích)

```dart
    test('idle, loading rồi ready: đúng thứ tự, đúng dữ liệu', () async {
      final source = FakeQuoteSource();
      final container = _container(source);
      final seen = <QuoteScreenState>[];
      container.listen(
        quoteControllerProvider,
        (_, next) => seen.add(next),
        fireImmediately: true,
      );

      final done = container
          .read(quoteControllerProvider.notifier)
          .fetch(_input);
      source.answerQuote(0, calculateFare(5));
      await done;

      expect(seen, [
        const QuoteScreenState.idle(),
        const QuoteScreenState.loading(input: _input),
        QuoteScreenState.ready(input: _input, fare: calculateFare(5)),
      ]);
      expect(calculateFare(5), const Money(25500));
    });
```

Chi phí: `providers.g.dart` dài 214 dòng cho 54 dòng nguồn, và `riverpod_annotation` 4.0.8 ghim `riverpod` đúng 3.5.0 nên `flutter_riverpod` buộc ở 3.5.0. `flutter pub add` giải được cả bộ gói (analyzer 14.7.0) ngay lần đầu.

## 6. build_runner: watch, commit file sinh và kiểm không lệch

`build` sinh một lần, `watch` chạy nền và sinh lại mỗi khi lưu file:

```bash
dart run build_runner build
dart run build_runner watch
```

Cờ `--delete-conflicting-outputs` (hay `-d`) trong nhiều hướng dẫn cũ đã bị gỡ ở `build_runner` 2.17.1: lệnh in `These options have been removed and were ignored` rồi vẫn chạy. Lần chạy đầu biên dịch builder mất 131 giây trên máy kiểm, các lần sau 5 đến 20 giây (bậc độ lớn, lúc máy còn nhiều tiến trình khác).

Tuỳ chọn generator đặt trong `build.yaml`; lab bật `checked`:

### build.yaml · 26-codegen/build.yaml

```yaml
targets:
  $default:
    builders:
      json_serializable:
        options:
          # Lỗi fromJson mang tên khoá và tên lớp (CheckedFromJsonException)
          # thay vì TypeError trần.
          checked: true
```

Commit file sinh hay không? Hai cách đều hợp lệ:

| | Commit file sinh (lab này) | Không commit |
|---|---|---|
| Clone sạch rồi `flutter test` | xanh ngay | phải chạy `build_runner` trước, thiếu thì `uri_has_not_been_generated` |
| Diff của PR | dài: lab này 2.002 dòng sinh so với 633 dòng viết tay | chỉ có nguồn |
| Rủi ro chính | file sinh lệch nguồn; xung đột merge ở file sinh | máy mới hoặc CI quên bước build |
| Gợi ý của ta | lab, thư viện nhỏ, đội ít người | app lớn, nhiều người cùng sửa model |

Lab chọn commit để `flutter test` xanh từ trạng thái sạch (`generated_files_test.dart` giữ lời hứa đó). Rủi ro file sinh lệch nguồn được chặn bằng `--only-check` (từ `build_runner` 2.16), chạy trong CI (Hình 3):

```bash
dart run build_runner build --only-check
```

Cờ này không ghi, không xoá, chỉ so kết quả mong đợi với đĩa: sạch thì mã thoát 0; đổi nguồn hoặc sửa tay file sinh mà chưa sinh lại thì mã thoát 1 và in tên file lệch. Nó không cần cây git sạch. Ta đã thử cả ba tình huống và xác nhận file sinh không bị ghi. Cách phụ là `dart run build_runner build && git diff --exit-code -- lib`, nhưng nó ghi đè file và `git diff` không thấy file chưa theo dõi.

Một rủi ro lab không chặn được: `.gitignore` của lab bỏ `pubspec.lock`, còn generator khai bằng caret (`freezed: ^4.0.3`). Clone sạch có thể giải ra bản vá mới mà bản vá đổi code sinh, nên lệnh kiểm đỏ giả dù không ai sửa gì (suy từ changelog, ta chưa tái hiện). Dự án thật nên commit `pubspec.lock` và chạy `flutter pub get --enforce-lockfile` trong CI; lab giữ quy ước không commit lock của loạt bài.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 200" role="img" aria-label="Hình 3. Kiểm file sinh không lệch nguồn. Sửa nguồn rồi chạy build_runner build --only-check. Lệnh không ghi file, chỉ so với nội dung trên đĩa. Mã thoát 0 nghĩa là file sinh khớp nguồn. Mã thoát 1 nghĩa là nguồn hoặc file sinh đã đổi mà chưa sinh lại, và lệnh in tên file lệch.">
<defs><marker id="fl26-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl26-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="16" y="60" width="230" height="80" rx="10" class="f-phys-soft s-phys"/><text x="30" y="92" class="t-b">Sửa nguồn hoặc file sinh</text><text x="30" y="114" class="t-small">chưa chạy lại generator</text>
<rect x="306" y="60" width="270" height="80" rx="10" class="f-surface s-line"/><text x="320" y="88" class="t-mono">build_runner build</text><text x="320" y="110" class="t-mono">--only-check</text><text x="320" y="130" class="t-small">không ghi, chỉ so với đĩa</text>
<rect x="716" y="14" width="148" height="76" rx="10" class="f-pk-soft s-pk"/><text x="730" y="44" class="t-b">exit 0</text><text x="730" y="66" class="t-small">khớp nguồn</text>
<rect x="716" y="110" width="148" height="76" rx="10" class="f-warn-soft s-warn"/><text x="730" y="140" class="t-b">exit 1</text><text x="730" y="162" class="t-small">in tên file lệch</text>
<path d="M248 100 L304 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-3-a)"/>
<path d="M578 84 L714 58" class="f-none s-pk" stroke-width="2" marker-end="url(#fl26-3-a)"/>
<path d="M578 116 L714 140" class="f-none s-warn" stroke-width="2" marker-end="url(#fl26-3-w)"/>
</svg>
</div>
<figcaption>Hình 3. Lệnh kiểm file sinh. Mũi tên chỉ chiều chạy lệnh. Nhánh trên (mã thoát 0) là trạng thái mong muốn; nhánh dưới (mã thoát 1) chỉ ra nguồn và file sinh đã lệch nhau.</figcaption>
</figure>

`analysis_options.yaml` loại trừ file sinh khỏi `flutter analyze`:

### analysis_options.yaml · 26-codegen/analysis_options.yaml

```yaml
include: package:flutter_lints/flutter.yaml

analyzer:
  exclude:
    - build/**
    - android/**
    - ios/**
    # File do build_runner sinh: không phải mã ta viết, không phân tích, để
    # kết quả của analyze chỉ nói về code của ta. Hiện chúng vốn sạch, nhưng
    # điều đó phụ thuộc phiên bản generator và bộ lint (xem bài viết).
    - "**/*.g.dart"
    - "**/*.freezed.dart"
  errors:
    # `@JsonSerializable(...)` đặt trên factory của freezed (naive_trip_dto.dart)
    # bị cảnh báo này dù là cách dùng đúng theo tài liệu của freezed.
    invalid_annotation_target: ignore
```

`flutter analyze` ra 0 issue. Bỏ riêng phần loại trừ thì vẫn 0 issue vì file sinh hiện đang sạch; loại trừ chỉ giữ kết quả nói về code ta viết khi generator đổi phiên bản. Dòng `invalid_annotation_target: ignore` mới có tác dụng ngay: bỏ nó thì analyze báo 1 issue, do `@JsonSerializable` đặt trên `factory` trong `naive_trip_dto.dart`.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp khi dùng code generation</span>
<ul>
<li><strong>Quên chạy <code>build_runner</code> hoặc quên commit file sinh.</strong> Dấu hiệu: analyzer báo <code>uri_has_not_been_generated</code>, hoặc biên dịch báo <code>Undefined name '_$TripDtoFromJson'</code>. Cách sửa: chạy <code>build</code>, commit file sinh, chạy <code>build --only-check</code> trong CI. Khi hai nhánh cùng sửa nguồn, đừng merge file sinh bằng tay: hợp nhất nguồn rồi sinh lại.</li>
<li><strong>Lớp <code>freezed</code> thiếu <code>abstract</code> hoặc <code>sealed</code>.</strong> Dấu hiệu: <code>non_abstract_class_inherits_abstract_member</code> ở lớp một biến thể, hoặc <code>non_exhaustive_switch_expression</code> ở mọi <code>switch</code> trên union. Cách sửa: thêm từ khoá đúng.</li>
<li><strong>Tin rằng <code>fromJson</code> sinh ra là an toàn.</strong> Dấu hiệu: <code>distanceMeters</code> hay <code>fare</code> mất phần lẻ, ngày <code>2026-13-45</code> thành <code>2027-02-14</code>, <code>TypeError</code> chui ra tận UI. Nguyên nhân: code sinh theo kiểu Dart, không theo hợp đồng API. Cách sửa: converter, <code>checked: true</code>, <code>decodeDto</code> và test so sánh.</li>
<li><strong>Nghĩ converter canh cả constructor.</strong> Dấu hiệu: <code>copyWith(distanceMeters: -5)</code> chạy được và <code>toDomain</code> trả <code>Trip</code> dài -0,005 km. Nguyên nhân: converter chỉ chạy trong <code>fromJson</code> và <code>toJson</code>. Cách sửa: chặn ở biên JSON và kiểm bất biến trong domain, đừng coi DTO là nơi giữ bất biến nghiệp vụ.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/26-codegen
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3: `flutter analyze` 0 issue, `flutter test` **134 test xanh**, không cần chạy `build_runner` vì file sinh đã nằm trong repo. Mức: **Kiểm chứng tự động**, không cần thiết bị hay mạng ngoài (báo giá đi qua `InProcessAdapter` vào fake API trong tiến trình). Để sinh lại và kiểm không lệch, dùng các lệnh ở mục 6.

Chín file test chia theo mục của bài: so sánh với bản viết tay (`dto_parity_test.dart`, `fake_api_json_test.dart`), bản mặc định (`naive_dto_test.dart`), hành vi do `freezed` sinh (`dto_behaviour_test.dart`, `quote_state_test.dart`), provider và màn hình (`providers_test.dart`, `quote_screen_test.dart`), và `generated_files_test.dart` cho file sinh.

**Chưa làm**: chưa chạy trên simulator, emulator hay thiết bị; chưa thay DTO trong `toi_noi_api` bằng bản sinh mã (lab chỉ chứng minh hai bản tương đương, package chung không đổi); `QuoteRequestDto` và `TripSnapshot` vẫn viết tay; chưa có CI chạy lệnh kiểm file sinh (bài 33); chưa dùng `riverpod_lint` (bài 32).

## Kết Phần 3: đối chiếu mục tiêu

Mục tiêu Phần 3: app khách gọi được backend, có state rõ ràng bằng Riverpod, lưu cục bộ, đăng nhập, cấu hình ba môi trường và kiến trúc tách lớp. Đối chiếu:

| Mục tiêu | Bài | Kết quả |
|---|---|---|
| Gọi backend | [19](/articles/flutter-http-dio-json-dto-goi-api-bao-gia), [20](/articles/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong) | `dio`, DTO, `ApiFailure`, retry; bài 26: DTO sinh mã giữ nguyên hợp đồng |
| State rõ ràng bằng Riverpod | [17](/articles/flutter-state-inheritedwidget-changenotifier-bao-gia), [18](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen), 26 | `ChangeNotifier`, Notifier; bài 26: union `sealed`, `@riverpod` |
| Lưu cục bộ | [22](/articles/flutter-local-persistence-drift-secure-storage-lich-su-offline) | `drift`, `shared_preferences`, secure storage |
| Đăng nhập | [24](/articles/flutter-authentication-otp-jwt-refresh-token-secure-storage) | OTP, JWT, refresh token |
| Ba môi trường | [25](/articles/flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod) | `--dart-define-from-file`, `AppConfig`, override theo môi trường |
| Kiến trúc tách lớp | [21](/articles/flutter-clean-architecture-domain-data-presentation-mobile), [23](/articles/flutter-go-router-deep-link-shellroute-guard) | domain, data, presentation; điều hướng khai báo |

Còn hai điều chưa đạt: backend của lab là fake API (trip-service .NET thật là tuỳ chọn, bài này không gọi nó), và mỗi lab của Phần 3 là một project độc lập, chưa gộp thành app chạy đủ cả sáu việc (bắt đầu ở bài 43). Mọi kiểm chứng là mức tự động, chưa chạy trên thiết bị.

## Bài tập

1. Thêm `String? note` vào `TripDto`: sửa mấy chỗ, chạy lệnh gì, test nào đỏ?
2. Server gửi `fare` dạng chuỗi `"25500"`. Hai bản phản ứng thế nào, và sửa gì để nhận được?
3. Thêm nhánh `QuoteScreenState.offline()` vào union. Chuyện gì xảy ra với `quote_screen.dart` và `quote_state_test.dart`?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Một chỗ: thêm <code>String? note,</code> vào <code>factory TripDto</code>. Chạy <code>dart run build_runner build</code> rồi commit cả file sinh. Chạy <code>flutter test</code> mà chưa build thì không biên dịch được. Sau khi build, ta đã thử và có 11 test đỏ: 8 ca hợp lệ của <code>dto_parity_test.dart</code>, 2 test <code>toJson</code> của <code>dto_behaviour_test.dart</code> và test <code>đặt chuyến, đọc lại, tài xế nhận</code> của <code>fake_api_json_test.dart</code>, vì <code>toJson</code> của bản sinh thêm khoá <code>note: null</code> mà bản viết tay không có. Hai bản không còn cùng hợp đồng cho tới khi bản viết tay sửa theo.</p>
<p><strong>2.</strong> Bản mặc định ném <code>TypeError</code>; bản có converter ném <code>FormatException</code>, thành <code>ParseFailure</code> ở trường <code>fare</code>. Để nhận chuỗi, thêm nhánh <code>json is String</code> với <code>int.tryParse</code> vào <code>WholeVndConverter</code>, vẫn từ chối số âm hoặc quá 2^53. Ca <code>cước là chuỗi</code> trong bảng so sánh sẽ đỏ vì bản viết tay vẫn từ chối: đổi hợp đồng thì đổi cả hai bên.</p>
<p><strong>3.</strong> Sau khi chạy lại <code>build_runner</code>, ta đã thử và analyzer báo ba lỗi: <code>non_exhaustive_switch_expression</code> ở <code>switch</code> trong <code>quote_screen.dart</code>, cùng lỗi đó ở hàm <code>describe</code> trong <code>quote_state_test.dart</code>, và <code>missing_required_argument</code> (thiếu tham số <code>offline</code>) ở lời gọi <code>state.when</code> của test <code>when và map vẫn có</code>. <code>_describe(ApiFailure)</code> không đổi vì nó <code>switch</code> trên <code>ApiFailure</code>.</p>
</details>

Bài trước: [Cấu hình môi trường: flavors, --dart-define-from-file và DI theo môi trường](/articles/flutter-flavors-dart-define-cau-hinh-moi-truong-dev-staging-prod). Bài tiếp: [Unit test với mocktail và fake: quy tắc cước và use case](/articles/flutter-unit-test-mocktail-fake-quy-tac-cuoc-use-case).

## Tài liệu gốc

- [freezed](https://pub.dev/packages/freezed) và [json_serializable](https://pub.dev/packages/json_serializable) (pub.dev).
- [JsonConverter](https://pub.dev/documentation/json_annotation/latest/json_annotation/JsonConverter-class.html) và [json_annotation](https://pub.dev/packages/json_annotation) (pub.dev).
- [riverpod_generator](https://pub.dev/packages/riverpod_generator), [riverpod_annotation](https://pub.dev/packages/riverpod_annotation) và [riverpod.dev](https://riverpod.dev/) (pub.dev, riverpod.dev).
- [build_runner](https://pub.dev/packages/build_runner) và [Using build_runner](https://dart.dev/tools/build_runner) (pub.dev, dart.dev).
- [Patterns](https://dart.dev/language/patterns) và [Class modifiers](https://dart.dev/language/class-modifiers) (dart.dev).
- [JSON and serialization](https://docs.flutter.dev/data-and-backend/serialization/json) (docs.flutter.dev).
