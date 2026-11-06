---
slug: flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen
title: "Riverpod: Provider, Notifier và AsyncNotifier cho luồng đặt chuyến"
summary: "Bài 18: dựng luồng đặt chuyến bằng flutter_riverpod 3.5: ProviderScope, Provider, Notifier, FutureProvider và AsyncNotifier, ref.watch/read/listen, AsyncValue, family, auto-dispose, retry mặc định và override để test. Có 87 test, kiểm các phát biểu về retry và dispose."
category: programming
level: intermediate
tags: [flutter, dart, mobile, toi-noi, riverpod, state-management]
language: vi
---

Khách mở app Tới Nơi, chọn **Giao đồ ăn** và nâng quãng đường lên 12 km. Ba chỗ trên màn hình phải đổi cùng lúc: chip dịch vụ, thẻ báo giá và nút Đặt chuyến. Báo giá nằm trên server nên còn ba tình huống nữa: đang chờ, thất bại, và khách đổi ý khi câu trả lời cũ chưa về. Bài trước đưa state lên cao bằng `ChangeNotifier`, nhưng phần chờ, lỗi, thử lại và bỏ việc đang dở vẫn do ta tự viết.

Riverpod lo đúng phần đó. Mỗi mảnh state là một **provider** có tên, provider này đọc provider kia và tự tính lại khi nguồn đổi, còn lỗi và trạng thái chờ được gói trong `AsyncValue`. Đây là bài 18 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 17 về InheritedWidget và ChangeNotifier](/articles/flutter-state-inheritedwidget-changenotifier-bao-gia) trước, và nắm `Future` ở [bài 05](/articles/dart-future-async-await-stream-bao-gia-va-vi-tri). Mọi ví dụ dùng `Trip`, `Money` và `calculateFare` của [`toi_noi_core`](/articles/dart-package-pubspec-lint-test-toi-noi-core), bảng giá là số minh hoạ của hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi).

Bài dựa trên `flutter_riverpod` **3.5.0**. Nhiều hướng dẫn trên mạng viết cho Riverpod 2 và không còn đúng, nên mọi câu về retry, dispose và `mounted` dưới đây có test hoặc mã nguồn package làm chứng.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p><code>quoteProvider</code> là một <code>FutureProvider</code> và ta không cấu hình gì về retry. Server báo bận bằng <code>QuoteUnavailable</code> (một <code>Exception</code>) ở mọi lần gọi. Sau bao lâu màn hình báo giá mới hiện lỗi?</p>
<details><summary>Xem đáp án</summary><p>Khoảng <strong>38 giây</strong>, không phải ngay. Riverpod 3 tự thử lại provider lỗi tối đa 10 lần, chờ 200, 400, 800, 1600, 3200 ms rồi 6400 ms cho các lần còn lại, tổng 11 lần gọi và 38,2 giây. Suốt thời gian đó state là <code>AsyncLoading</code> mang theo lỗi cũ. Test <code>Exception: 11 lần gọi trong 38,2 giây rồi mới thành AsyncError</code> chứng minh con số này; mục 6 cho cách đặt chính sách riêng.</p></details>
</div>

## 1. Provider và ProviderScope: khai báo ở ngoài, state ở trong

Một provider là **bản khai báo** cách tạo một giá trị. Biến `final quoteRepositoryProvider = Provider(...)` chỉ là chìa khoá. Giá trị thật nằm trong một `ProviderContainer`, và `ProviderScope` tạo container đó rồi đặt lên cây widget. Vì vậy hai container gốc độc lập (hai `ProviderScope` không lồng nhau) có hai bộ state tách biệt dù dùng chung biến khai báo; test tận dụng điều đó để ghi đè provider mà không đụng code chạy thật.

Provider đầu tiên của lab là cổng tới backend. Màn hình và các provider khác chỉ biết interface, chưa biết HTTP:

### quote_repository.dart · 18-riverpod/lib/quote_repository.dart (trích)

```dart
abstract interface class QuoteRepository {
  /// Lấy báo giá cho [serviceKind] và [distanceKm].
  ///
  /// Ném [QuoteUnavailable] khi dịch vụ bận.
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  });

  /// Đặt chuyến từ [quote]; trả về chuyến ở trạng thái `requested`.
  Future<Trip> placeTrip(Quote quote);

  /// Đọc một chuyến; ném [TripNotFound] nếu không có.
  Future<Trip> fetchTrip(String id);
}
```

### providers.dart · 18-riverpod/lib/providers.dart (trích)

```dart
final quoteRepositoryProvider = Provider<QuoteRepository>(
  (ref) => FakeQuoteRepository(
    latency: const Duration(milliseconds: 400),
    unavailableAboveKm: 30,
  ),
);
```

### main.dart · 18-riverpod/lib/main.dart

```dart
void main() {
  // ProviderScope giữ state của mọi provider; phải nằm trên MaterialApp.
  runApp(const ProviderScope(child: ToiNoiApp()));
}
```

`ProviderScope` phải nằm trên `MaterialApp` để mọi màn hình tìm được container. Thiếu nó, widget đọc provider ném `StateError` có thông điệp `No ProviderScope found` (test `thiếu ProviderScope thì lỗi ngay khi build`).

Hình 1 vẽ các provider của lab. Mũi tên chỉ từ bên **dùng** sang bên **được dùng**. Khi test, ta thay đúng một nút ở đáy sơ đồ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 372" role="img" aria-label="Hình 1. Đồ thị phụ thuộc. BookingScreen dùng quoteProvider và placeTripProvider, TripScreen dùng tripByIdProvider. quoteProvider dùng tripDraftProvider và quoteRepositoryProvider. placeTripProvider và tripByIdProvider dùng quoteRepositoryProvider. Tất cả provider nằm trong một ProviderContainer do ProviderScope tạo. Khi test, overrideWithValue thay quoteRepositoryProvider.">
<defs><marker id="fl18-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl18-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="240" y="16" width="520" height="330" rx="12" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 5"/><text x="256" y="38" class="t-small">ProviderContainer: nơi giữ state</text>
<rect x="16" y="96" width="190" height="74" rx="10" class="f-phys-soft s-phys"/><text x="30" y="126" class="t-b">BookingScreen</text><text x="30" y="150" class="t-small">ConsumerWidget</text>
<rect x="16" y="250" width="190" height="74" rx="10" class="f-phys-soft s-phys"/><text x="30" y="280" class="t-b">TripScreen</text><text x="30" y="304" class="t-small">ConsumerWidget</text>
<rect x="260" y="56" width="210" height="58" rx="10" class="f-pk-soft s-pk"/><text x="274" y="82" class="t-mono">quoteProvider</text><text x="274" y="102" class="t-small">FutureProvider</text>
<rect x="260" y="140" width="210" height="58" rx="10" class="f-pk-soft s-pk"/><text x="274" y="166" class="t-mono">placeTripProvider</text><text x="274" y="186" class="t-small">AsyncNotifier</text>
<rect x="260" y="250" width="210" height="58" rx="10" class="f-pk-soft s-pk"/><text x="274" y="276" class="t-mono">tripByIdProvider(id)</text><text x="274" y="296" class="t-small">FutureProvider.family</text>
<rect x="530" y="56" width="210" height="58" rx="10" class="f-pk-soft s-pk"/><text x="544" y="82" class="t-mono">tripDraftProvider</text><text x="544" y="102" class="t-small">Notifier</text>
<rect x="530" y="170" width="210" height="82" rx="10" class="f-pk-soft s-pk"/><text x="544" y="196" class="t-mono">quoteRepositoryProvider</text><text x="544" y="216" class="t-small">Provider</text><text x="544" y="236" class="t-small">Fake khi chạy, Stub khi test</text>
<path d="M208 112 L258 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-1-a)"/>
<path d="M208 152 L258 169" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-1-a)"/>
<path d="M208 285 L258 279" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-1-a)"/>
<path d="M472 85 L528 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-1-a)"/>
<path d="M450 116 L528 188" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-1-a)"/>
<path d="M472 172 L528 200" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-1-a)"/>
<path d="M472 270 L528 236" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-1-a)"/>
<rect x="530" y="280" width="210" height="54" rx="10" class="f-warn-soft s-warn"/><text x="544" y="304" class="t-b">Khi test</text><text x="544" y="322" class="t-mono">overrideWithValue(stub)</text>
<path d="M635 280 L635 254" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl18-1-w)"/>
<text x="16" y="364" class="t-small">Mũi tên đi từ bên gọi ref.watch hoặc ref.read tới bên được gọi.</text>
</svg>
</div>
<figcaption>Hình 1. Các provider của lab và chiều phụ thuộc. Ghi đè một provider trong ProviderScope (hoặc ProviderContainer) thay cả nhánh dùng nó mà không sửa code.</figcaption>
</figure>

Ghi đè nằm ngay trong `ProviderScope`. Hàm dựng app của test chỉ khác `main()` ở tham số `overrides`:

### support.dart · 18-riverpod/test/support.dart (trích)

```dart
Future<ProviderContainer> pumpApp(
  WidgetTester tester, {
  required QuoteRepository repository,
  List<Override> overrides = const [],
}) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        quoteRepositoryProvider.overrideWithValue(repository),
        ...overrides,
      ],
      child: const ToiNoiApp(),
    ),
  );
  return ProviderScope.containerOf(tester.element(find.byType(BookingScreen)));
}
```

Test không cần widget dùng `ProviderContainer.test(overrides: [...])`, tự `dispose` cuối mỗi test.

Một vài điều khác tài liệu cũ, đã đối chiếu với `CHANGELOG.md` và mã nguồn của `riverpod` 3.5.0:

| Cách viết cũ (Riverpod 2) | Riverpod 3.5.0 |
|---|---|
| `FutureProviderRef`, `AutoDisposeNotifier`, `FamilyNotifier` | Chỉ còn một `Ref`, `Notifier` và `AsyncNotifier`; tham số của family đi vào hàm tạo (chữ ký trong `builder.dart`) |
| Provider lỗi thì dừng ở lỗi | Tự thử lại theo chính sách mặc định (mục 6) |
| Cách lọc cập nhật của Riverpod 2 | Theo mục 3.0.0-dev.12 của changelog, mọi provider so state mới với cũ bằng `==` rồi mới báo listener; muốn hành vi cũ thì ghi đè `updateShouldNotify` trong Notifier |
| `StateProvider`, `StateNotifierProvider`, `ChangeNotifierProvider` trong `flutter_riverpod.dart` | Chuyển sang `package:flutter_riverpod/legacy.dart`; kiểu `Override`, `ProviderException` nằm ở `misc.dart` |

## 2. Notifier: state đồng bộ có method

Bản nháp của khách (dịch vụ, quãng đường) là state đồng bộ có vài thao tác, nên hợp với `Notifier`. `TripDraft` bất biến và định nghĩa `==` theo giá trị; `build()` trả giá trị khởi đầu; mọi thay đổi đi qua method và gán `state`:

### trip_draft.dart · 18-riverpod/lib/trip_draft.dart (trích)

```dart
class TripDraftNotifier extends Notifier<TripDraft> {
  @override
  TripDraft build() =>
      const TripDraft(serviceKind: ServiceKind.ride, distanceKm: 5);

  void selectService(ServiceKind kind) {
    state = state.copyWith(serviceKind: kind);
  }

  /// Đặt quãng đường; ném [ArgumentError] khi ngoài miền 0..100 km của bảng
  /// giá (kể cả NaN) và giữ nguyên bản nháp.
  void setDistanceKm(double km) {
    final isValid = km >= 0 && km <= FarePolicy.maxKm;
    if (!isValid) {
      throw ArgumentError.value(
        km,
        'km',
        'phải nằm trong khoảng 0..${FarePolicy.maxKm} km',
      );
    }
    state = state.copyWith(distanceKm: km);
  }
}

final tripDraftProvider = NotifierProvider<TripDraftNotifier, TripDraft>(
  TripDraftNotifier.new,
);
```

Mỗi lần gán `state`, Riverpod so giá trị mới với giá trị cũ bằng `==` và chỉ báo listener khi khác. Vì thế `TripDraft` phải có `==`, và gán lại đúng bản nháp cũ không làm ai build lại. Test `đặt lại đúng giá trị cũ không báo listener` kiểm cả hai nhánh. Quãng đường ngoài miền 0 đến 100 km ném `ArgumentError` và để nguyên state; test thử -0,001, 100,001, NaN và vô cực.

Widget gọi method qua `ref.read(tripDraftProvider.notifier)`; notifier ổn định giữa các lần đọc, còn `ref.watch(tripDraftProvider)` mới trả về state.

## 3. watch, read và listen: chọn theo việc cần làm

Ba cách đọc provider khác nhau ở việc có đăng ký theo dõi hay không, và khi state đổi thì chuyện gì xảy ra (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 2. Khi state của tripDraftProvider đổi, ref.watch làm widget build lại hoặc provider tính lại, ref.read không nhận thông báo nào, ref.listen gọi callback với giá trị trước và sau.">
<defs><marker id="fl18-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl18-2-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="120" width="240" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="154" class="t-mono">tripDraftProvider</text><text x="36" y="178" class="t-small">5 km đổi thành 6 km</text><text x="36" y="198" class="t-small">và == báo khác nhau</text>
<rect x="340" y="16" width="520" height="100" rx="10" class="f-pk-soft s-pk"/><text x="356" y="44" class="t-mono t-pk">ref.watch(provider)</text><text x="356" y="68" class="t-small">Ở đâu: build của widget, thân provider</text><text x="356" y="88" class="t-small">Khi state đổi: build lại, hoặc tính lại provider</text><text x="356" y="108" class="t-small">Dùng cho: giao diện và giá trị dẫn xuất</text>
<rect x="340" y="130" width="520" height="100" rx="10" class="f-surface s-muted"/><text x="356" y="158" class="t-mono">ref.read(provider)</text><text x="356" y="182" class="t-small">Ở đâu: onPressed, method của notifier</text><text x="356" y="202" class="t-small">Khi state đổi: không nhận gì, giá trị đứng yên</text><text x="356" y="222" class="t-small">Dùng cho: gọi method, lấy notifier một lần</text>
<rect x="340" y="244" width="520" height="100" rx="10" class="f-warn-soft s-warn"/><text x="356" y="272" class="t-mono t-warn">ref.listen(provider, callback)</text><text x="356" y="296" class="t-small">Ở đâu: build của ConsumerWidget (hoặc listenManual)</text><text x="356" y="316" class="t-small">Khi state đổi: gọi callback(previous, next)</text><text x="356" y="336" class="t-small">Dùng cho: SnackBar, điều hướng, ghi log</text>
<path d="M262 150 L338 70" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-2-a)"/>
<path d="M262 170 L338 180" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl18-2-m)"/>
<path d="M262 190 L338 290" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Một thay đổi state đi tới ba người đọc. Đường liền là có thông báo, đường đứt là không có. Mũi tên ở hình này chỉ chiều thông báo, ngược với Hình 1.</figcaption>
</figure>

`BookingScreen` dùng đủ ba cách, mỗi cách đúng chỗ của nó:

### booking_screen.dart · 18-riverpod/lib/booking_screen.dart (trích)

```dart
    // listen: việc phụ (SnackBar, chuyển trang) khi state đổi. Chỉ chạy khi
    // có thay đổi, không chạy ở lần build đầu.
    ref.listen(quoteProvider, (previous, next) {
      // Trong lúc thử lại, state là AsyncLoading nhưng vẫn mang lỗi cũ;
      // chỉ báo khi thật sự hết lượt.
      if (next.hasError && !next.isLoading) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Không lấy được báo giá: ${describeError(next.error!)}',
            ),
          ),
        );
      }
    });
    ref.listen(placeTripProvider, (previous, next) {
      switch (next) {
        case AsyncData(value: final trip?):
          Navigator.of(context).push(
            MaterialPageRoute<void>(
              builder: (_) => TripScreen(tripId: trip.id),
            ),
          );
        case AsyncError(:final error) when !next.isLoading:
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Không đặt được chuyến: ${describeError(error)}'),
            ),
          );
        case _:
          break;
      }
    });

    // watch: phần giao diện phụ thuộc state; state đổi thì build lại.
    final quote = ref.watch(quoteProvider);
    final placing = ref.watch(placeTripProvider).isLoading;
```

Hai `ref.listen` làm việc phụ: SnackBar khi báo giá lỗi, mở màn hình chuyến khi đặt xong. Đừng gọi `showSnackBar` thẳng trong `build`, vì `build` có thể chạy lại nhiều lần. Callback của `listen` chỉ chạy khi state đổi, không chạy ở lần dựng đầu (test `ref.listen không chạy ở lần build đầu, chỉ khi state đổi`).

`ref.watch` ở gốc `build` đăng ký theo dõi: state đổi là widget dựng lại. `select` thu hẹp phạm vi đó:

### booking_screen.dart · 18-riverpod/lib/booking_screen.dart (trích)

```dart
class ServicePicker extends ConsumerWidget {
  const ServicePicker({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selected = ref.watch(
      tripDraftProvider.select((draft) => draft.serviceKind),
    );
    return Wrap(
      spacing: ToiNoiSpacing.sm,
      children: [
        for (final kind in ServiceKind.values)
          ChoiceChip(
            label: Text(kind.label),
            selected: kind == selected,
            onSelected: (_) =>
                ref.read(tripDraftProvider.notifier).selectService(kind),
          ),
      ],
    );
  }
}
```

`ServicePicker` chỉ theo dõi `serviceKind`, nên đổi quãng đường không dựng lại nó. Test đếm số lần `builder` chạy xác nhận điều đó.

`ref.read` đọc một lần: đúng chỗ trong `onPressed`, nhưng trong `build` nó đóng băng giá trị. Test dựng hai `Consumer` rồi đổi quãng đường lên 9 km: widget dùng `watch` hiện 9.0, widget dùng `read` vẫn hiện 5.0. Provider cũng vậy: provider `watch` bản nháp tính lại, provider `read` giữ giá trị cũ.

## 4. AsyncValue: FutureProvider và AsyncNotifier

Báo giá đến từ `Future`, nên `quoteProvider` là `FutureProvider`. Hai dòng `ref.watch` trong thân biến nó thành hàm của bản nháp và repository:

### providers.dart · 18-riverpod/lib/providers.dart (trích)

```dart
final quoteProvider = FutureProvider.autoDispose<Quote>((ref) {
  final draft = ref.watch(tripDraftProvider);
  final repository = ref.watch(quoteRepositoryProvider);
  return repository.fetchQuote(
    serviceKind: draft.serviceKind,
    distanceKm: draft.distanceKm,
  );
}, retry: retryTransientOnce);
```

Khách bấm `+`, bản nháp đổi, provider tính lại và hỏi km mới. Câu trả lời cũ về muộn bị bỏ: test `phản hồi của lần hỏi cũ về muộn...` hoàn thành lần hỏi 7 km trước rồi lần 5 km sau, và listener chỉ thấy dữ liệu 7 km.

Kiểu `AsyncValue<Quote>` là `sealed class` với ba lớp con, nên ta dùng `switch` vét cạn. Bốn tình huống của báo giá rơi vào ba nhánh (Hình 3):

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Các trạng thái của AsyncValue. AsyncLoading chuyển sang AsyncData khi Future hoàn thành, sang AsyncError lỗi, hết lượt thử lại. Khi còn lượt, AsyncLoading mang lỗi cũ và retrying bằng true. Sau khi đã có state, đổi bản nháp cho AsyncLoading giữ value cũ, và invalidate sau lỗi cho AsyncError với isRefreshing.">
<defs><marker id="fl18-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl18-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="20" y="40" class="t-small">lỗi còn lượt: retrying = true</text>
<path d="M60 110 C 40 60, 200 60, 190 108" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl18-3-w)"/>
<rect x="20" y="112" width="220" height="100" rx="10" class="f-phys-soft s-phys"/><text x="36" y="142" class="t-b">AsyncLoading</text><text x="36" y="166" class="t-small">chưa có kết quả</text><text x="36" y="186" class="t-small">isLoading = true</text>
<rect x="340" y="30" width="240" height="100" rx="10" class="f-pk-soft s-pk"/><text x="356" y="60" class="t-b">AsyncData(value)</text><text x="356" y="84" class="t-small">khi Future hoàn thành</text><text x="356" y="104" class="t-small">hiện QuoteCard</text>
<rect x="340" y="194" width="240" height="100" rx="10" class="f-warn-soft s-warn"/><text x="356" y="224" class="t-b">AsyncError(error)</text><text x="356" y="248" class="t-small">khi lỗi và hết lượt thử lại</text><text x="356" y="268" class="t-small">hiện lỗi và nút Thử lại</text>
<path d="M242 136 L338 90" class="f-none s-pk" stroke-width="2" marker-end="url(#fl18-3-a)"/>
<path d="M242 190 L338 236" class="f-none s-warn" stroke-width="2" marker-end="url(#fl18-3-w)"/>
<rect x="600" y="30" width="260" height="264" rx="10" class="f-surface s-line"/>
<text x="616" y="60" class="t-b">Khi đã có state</text>
<text x="616" y="88" class="t-small">Bản nháp đổi:</text><text x="616" y="108" class="t-small">AsyncLoading giữ value cũ,</text><text x="616" y="128" class="t-small">isReloading = true</text>
<text x="616" y="160" class="t-small">invalidate sau lỗi:</text><text x="616" y="180" class="t-small">vẫn là AsyncError nhưng</text><text x="616" y="200" class="t-small">isRefreshing = true</text>
<text x="616" y="232" class="t-small">AsyncError cũng giữ value</text><text x="616" y="252" class="t-small">của lần thành công trước,</text><text x="616" y="272" class="t-small">có thể của bản nháp khác</text>
</svg>
</div>
<figcaption>Hình 3. Bốn tình huống của AsyncValue trong quoteProvider. Ba hộp bên trái là ba nhánh của switch; cột bên phải là các trạng thái "pha trộn" đã đo bằng test.</figcaption>
</figure>

Switch của màn hình xử lý đủ cả ba lớp con, và dùng thêm cờ để phân biệt các trạng thái pha trộn:

### booking_screen.dart · 18-riverpod/lib/booking_screen.dart (trích)

```dart
          switch (quote) {
            AsyncData(:final value) => QuoteCard(
              quote: value,
              onBook: placing
                  ? null
                  // read: trong handler chỉ cần lấy notifier, không đăng ký
                  // theo dõi.
                  : () => ref.read(placeTripProvider.notifier).place(value),
            ),
            AsyncError(:final error) when !quote.isLoading => QuoteError(
              message: describeError(error),
              onRetry: () => ref.invalidate(quoteProvider),
            ),
            AsyncError() ||
            AsyncLoading() => QuoteLoading(retrying: quote.retrying),
          },
```

Nhánh `AsyncError() when !quote.isLoading` chỉ hiện lỗi khi đã hết lượt thử lại. Khi `invalidate` sau lỗi, state vẫn là `AsyncError` nhưng `isRefreshing` bằng `true`; nhánh cuối gom nó với `AsyncLoading` nên khách thấy spinner thay vì lỗi cũ (test `bấm Thử lại`). Vì `AsyncError` giữ `value` của lần trước, màn hình không đọc `.value` trong nhánh lỗi: test cho báo giá 5 km thành công rồi 6 km lỗi, `value` vẫn là 25.500 VND nhưng giao diện không hiện nó.

Với thao tác có method, ta dùng `AsyncNotifier`. Đặt chuyến không phải thứ để tính lại tự động mà là hành động của khách:

### providers.dart · 18-riverpod/lib/providers.dart (trích)

```dart
class PlaceTripNotifier extends AsyncNotifier<Trip?> {
  @override
  Trip? build() => null;

  /// Gửi [quote] lên server. Không tự thử lại: đặt chuyến không idempotent.
  Future<void> place(Quote quote) async {
    state = const AsyncLoading();
    // Trong method, đọc bằng `read`: ta chỉ dùng giá trị một lần, không cần
    // provider tính lại khi repository đổi.
    final repository = ref.read(quoteRepositoryProvider);
    final result = await AsyncValue.guard(() => repository.placeTrip(quote));
    // Provider có thể đã bị dispose trong lúc chờ (khách rời màn hình);
    // đụng vào `state` lúc đó sẽ ném lỗi.
    if (!ref.mounted) return;
    state = result;
  }
}

final placeTripProvider =
    AsyncNotifierProvider.autoDispose<PlaceTripNotifier, Trip?>(
      PlaceTripNotifier.new,
    );
```

`AsyncValue.guard` bọc `try/catch` và trả `AsyncData` hoặc `AsyncError`. Quy tắc chọn: UI không gọi method nào thì dùng `FutureProvider`; có method như `place` thì dùng `AsyncNotifier`. `FutureProvider` có `overrideWithValue`, nên test dựng ba trạng thái màn hình mà không cần repository.

## 5. Family và auto-dispose: state theo tham số, tự dọn

Màn hình chuyến cần đọc một chuyến theo mã. `family` thêm một tham số vào provider: mỗi giá trị tham số có state riêng.

### providers.dart · 18-riverpod/lib/providers.dart (trích)

```dart
final tripByIdProvider = FutureProvider.autoDispose.family<Trip, String>((
  ref,
  id,
) {
  return ref.watch(quoteRepositoryProvider).fetchTrip(id);
}, retry: retryTransientOnce);
```

`tripByIdProvider('T-1')` bằng `tripByIdProvider('T-1')` và khác `tripByIdProvider('T-2')`; hai nơi cùng nghe một mã dùng chung một lần gọi repository (test `mỗi id có state riêng`). Ghi đè một thành viên cũng được: `tripByIdProvider('T-9').overrideWithValue(AsyncData(chuyến))`, hoặc cả family bằng `overrideWith((ref, id) => ...)`.

**Auto-dispose** quyết định state sống bao lâu. Với `.autoDispose` hoặc `isAutoDispose: true` (hai cách viết cho cùng hành vi, test kiểm cả hai), state bị bỏ khi không còn ai nghe. Theo ghi chú trong mã nguồn của scheduler, provider chỉ bị bỏ khi đã trọn một khung hình không có người nghe, chứ không phải ngay lúc người nghe cuối rời đi. Trong test ta chờ bằng `await container.pump()` và hỏi `container.read(provider.exists)`. `container.exists` đã bị đánh dấu deprecated từ 3.5.0; thông báo của analyzer ghi `provider.exist`, nhưng tên thật của getter là `exists`.

### quote_provider_test.dart · 18-riverpod/test/quote_provider_test.dart (trích)

```dart
    test('hết người nghe thì state bị bỏ, nghe lại thì hỏi lại', () async {
      final repository = StubQuoteRepository();
      final container = ProviderContainer.test(
        overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
      );

      final first = container.listen(quoteProvider, (_, _) {});
      await container.read(quoteProvider.future);
      expect(container.read(quoteProvider.exists), isTrue);

      first.close();
      await container.pump();
      expect(container.read(quoteProvider.exists), isFalse);

      container.listen(quoteProvider, (_, _) {});
      await container.read(quoteProvider.future);
      expect(repository.quoteRequests, hasLength(2));
    });
```

Provider thường giữ state sau khi hết người nghe. `ref.keepAlive()` giữ state của provider auto-dispose. `ref.onDispose` chạy mỗi lần state bị bỏ, kể cả khi provider tính lại vì nguồn đổi, nên dùng được để huỷ request đang dở hoặc đóng stream. Rời `TripScreen` làm `tripByIdProvider('T-0001')` biến mất (test `rời màn hình chuyến...`).

Disposal gài một bẫy cho code sau `await`. Khách bấm Đặt chuyến rồi rời màn hình: `placeTripProvider` bị bỏ khi `placeTrip` còn chạy, và gán `state` lúc đó ném lỗi. `ref.mounted` cho biết provider còn sống, nên `place` có dòng `if (!ref.mounted) return;` sau `await`. Trong method của `Notifier`, `ref` luôn là của lần dựng hiện tại nên chỉ dispose mới làm nó `false`; `ref` bắt trong thân provider dạng hàm thì là của lần dựng cũ và trả `false` sau khi tính lại (test `ref của provider dạng hàm...`). Test dựng bản `_UnsafePlaceTripNotifier` bỏ dòng ấy: nó ném một `Exception` có thông điệp `after it has been disposed`, còn bản thật kết thúc êm.

## 6. Retry mặc định của Riverpod 3

Hàm `ProviderContainer.defaultRetry` quyết định mọi provider lỗi sẽ thử lại thế nào. Mã nguồn và test cho thấy các luật sau:

- Tối đa 10 lần thử lại, mỗi lần chờ `200 ms × 2^lần`, trần 6400 ms. Tổng 11 lần gọi, 38,2 giây.
- Thử lại mọi lỗi trừ `Error` (như `StateError`, `ArgumentError`) và `ProviderException`: hai loại này không bao giờ được thử lại, còn một lỗi bất kỳ khác, kể cả `throw 'boom'`, thì có. Theo quy ước của Dart, `Error` dành cho lỗi lập trình.
- Trong lúc chờ, state là `AsyncLoading` mang lỗi cũ và `retrying` bằng `true`; hết lượt mới thành `AsyncError`.

Test đo từng mốc bằng đồng hồ giả, nên 38 giây chạy trong vài mili giây:

### retry_test.dart · 18-riverpod/test/retry_test.dart (trích)

```dart
    test('Exception: 11 lần gọi trong 38,2 giây rồi mới thành AsyncError', () {
      _fake((async) {
        final calls = <Duration>[];
        final provider = failingWith(const QuoteUnavailable(), calls, async);
        final container = ProviderContainer.test();
        container.listen(provider, (_, _) {});

        async.flushMicrotasks();
        async.elapse(const Duration(seconds: 38));
        expect(container.read(provider).isLoading, isTrue);

        async.elapse(const Duration(milliseconds: 200));
        expect(calls.map((d) => d.inMilliseconds), [
          0,
          200,
          600,
          1400,
          3000,
          6200,
          12600,
          19000,
          25400,
          31800,
          38200,
        ]);
        final state = container.read(provider);
        expect(state, isA<AsyncError<Quote>>());
        expect(state.isLoading, isFalse);
        expect(state.retrying, isFalse);
      });
    });
```

Với Tới Nơi, 38 giây nhìn spinner là quá dài. Báo giá chỉ đọc, không tạo chuyến, nên thử lại một lần cho lỗi tạm thời là hợp lý; `TripNotFound` thì thử bao nhiêu lần cũng vô ích. Ta truyền hàm `retry` cho từng provider:

### providers.dart · 18-riverpod/lib/providers.dart (trích)

```dart
Duration? retryTransientOnce(int retryCount, Object error) =>
    error is QuoteUnavailable && retryCount < 1
    ? const Duration(seconds: 1)
    : null;
```

Hàm trả `null` nghĩa là không thử lại. Thứ tự ưu tiên là `retry` của provider, rồi `retry` của container (`ProviderContainer.test(retry: (_, _) => null)` hoặc tham số `retry` của `ProviderScope`), rồi mới tới mặc định; test kiểm cả ba tầng. Hàm tự viết thay hẳn luật mặc định, kể cả luật bỏ qua `Error`: một hàm trả `Duration` cho mọi lỗi sẽ thử lại cả `StateError` (test `hàm trả Duration cho mọi lỗi thì StateError cũng bị thử lại`). Bộ đếm lượt không tự đặt lại khi `invalidate`: hết lượt rồi bấm Thử lại chỉ gọi thêm đúng một lần (test `invalidate sau khi hết lượt...`).

Retry chỉ áp dụng khi **dựng** provider, tức `build` hoặc thân `FutureProvider`. Method của notifier không được thử lại: `place()` lỗi là `AsyncError` ngay và repository chỉ nhận một yêu cầu (test `place() là method, không phải build`). Với thao tác ghi như đặt chuyến, đó là điều ta muốn, vì gửi lại có thể tạo hai chuyến. Backoff có kiểm soát và phân loại lỗi mạng thuộc bài 20.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với Riverpod 3</span>
<ul>
<li><strong>Dùng <code>ref.read</code> trong <code>build</code>.</strong> Dấu hiệu: state đổi mà giao diện đứng yên, vì <code>read</code> không đăng ký theo dõi. Cách sửa: <code>ref.watch</code> trong <code>build</code>, <code>ref.read</code> chỉ trong handler.</li>
<li><strong>Gọi <code>ref.listen</code> trong <code>onPressed</code>.</strong> Dấu hiệu: <code>AssertionError</code> với thông điệp <code>ref.listen can only be used within the build method</code> khi chạy debug. Cách sửa: đặt <code>ref.listen</code> ở gốc <code>build</code>, hoặc dùng <code>ref.listenManual</code> trong <code>initState</code>.</li>
<li><strong>State thiếu <code>==</code>, hoặc bị sửa tại chỗ.</strong> Hai lỗi khác nhau. Thiếu <code>==</code>: gán đúng giá trị cũ mà listener vẫn được báo, tức build thừa; sửa bằng <code>==</code> và <code>hashCode</code>. Sửa tại chỗ (<code>state.add(x); state = state;</code>): dữ liệu đổi nhưng không ai được báo, vì cùng đối tượng thì <code>==</code> đúng; sửa bằng cách tạo bản mới qua <code>copyWith</code>.</li>
<li><strong>Dùng <code>state</code> hoặc <code>ref</code> sau <code>await</code> mà không kiểm <code>ref.mounted</code>.</strong> Dấu hiệu: lỗi chứa chữ "after it has been disposed" khi khách rời màn hình giữa chừng. Cách sửa: <code>if (!ref.mounted) return;</code> sau <code>await</code>, hoặc huỷ việc bằng <code>ref.onDispose</code>.</li>
<li><strong>Quên chính sách retry.</strong> Dấu hiệu: spinner quay gần 40 giây, server nhận 11 yêu cầu cho một lần bấm. Cách sửa: tham số <code>retry</code> của provider, trả <code>null</code> cho lỗi không đáng thử lại. Lỗi liên quan: đọc <code>.value</code> trong trạng thái lỗi, vì <code>AsyncError</code> vẫn giữ giá trị của lần trước.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/18-riverpod
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4, Dart 3.13.3, `flutter_riverpod` 3.5.0: `flutter analyze` 0 issue, **87 test** xanh. Mức: **Kiểm chứng tự động**, không cần thiết bị hay mạng. Đồng hồ giả (`fake_async`) thay thời gian thật ở các test retry.

| Test | Chứng minh điều gì |
|---|---|
| `draft_notifier_test.dart` (8) | `build` khởi đầu, miền 0 đến 100 km, ngoài miền giữ nguyên state, `==` chặn thông báo thừa, thiếu `==`, sửa tại chỗ |
| `quote_provider_test.dart` (18) | Cước 25.500, 12.005 và 453.000 VND qua provider; `isReloading`; `invalidate` sau lỗi; phản hồi sai thứ tự; auto-dispose, `keepAlive`, `onDispose` (kể cả khi tính lại); `watch` khác `read` |
| `retry_test.dart` (19) | Các mốc 0, 200, 600 ... 38.200 ms; `Error` không retry, chuỗi bị ném thì có; retry của provider, của container; hàm tự viết với `StateError`; `invalidate` sau khi hết lượt; `place()` không retry |
| `place_trip_test.dart` (8) | `AsyncNotifier` thành công và lỗi; `ref.mounted` (cả sau khi tính lại); bản thiếu kiểm tra ném lỗi; `ProviderException` |
| `trip_by_id_test.dart` (7) | Family: state riêng từng id, dispose từng id, `overrideWith`, `overrideWithValue` cho một id |
| `fake_repository_test.dart` (6) | Bản giả dùng `calculateFare`, mã chuyến tăng dần, độ trễ |
| `booking_screen_test.dart` (16) | Ba trạng thái, SnackBar đúng một lần sau retry, nút Thử lại, nút Đặt chuyến tắt khi đang đặt, ghi đè `ProviderScope` |
| `ref_usage_test.dart` (5) | `select`, `read` trong `build` đứng yên, `ref.listen` sai chỗ bị assert, thiếu `ProviderScope` |

**Chưa làm**: gọi HTTP thật (bài 19), phân loại lỗi mạng và backoff (bài 20), mutation (còn là API thử nghiệm ở `experimental`), `StreamProvider`, `ProviderObserver`, scoped provider với `dependencies:`, `freezed` và `riverpod_generator` (bài 26), chạy trên simulator hay thiết bị.

## Bài tập

1. Đổi `retryTransientOnce` thành hàm trả `Duration(seconds: 1)` khi `retryCount < 3` cho mọi lỗi, rồi mở chuyến có mã không tồn tại. Repository bị gọi mấy lần và khách chờ bao lâu mới thấy lỗi?
2. Trong `BookingScreen`, đổi `ref.watch(placeTripProvider).isLoading` thành `ref.read(placeTripProvider).isLoading`. Khách bấm Đặt chuyến hai lần liên tiếp trong lúc chờ server. Repository nhận mấy yêu cầu, và vì sao?
3. Viết override để test `TripScreen` cho mã `T-9` luôn hiện chuyến đang ở trạng thái `searching`, không cần repository.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> 4 lần gọi, tại các mốc 0, 1, 2 và 3 giây, nên khách chờ 3 giây rồi mới thấy <code>Không tìm thấy chuyến T-9</code>. Hàm mới bỏ điều kiện <code>error is QuoteUnavailable</code>, nên <code>TripNotFound</code> cũng bị thử (luật mặc định cũng thử nó, nhưng tới 10 lần); test <code>bỏ điều kiện QuoteUnavailable...</code> đo các mốc này.</p>
<p><strong>2.</strong> Hai yêu cầu. <code>read</code> trong <code>build</code> đóng băng giá trị <code>isLoading</code> ở lần dựng đó, như test <code>watch cập nhật giao diện, read trong build thì không</code>. Khi <code>place()</code> đặt state thành <code>AsyncLoading</code>, màn hình không build lại, nút vẫn bấm được và lần bấm thứ hai gọi <code>placeTrip</code> lần nữa. Với <code>watch</code>, nút tắt ngay sau lần bấm đầu (test <code>đang đặt thì nút tắt</code>) và chỉ có một yêu cầu.</p>
<p><strong>3.</strong> <code>tripByIdProvider('T-9').overrideWithValue(AsyncData(tripFor('T-9').apply(const SearchStarted())))</code> trong <code>ProviderScope(overrides: [...])</code>. Trong test <code>overrideWithValue cho đúng id T-9</code>, màn hình hiện chip "Đang tìm tài xế" và cước 25.500 VND mà không cần <code>quoteRepositoryProvider</code>.</p>
</details>

Bài trước: [Quản lý state: InheritedWidget, ChangeNotifier và ranh giới rebuild](/articles/flutter-state-inheritedwidget-changenotifier-bao-gia). Bài tiếp: [HTTP, dio và JSON: gọi API báo giá của trip-service](/articles/flutter-http-dio-json-dto-goi-api-bao-gia).

## Tài liệu gốc

- [Riverpod: Getting started](https://riverpod.dev/docs/introduction/getting_started) và [What's new in Riverpod 3.0](https://riverpod.dev/docs/whats_new) (riverpod.dev).
- [Providers](https://riverpod.dev/docs/concepts2/providers), [Refs](https://riverpod.dev/docs/concepts2/refs) và [Auto dispose](https://riverpod.dev/docs/concepts2/auto_dispose) (riverpod.dev).
- [Automatic retry](https://riverpod.dev/docs/concepts2/retry) và [Testing](https://riverpod.dev/docs/how_to/testing) (riverpod.dev).
- [flutter_riverpod](https://pub.dev/packages/flutter_riverpod), [riverpod](https://pub.dev/packages/riverpod) và [changelog của riverpod](https://pub.dev/packages/riverpod/changelog) (pub.dev).
- [AsyncValue](https://pub.dev/documentation/riverpod/3.5.0/riverpod/AsyncValue-class.html) và [ProviderContainer.defaultRetry](https://pub.dev/documentation/riverpod/3.5.0/riverpod/ProviderContainer/defaultRetry.html) (pub.dev).
- [State management](https://docs.flutter.dev/data-and-backend/state-mgmt/options) (docs.flutter.dev).
