---
slug: flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia
title: "Widget test: pumpWidget, finder, matcher và màn hình báo giá"
summary: "Bài 28: widget test cho màn hình báo giá Tới Nơi: pumpWidget trong ProviderScope override, pump, pump(Duration), pumpAndSettle (kèm ca hết hạn), finder và matcher, enterText, tap, warnIfMissed, ensureVisible, runAsync, font FlutterTest, kích thước màn hình và điều hướng go_router."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, testing, widget-test, riverpod, go-router]
language: vi
---

Khách mở màn hình báo giá của Tới Nơi, gõ "2,001" vào ô quãng đường, bấm "Xem báo giá" và thấy 12.005 VND. Họ cuộn xuống, bấm "Đặt chuyến", thấy vòng quay chờ, rồi sang màn hình chuyến T-0001. Bài 27 đã kiểm quy tắc cước và use case mà không cần widget nào. Nhưng lỗi hay nằm ở chỗ khác: dấu phẩy thập phân bị từ chối, nút Đặt chuyến nằm dưới đáy màn hình, màn hình vẫn hiện lỗi cũ khi khách bấm Thử lại.

**Widget test** dựng đúng cây widget đó trong bộ nhớ, không cần simulator. Đây là bài 28 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 27 về unit test với mocktail và fake](/articles/flutter-unit-test-mocktail-fake-quy-tac-cuoc-use-case) và [bài 18 về Riverpod](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen). Ví dụ dùng `Money`, `calculateFare` của `toi_noi_core` (hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi)); bảng giá là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Màn hình đang hiện vòng quay chờ báo giá, và báo giá không bao giờ về. Test gọi <code>await tester.pumpAndSettle();</code>. Chuyện gì xảy ra?</p>
<details><summary>Xem đáp án</summary><p><strong>Không bao giờ yên, rồi ném lỗi.</strong> <code>pumpAndSettle</code> lặp <code>pump(100 ms)</code> cho tới khi không còn khung hình nào được xin. <code>CircularProgressIndicator</code> xin khung mới liên tục, nên vòng lặp chạy tới hết 10 phút đồng hồ giả (khoảng 6.000 lần) rồi ném <code>FlutterError: pumpAndSettle timed out</code>. Test <code>pumpAndSettle hết hạn khi vòng quay chạy mãi</code> chứng minh; mục 2 nói cách kiểm trạng thái chờ đúng.</p></details>
</div>

## 1. Dựng màn hình trong test: ProviderScope, override và đồng hồ giả

`testWidgets` đưa cho ta một `WidgetTester`. `pumpWidget` gắn cây widget vào một view ảo và dựng khung đầu tiên. Thân test chạy trong `FakeAsync` của binding: `Future.delayed`, `Timer` và hoạt ảnh dùng **đồng hồ giả** chỉ chạy khi test ra lệnh (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Ba bên trong một widget test. Thân test gọi pumpWidget, tap và expect. Binding kiểm soát đồng hồ giả, khung hình chỉ dựng khi pump, view 800x600 và font FlutterTest. Cây widget gồm ProviderScope có overrides và MaterialApp với QuoteScreen. QuoteScreen gọi fetchQuote trên ControlledQuoteRepository. Thân test tự gọi complete hoặc fail để quyết định lúc nào repository giả trả lời.">
<defs><marker id="fl28-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="250" height="200" rx="12" class="f-phys-soft s-phys"/><text x="36" y="56" class="t-b">Thân test (async)</text>
<text x="36" y="84" class="t-mono">pumpWidget(...)</text><text x="36" y="108" class="t-mono">tester.tap(finder)</text><text x="36" y="132" class="t-mono">expect(finder, matcher)</text>
<text x="36" y="168" class="t-small">chạy trong FakeAsync</text><text x="36" y="188" class="t-small">không có thời gian thật</text>
<rect x="315" y="30" width="250" height="200" rx="12" class="f-pk-soft s-pk"/><text x="331" y="56" class="t-b">Binding kiểm soát</text>
<text x="331" y="84" class="t-small">đồng hồ giả: chỉ trôi khi pump</text><text x="331" y="108" class="t-small">khung hình: chỉ dựng khi pump</text><text x="331" y="132" class="t-small">view 800x600 logical, dpr 3</text><text x="331" y="156" class="t-small">chữ: font FlutterTest</text>
<rect x="610" y="30" width="250" height="130" rx="12" class="f-surface s-line"/><text x="626" y="56" class="t-b">Cây widget</text>
<text x="626" y="84" class="t-mono">ProviderScope(overrides)</text><text x="626" y="108" class="t-mono">MaterialApp + QuoteScreen</text><text x="626" y="138" class="t-small">đọc quoteRepositoryProvider</text>
<rect x="610" y="200" width="250" height="70" rx="12" class="f-phys-soft s-phys"/><text x="626" y="226" class="t-mono">ControlledQuoteRepository</text><text x="626" y="250" class="t-small">test quyết định lúc nào trả lời</text>
<path d="M272 100 L311 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl28-1-a)"/>
<path d="M567 100 L606 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl28-1-a)"/>
<path d="M735 162 L735 196" class="f-none s-pk" stroke-width="2" marker-end="url(#fl28-1-a)"/><text x="745" y="184" class="t-small">fetchQuote</text>
<path d="M145 232 L145 310 L735 310 L735 274" class="f-none s-pk" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl28-1-a)"/>
<text x="200" y="302" class="t-small">complete() hoặc fail(): test trả lời báo giá</text>
</svg>
</div>
<figcaption>Hình 1. Thân test điều khiển binding và repository giả; binding dựng cây widget và gửi cú chạm vào đó. Mũi tên đứt là lời gọi trực tiếp của test tới repository giả.</figcaption>
</figure>

Màn hình báo giá lấy dữ liệu từ `quoteProvider`, và provider này đọc `quoteRepositoryProvider`. Test **ghi đè** repository bằng một bản giả điều khiển bằng `Completer`: yêu cầu nào cũng treo cho tới khi test gọi `complete`. Không có `Timer` nào nên trạng thái loading đứng yên, không phụ thuộc thời gian thật.

### support.dart · 28-widget-test/test/support.dart (trích)

```dart
final class ControlledQuoteRepository implements QuoteRepository {
  final quoteRequests = <({ServiceKind serviceKind, double distanceKm})>[];
  final _quotes = <Completer<Quote>>[];

  int get fetchCount => quoteRequests.length;

  @override
  Future<Quote> fetchQuote({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) {
    quoteRequests.add((serviceKind: serviceKind, distanceKm: distanceKm));
    final completer = Completer<Quote>();
    _quotes.add(completer);
    return completer.future;
  }

  /// Trả lời yêu cầu mới nhất bằng cước tính từ calculateFare.
  void completeQuote({int? index}) {
    final i = index ?? _quotes.length - 1;
    final request = quoteRequests[i];
    _quotes[i].complete(Quote(
      serviceKind: request.serviceKind,
      distanceKm: request.distanceKm,
      fare: calculateFare(request.distanceKm),
    ));
  }
  // failQuote, placeTrip, completePlace: xem file trong lab
}

Future<void> pumpQuoteScreen(
  WidgetTester tester, {
  List<Override> overrides = const [],
}) {
  return tester.pumpWidget(
    ProviderScope(
      overrides: overrides,
      child: MaterialApp(theme: ToiNoiTheme.light(), home: const QuoteScreen()),
    ),
  );
}
```

Có hai tầng để ghi đè. Ghi đè `quoteRepositoryProvider` để chạy qua provider thật và đếm được số lần gọi (`fetchCount`). Ghi đè thẳng `quoteProvider.overrideWithValue(const AsyncLoading())` hay `AsyncError(...)` khi chỉ cần xem màn hình vẽ một state. Lab dùng cả hai (`states_test.dart`). Provider của lab đặt `retry: (_, _) => null`, vì Riverpod 3 mặc định thử lại lỗi `Exception` nhiều lần; bỏ dòng đó thì hai test trong `states_test.dart` đỏ vì lỗi không hiện ngay.

## 2. pump, pump(Duration) và pumpAndSettle: thời gian trong test

Khung hình trong test **không tự chạy**. Ba lệnh khác nhau ở chỗ chúng cho thời gian trôi hay không (Hình 2).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 2. Dòng thời gian đồng hồ giả từ 0 đến 400 mili giây với repository có độ trễ 400 ms. Timer hết hạn ở 400 ms và báo giá về. pump không tham số dựng khung nhưng đồng hồ đứng ở 0 ms nên vẫn loading. pump 399 ms tua tới 399 ms, vẫn loading. pump 1 ms tua tới 400 ms và hiện dữ liệu. pumpAndSettle tua từng bước 100 ms và dừng khi không còn khung được xin. Khi vòng quay chạy mãi, pumpAndSettle lặp khoảng 6.000 bước rồi ném lỗi hết hạn.">
<defs><marker id="fl28-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl28-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="20" y="26" class="t-b">Đồng hồ giả của test, FakeQuoteRepository(latency: 400 ms)</text>
<path d="M560 56 L560 114" class="f-none s-warn" stroke-width="2" stroke-dasharray="5 4"/><text x="330" y="48" class="t-small t-warn">Timer 400 ms hết hạn: báo giá về</text>
<path d="M60 120 L840 120" class="f-none s-muted" stroke-width="2"/>
<path d="M80 113 L80 127 M200 113 L200 127 M320 113 L320 127 M440 113 L440 127 M560 113 L560 127" class="f-none s-muted" stroke-width="2"/>
<text x="76" y="144" class="t-mono">0</text><text x="188" y="144" class="t-mono">100</text><text x="308" y="144" class="t-mono">200</text><text x="428" y="144" class="t-mono">300</text><text x="536" y="144" class="t-mono">400 ms</text>
<circle cx="80" cy="176" r="5" class="f-pk"/><text x="96" y="180" class="t-mono">pump()</text><text x="160" y="180" class="t-small">dựng khung, đồng hồ đứng ở 0 ms: vẫn loading</text>
<text x="90" y="206" class="t-mono">pump(399 ms)</text><text x="190" y="206" class="t-small">vẫn loading</text>
<path d="M80 214 L556 214" class="f-none s-pk" stroke-width="2" marker-end="url(#fl28-2-a)"/>
<circle cx="560" cy="246" r="5" class="f-phys"/><text x="290" y="250" class="t-mono">pump(1 ms)</text><text x="370" y="250" class="t-small">chạm 400 ms: hiện dữ liệu</text>
<text x="90" y="284" class="t-mono">pumpAndSettle</text><text x="190" y="284" class="t-small">tua từng bước 100 ms, dừng khi không còn khung</text>
<path d="M80 292 L196 292" class="f-none s-pk" stroke-width="2" marker-end="url(#fl28-2-a)"/><path d="M204 292 L316 292" class="f-none s-pk" stroke-width="2" marker-end="url(#fl28-2-a)"/><path d="M324 292 L436 292" class="f-none s-pk" stroke-width="2" marker-end="url(#fl28-2-a)"/><path d="M444 292 L556 292" class="f-none s-pk" stroke-width="2" marker-end="url(#fl28-2-a)"/>
<text x="574" y="296" class="t-small">yên: trả về</text>
<text x="90" y="324" class="t-small t-warn">vòng quay chạy mãi: khoảng 6.000 bước rồi ném 'pumpAndSettle timed out'</text>
<path d="M80 334 L840 334" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl28-2-w)"/>
</svg>
</div>
<figcaption>Hình 2. Một dòng thời gian, bốn lệnh. Chỉ <code>pump(Duration)</code> và <code>pumpAndSettle</code> làm đồng hồ giả tiến; <code>pump()</code> chỉ dựng khung ở thời điểm hiện tại.</figcaption>
</figure>

Theo mã nguồn `flutter_test`, `pump(duration)` tua đồng hồ giả `duration` (timer tới hạn chạy luôn), rồi nếu có khung đang chờ thì dựng khung. `pump()` không tham số bỏ bước tua. `pumpAndSettle` lặp `pump(100 ms)` tới khi `hasScheduledFrame` là `false`, và ném `FlutterError('pumpAndSettle timed out')` nếu vượt `timeout` (mặc định 10 phút đồng hồ giả).

### pump_test.dart · 28-widget-test/test/pump_test.dart (trích)

```dart
testWidgets('pump(Duration) tua đồng hồ giả: 399 ms chưa, 400 ms rồi', (
  tester,
) async {
  await _pumpWithLatency(tester); // FakeQuoteRepository(latency: 400 ms)

  await tester.pump(const Duration(milliseconds: 399));
  expect(find.byKey(_loading), findsOneWidget);

  await tester.pump(const Duration(milliseconds: 1));
  expect(find.byKey(_loading), findsNothing);
  expect(find.byKey(const Key('quote-fare')), findsOneWidget);
});

testWidgets('pumpAndSettle hết hạn khi vòng quay chạy mãi', (tester) async {
  final repository = ControlledQuoteRepository(); // không bao giờ trả lời
  await pumpQuoteScreen(
    tester,
    overrides: [quoteRepositoryProvider.overrideWithValue(repository)],
  );

  Object? error;
  try {
    await tester.pumpAndSettle(
      const Duration(milliseconds: 100),
      EnginePhase.sendSemanticsUpdate,
      const Duration(seconds: 2), // rút ngắn để test chạy nhanh
    );
  } on FlutterError catch (e) {
    error = e;
  }

  expect(error, isA<FlutterError>().having(
    (e) => e.message, 'message', 'pumpAndSettle timed out'));
});
```

Với repository có độ trễ 400 ms, `pumpAndSettle` chạy tốt vì vòng quay biến mất khi dữ liệu về. Nó chỉ treo khi hoạt ảnh không bao giờ dừng. Muốn kiểm trạng thái loading thì dùng `pump()` hoặc `pump(Duration)`, hoặc ghi đè `quoteProvider` bằng `AsyncLoading`. Khi không có hoạt ảnh nào, `pumpAndSettle` trả về sau đúng một khung.

Hai chi tiết nữa đều có test. Một, test kết thúc khi còn `Timer` chưa tới hạn bị báo `A Timer is still pending`; hai test đầu của `pump_test.dart` vì thế kết thúc bằng `pump(Duration)` qua hạn 400 ms. Hai, `pump()` chỉ dựng khung nếu lúc bắt đầu đã có khung được xin. Màn hình đang yên mà ta `complete()` một `Future`, callback chạy ở cuối `pump()` và gọi `setState` quá muộn cho khung đó, nên cần pump lần hai; `pump(Duration.zero)` chạy microtask trước nên đủ ngay. Khi vòng quay còn chạy thì khung luôn sẵn và một `pump()` là đủ.

## 3. Finder và matcher: tìm gì, kỳ vọng gì

**Finder** mô tả cách tìm widget trong cây; **matcher** (`findsOneWidget`, `findsNothing`, `findsNWidgets(n)`, `findsWidgets`) nói ta kỳ vọng tìm được bao nhiêu. Finder chỉ thấy cây, không thấy màn hình: widget nằm ngoài khung nhìn của vùng cuộn vẫn được tìm thấy.

| Finder | Tìm | Lưu ý |
|---|---|---|
| `find.byKey(Key('quote-fare'))` | widget có `key` bằng | bền với đổi nhãn; key phải do ta đặt |
| `find.text('25.500 VND')` | `Text`, `Text.rich`, `EditableText` có chuỗi **bằng đúng** | `textContaining` cho khớp một phần |
| `find.byType(ChoiceChip)` | widget có đúng kiểu | trả cả những thứ ta không nhìn thấy |
| `find.widgetWithText(FilledButton, 'Đặt chuyến')` | tổ tiên kiểu cho trước của một `Text` | dùng cho nút theo nhãn |
| `find.descendant(of:, matching:)` | con cháu của một widget | thu hẹp vùng tìm |

Màn hình báo giá hiện cước hai lần: ở thẻ cước và ở dòng "Tổng". Vì vậy `find.text('25.500 VND')` cho hai kết quả: ta đếm bằng `findsNWidgets(2)` hoặc chỉ vào đúng khối bằng `descendant`.

### finders_test.dart · 28-widget-test/test/finders_test.dart (trích)

```dart
expect(find.text('25.500 VND'), findsNWidgets(2));

expect(
  find.descendant(
    of: find.byKey(const Key('quote-card')),
    matching: find.text('25.500 VND'),
  ),
  findsOneWidget,
);

final fare = tester.widget<MoneyText>(find.byKey(const Key('quote-fare')));
expect(fare.money, calculateFare(5));
```

`tester.widget<T>` lấy chính widget ra để kiểm thuộc tính: ta so `Money` thật, nên đổi cách định dạng tiền không làm test gãy. Còn `tester.tap(find.text('25.500 VND'))` sẽ ném `ambiguously found multiple matching widgets`: hàm cần một mục tiêu đòi finder khớp đúng một widget.

Chọn text hay key? Text là thứ khách đọc nên là hợp đồng với người dùng, nhưng khi đội sản phẩm đổi nhãn thành "Đặt xe ngay" mọi test tìm theo text đồng loạt đỏ, còn key vẫn đúng. Quy ước của lab: **key để tìm và tương tác** với phần tử cấu trúc (nút, ô nhập, khối), **text để kiểm nội dung** khách thấy (cước, thông báo lỗi).

### finders_test.dart · 28-widget-test/test/finders_test.dart (trích)

```dart
await tester.pumpWidget(screen('Đặt chuyến'));
expect(find.text('Đặt chuyến'), findsOneWidget);
expect(find.byKey(const Key('submit')), findsOneWidget);

await tester.pumpWidget(screen('Đặt xe ngay')); // đội sản phẩm đổi nhãn
expect(find.text('Đặt chuyến'), findsNothing);
expect(find.byKey(const Key('submit')), findsOneWidget);
```

## 4. enterText, tap, warnIfMissed và ensureVisible

`tester.enterText(finder, text)` cho ô nhập tiêu điểm rồi **thay toàn bộ** nội dung bằng `text`, như bàn phím ảo; nó không nối thêm, không gửi form, và chữ mới chưa lên khung hình nên cần `pump`. Nút "Xem báo giá" nhận số như "2,001" (dấu phẩy kiểu Việt) và từ chối chuỗi ngoài miền 0 đến 100 km. Ta liệt kê ca biên trong một bảng và sinh test từ đó:

### input_test.dart · 28-widget-test/test/input_test.dart (trích)

```dart
// 12.000 + (mét vượt x 4.500 + 500) ~/ 1.000. 2,001 km vượt 1 m: 12.005;
// 100 km vượt 98.000 m: 453.000.
const cases = [
  (input: '0', fare: '12.000 VND'),
  (input: '7', fare: '34.500 VND'),
  (input: '2,001', fare: '12.005 VND'),
  (input: '2.001', fare: '12.005 VND'),
  (input: ' 100 ', fare: '453.000 VND'),
];

for (final c in cases) {
  testWidgets('nhập "${c.input}" km: cước ${c.fare}', (tester) async {
    final repository = await _pumpReady(tester);

    await tester.enterText(find.byKey(_field), c.input);
    await tester.tap(find.byKey(_apply));
    await tester.pump();
    expect(find.byKey(const Key('quote-loading')), findsOneWidget);
    repository.completeQuote();
    await tester.pump();

    expect(
      find.descendant(
        of: find.byKey(const Key('quote-card')),
        matching: find.text(c.fare),
      ),
      findsOneWidget,
    );
    expect(repository.fetchCount, 2);
  });
}
```

Các chuỗi `''`, `abc`, `-1`, `100,5`, `100.001`, `NaN`, `Infinity` đều hiện "Nhập số từ 0 đến 100 km", không gọi repository và giữ bản nháp 5 km. Nhập `5.0` cho đúng bản nháp hiện tại nên Riverpod không tính lại (`fetchCount` vẫn 1), và `tester.testTextInput.receiveAction(TextInputAction.done)` gửi như bấm nút.

`tester.tap(finder)` gửi cú chạm vào **điểm giữa** widget, qua hit test như ngón tay thật. Nếu điểm đó không rơi vào widget (nằm ngoài màn hình, bị lớp khác che, hay widget bỏ qua chạm), cú chạm vẫn được gửi, nhưng rơi vào chỗ khác, và `warnIfMissed` (mặc định `true`) in cảnh báo ra console. Đây là nguồn của test "tap mà không có gì xảy ra" (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 3. Hai cách một cú chạm trượt nút Đặt chuyến. Bên trái: màn hình 360x640 có phần nội dung cuộn, nút Đặt chuyến nằm bên dưới mép dưới nên điểm chạm ngoài màn hình và không widget nào nhận; ensureVisible cuộn nút vào màn hình trước khi chạm. Bên phải: nút nằm trong màn hình nhưng lớp ModalBarrier cùng vòng quay đang đặt chuyến phủ lên, nên hit test trúng ModalBarrier trước và nút không nhận chạm.">
<text x="20" y="26" class="t-b">1. Nút nằm ngoài màn hình</text>
<rect x="120" y="40" width="200" height="220" rx="14" class="f-surface s-line"/>
<rect x="132" y="54" width="176" height="22" rx="6" class="f-pk-soft s-pk"/><rect x="132" y="86" width="176" height="26" rx="6" class="f-surface s-line"/><rect x="132" y="124" width="176" height="48" rx="8" class="f-pk-soft s-pk"/><rect x="132" y="182" width="176" height="20" rx="6" class="f-surface s-line"/><rect x="132" y="212" width="176" height="20" rx="6" class="f-surface s-line"/>
<text x="136" y="250" class="t-small">nội dung cuộn</text><text x="326" y="258" class="t-small">mép dưới 640</text>
<rect x="140" y="272" width="160" height="32" rx="8" class="f-warn-soft s-warn" stroke-dasharray="6 5"/><text x="172" y="293" class="t-b">Đặt chuyến</text>
<circle cx="220" cy="288" r="5" class="f-warn"/>
<text x="20" y="334" class="t-small">Điểm giữa nút nằm dưới mép 640: không widget nào nhận chạm.</text>
<text x="20" y="354" class="t-small">ensureVisible cuộn nút vào màn hình trước khi tap.</text>
<text x="460" y="26" class="t-b">2. Lớp phủ chặn chạm</text>
<rect x="560" y="40" width="200" height="220" rx="14" class="f-surface s-line"/>
<rect x="572" y="100" width="176" height="48" rx="8" class="f-pk-soft s-pk"/><rect x="572" y="160" width="176" height="20" rx="6" class="f-surface s-line"/>
<rect x="580" y="206" width="160" height="32" rx="8" class="f-pk-soft s-pk"/><text x="612" y="227" class="t-b">Đặt chuyến</text>
<rect x="564" y="44" width="192" height="212" rx="12" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5"/><text x="576" y="68" class="t-b t-warn">ModalBarrier</text><text x="576" y="86" class="t-small t-warn">và vòng quay đang đặt</text>
<circle cx="660" cy="222" r="5" class="f-warn"/>
<text x="460" y="334" class="t-small">Hit test đi từ lớp trên cùng: ModalBarrier trúng trước.</text>
<text x="460" y="354" class="t-small">Cố ý chạm lần hai: warnIfMissed: false.</text>
</svg>
</div>
<figcaption>Hình 3. Chấm cam là điểm chạm của <code>tester.tap</code>. Bên trái điểm đó nằm ngoài màn hình; bên phải nó nằm trên nút nhưng bị lớp phủ nhận chạm thay.</figcaption>
</figure>

Test dựng màn hình 360x640 và kiểm cả hai nhánh: `tap(..., warnIfMissed: false)` không gọi `placeTrip`; `ensureVisible` rồi `tap` gọi đúng một lần. Muốn cảnh báo thành lỗi thì đặt `WidgetController.hitTestWarningShouldBeFatal = true` (trả về `false` trong `addTearDown`).

### tap_test.dart · 28-widget-test/test/tap_test.dart (trích)

```dart
await tester.ensureVisible(find.byKey(_place));
await tester.pump();
await tester.tap(find.byKey(_place));
await tester.pump();
expect(find.byKey(const Key('placing')), findsOneWidget);

// Lớp phủ "đang đặt" chặn cú chạm: tap trúng ModalBarrier. Cố ý nên tắt cảnh báo.
await tester.tap(find.byKey(_place), warnIfMissed: false);
await tester.pump();

expect(repository.placeCount, 1); // không đặt trùng chuyến
```

`ensureVisible` chỉ chạy khi widget đã có trong cây. Với `ListView.builder`, phần tử xa chưa được dựng thì phải dùng `scrollUntilVisible`; màn hình báo giá dùng `SingleChildScrollView` nên mọi phần tử đều có mặt.

## 5. Kích thước màn hình, font FlutterTest và runAsync

Mặc định view của test là **800x600 logical**: mã nguồn đặt `_kDefaultTestViewportSize = Size(800, 600)`, và test kiểm `physicalSize` là 2400x1800 với `devicePixelRatio` 3. Muốn thử điện thoại thì đặt `tester.view.physicalSize` và `devicePixelRatio`, rồi **trả về mặc định**:

### support.dart · 28-widget-test/test/support.dart (trích)

```dart
void usePhoneScreen(WidgetTester tester) {
  tester.view.physicalSize = const Size(1080, 1920); // 360x640 logical
  tester.view.devicePixelRatio = 3;
  addTearDown(tester.view.reset);
}
```

Framework không tự trả lại kích thước: test cuối của `view_test.dart` kiểm trong chính `addTearDown` rằng view vẫn là 1080x1920, rồi mới gọi `reset()` để về 2400x1800. Thiếu bước đó, kích thước rò sang test sau trong cùng file.

Chữ trong `flutter test` mặc định dùng font **FlutterTest** (tài liệu `Flutter-Test-Fonts` của SDK; tool chạy engine với `--use-test-fonts`): hầu hết ký tự là hình vuông có cạnh bằng cỡ chữ, ascent 0,75 em và descent 0,25 em. Test đo `Text('ABCDE')` cỡ 20 ra đúng 100x20 với baseline ở 15 px (font Ahem, có sẵn để chọn thủ công, cho 16), và `Text('Đặt xe')` cỡ 10 rộng 60. Hệ quả: bề rộng chữ trong test chỉ có giá trị tương đối. Font này rộng hơn font thật, nên `Row` thiếu `Expanded` tràn trên màn hình 320 px sớm hơn ngoài đời (test `Row không Expanded tràn trên màn hình hẹp` lấy lỗi bằng `tester.takeException()`). Cũng vì thế ngưỡng chữ của test `màn hình báo giá không tràn` là hệ số 1,0 và 1,5; ở 2,0, hàng chi tiết tràn 1,5 px (màn 360) tới 42 px (màn 320), điều font thật không gây ra, nên ta không coi đó là lỗi màn hình. Kiểm giao diện với font thật là việc của golden test ở bài sau.

`tester.runAsync` dành cho việc bất đồng bộ **thật**. Tài liệu API ghi: nó dùng cho phương thức tạo isolate hoặc luồng hệ điều hành mà `pump` không thể thực thi đồng bộ. Ví dụ: đọc phản hồi mẫu `test/fixtures/quote_ride_5km.json`.

### run_async_test.dart · 28-widget-test/test/run_async_test.dart (trích)

```dart
// I/O thật phải bắt đầu VÀ kết thúc bên trong runAsync.
final json = await tester.runAsync(() => File(_fixturePath).readAsString());
final quote = _parse(json!);
expect(quote.fare, calculateFare(quote.distanceKm)); // file mẫu khớp domain

await pumpQuoteScreen(
  tester,
  overrides: [quoteRepositoryProvider.overrideWithValue(_FixedRepository(quote))],
);
await tester.pumpAndSettle();
```

Đảo lại: repository đọc file **ngay khi widget hỏi** thì chuỗi I/O bắt đầu trong vùng `FakeAsync` và **không tự xong**. Mỗi bước của chuỗi cần microtask của `FakeAsync` chạy, mà việc đó chỉ xảy ra khi `pump` hoặc khi một `runAsync` kết thúc (theo `binding.dart`). Một lượt `runAsync` 200 ms rồi `pump` vẫn để màn hình ở loading (test `I/O thật bắt đầu trong cây widget: một lượt runAsync chưa đủ`); lặp `runAsync` rồi `pump` thì cuối cùng có dữ liệu sau vài lượt (test `lặp runAsync rồi pump thì cuối cùng cũng xong, không tất định`). Số lượt phụ thuộc số bước của chuỗi chứ không phải thời gian, nên đừng dựa vào nó. Quy tắc là đọc dữ liệu thật trong `runAsync` ở thân test, rồi đưa dữ liệu đã đọc cho widget qua một bản giả. Nếu `runAsync` hỏng, nó trả `null` và lỗi nằm ở `tester.takeException()`.

## 6. Test điều hướng với go_router

Màn hình báo giá gọi `context.push('/trip/${trip.id}')` sau khi đặt chuyến thành công, nên test cần `GoRouter` thật. `pumpApp` tạo router, đưa nó cho `ToiNoiApp` (`MaterialApp.router`) và **trả router ra** để test đọc vị trí; `addTearDown(router.dispose)` dọn khi hết test.

### navigation_test.dart · 28-widget-test/test/navigation_test.dart (trích)

```dart
final (:repository, :router) = await _startPlacing(tester);
expect(router.state.uri.path, '/');

repository.completePlace(quote5km);
await tester.pumpAndSettle();

expect(find.byType(TripScreen), findsOneWidget);
expect(
  find.descendant(of: find.byType(AppBar), matching: find.text('Chuyến T-0001')),
  findsOneWidget,
);
expect(find.byType(TripStatusChip), findsOneWidget);
expect(router.state.uri.path, '/trip/T-0001');

// push: màn hình báo giá còn nằm dưới, bị Offstage.
expect(find.byType(QuoteScreen), findsNothing);
expect(find.byType(QuoteScreen, skipOffstage: false), findsOneWidget);
```

Sau `push`, màn hình bên dưới vẫn trong cây nhưng bị `Offstage`, và finder mặc định bỏ qua nó (`skipOffstage: true`); thêm `skipOffstage: false` để thấy. Vị trí của trang trên cùng đọc ở `router.state.uri`, giống chi tiết ở [bài 23](/articles/flutter-go-router-deep-link-shellroute-guard). `await tester.pageBack()` quay về màn hình báo giá với ô quãng đường vẫn là "12" vì `State` của nó chưa bị huỷ.

Ba kịch bản còn lại: đặt chuyến lỗi thì ở lại `/`, hiện `SnackBar` và bỏ lớp phủ; mở thẳng `initialLocation: '/trip/T-0009'` (như deep link) thì không có màn hình báo giá nào trong cây, kể cả `Offstage`; `router.go` thay cả ngăn xếp còn `router.push` chồng lên.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi viết widget test</span>

<p><strong>1. <code>pumpAndSettle</code> treo rồi báo <code>timed out</code>.</strong> Dấu hiệu: test chạy lâu rồi ném <code>pumpAndSettle timed out</code>. Nguyên nhân: có hoạt ảnh không dừng, thường là <code>CircularProgressIndicator</code>. Cách sửa: <code>pump(Duration)</code> cho trạng thái chờ; chỉ dùng <code>pumpAndSettle</code> khi dữ liệu sẽ về. Test <code>pumpAndSettle hết hạn khi vòng quay chạy mãi</code> tái hiện.</p>

<p><strong>2. <code>A Timer is still pending</code> ở cuối test.</strong> Dấu hiệu: test đã đúng nhưng vẫn đỏ. Nguyên nhân: repository giả dùng <code>Future.delayed</code> và test kết thúc trước hạn. Cách sửa: <code>pump(Duration)</code> qua hạn, hoặc dùng <code>Completer</code> như <code>ControlledQuoteRepository</code>.</p>

<p><strong>3. <code>tap</code> không làm gì.</strong> Dấu hiệu: <code>onPressed</code> không chạy, console có <code>would not hit test on the specified widget</code>. Nguyên nhân: nút ngoài màn hình hoặc bị lớp phủ che (Hình 3). Cách sửa: <code>ensureVisible</code>, đổi kích thước view, hoặc <code>warnIfMissed: false</code> khi chủ ý; bật <code>hitTestWarningShouldBeFatal</code> để cảnh báo không trôi qua.</p>

<p><strong>4. Finder khớp nhiều widget.</strong> Dấu hiệu: <code>ambiguously found multiple matching widgets</code> hoặc <code>findsOneWidget</code> thất bại với 2 kết quả. Nguyên nhân: cùng chuỗi hiện ở nhiều chỗ, như 25.500 VND. Cách sửa: <code>find.byKey</code> hoặc <code>find.descendant</code>.</p>

<p><strong>5. Màn hình vẫn hiện lỗi cũ lúc Thử lại.</strong> Dấu hiệu: bấm Thử lại, repository nhận yêu cầu thứ hai nhưng giao diện vẫn là thông báo lỗi. Nguyên nhân: sau <code>ref.invalidate</code> state vẫn là <code>AsyncError</code> mang cờ <code>isLoading</code> bằng <code>true</code>; <code>switch</code> chỉ xét <code>AsyncLoading()</code> sẽ rơi vào nhánh lỗi. Cách sửa: xét <code>isLoading</code> trước, như <code>quote_screen.dart</code>. Test <code>loading, rồi lỗi, rồi Thử lại, rồi dữ liệu</code> bắt lỗi này.</p>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/28-widget-test
flutter pub get
flutter analyze
flutter test
```

Kỳ vọng: `No issues found!` và `All tests passed!` với **70 test**. Đã kiểm với Flutter 3.47.4, Dart 3.13.3, `flutter_riverpod` 3.5.0 và `go_router` 18.1.3. Mức kiểm chứng: **Kiểm chứng tự động**. `flutter test` chạy với thư mục làm việc là gốc package, nên test đọc được file mẫu bằng đường dẫn tương đối.

Các nhóm test: `pump_test.dart` (11) cho thời gian giả; `finders_test.dart` (8) cho finder và matcher; `input_test.dart` (17) cho `enterText` và ca biên cước (12.005 và 453.000 VND); `states_test.dart` (7) cho ba trạng thái; `tap_test.dart` (7) cho cú chạm trượt; `view_test.dart` (10) cho kích thước và font; `navigation_test.dart` (6) cho go_router; `run_async_test.dart` (4) cho I/O thật.

Ta thử phá để chắc test bắt được lỗi: bỏ nhánh `isLoading` làm 1 test đỏ, bỏ đổi dấu phẩy thành dấu chấm 1 test, bỏ lớp phủ đang đặt 3 test, bỏ `context.push` 3 test, bỏ `retry: (_, _) => null` 2 test, bỏ `Expanded` của nhãn 11 test.

**Chưa kiểm**: giao diện với font thật của iOS và Android, golden và Semantics test (bài 29), test Riverpod bằng `ProviderContainer` và `fakeAsync` (bài 30), integration test trên simulator (bài 31), nút Back hệ thống và deep link do hệ điều hành gửi. Lab chưa nối `toi_noi_api` thật.

## Bài tập

1. Viết test: chọn chip "Giao bưu kiện", đổi quãng đường 1 km, trả lời báo giá. Dòng "Phần vượt" hiện gì, và nhãn dịch vụ trong thẻ cước là gì?
2. Sau khi bấm "Đặt chuyến" (repository chưa trả lời), `await tester.pumpAndSettle()` hết hạn còn `await tester.pump()` thì không. Vì sao, và nên viết lại thế nào để kiểm lớp phủ?
3. Test bấm "Đặt chuyến" ở màn hình 800x600 mặc định mà `placeTrip` không được gọi dù nút tồn tại. Nêu hai nguyên nhân có thể và cách sửa.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> 1 km nằm trong 2 km đầu nên phần vượt bằng 0: dòng hiện <code>0 VND</code>, cả giá mở cửa và tổng đều là <code>12.000 VND</code>. Nhãn trong thẻ là "Giao bưu kiện". Test <code>data dưới 2 km: phần vượt là 0 VND</code> kiểm đúng các con số này; thêm bước <code>tap</code> chip rồi <code>enterText</code> và <code>completeQuote</code> như trong <code>input_test.dart</code>.</p>
<p><strong>2.</strong> Lớp phủ có <code>CircularProgressIndicator</code> chạy mãi, nên <code>pumpAndSettle</code> không bao giờ yên. <code>pump()</code> chỉ dựng một khung. Viết lại: <code>await tester.pump();</code> rồi <code>expect(find.byKey(const Key('placing')), findsOneWidget);</code>, và dọn bằng <code>completePlace</code> trước khi kết thúc test.</p>
<p><strong>3.</strong> Một: ở 800x600 nội dung vẫn cao hơn màn hình nên điểm giữa nút nằm dưới đáy; thêm <code>ensureVisible</code> rồi <code>pump</code>. Hai: lớp phủ <code>ModalBarrier</code> của một lần đặt chuyến khác đang chặn; kiểm <code>find.byKey(const Key('placing'))</code>. Bật <code>hitTestWarningShouldBeFatal</code> chỉ ra nguyên nhân ngay.</p>
</details>

Bài trước: [Unit test với mocktail và fake: quy tắc cước và use case](/articles/flutter-unit-test-mocktail-fake-quy-tac-cuoc-use-case). Bài tiếp: [Golden test và Semantics test: khoá giao diện và accessibility](/articles/flutter-golden-test-semantics-test-giao-dien-on-dinh).

## Tài liệu gốc

- [Introduction to widget testing](https://docs.flutter.dev/cookbook/testing/widget/introduction), [Finding widgets](https://docs.flutter.dev/cookbook/testing/widget/finders), [Tap, drag, and enter text](https://docs.flutter.dev/cookbook/testing/widget/tap-drag) và [Handle scrolling](https://docs.flutter.dev/cookbook/testing/widget/scrolling) (docs.flutter.dev).
- [WidgetTester](https://api.flutter.dev/flutter/flutter_test/WidgetTester-class.html), gồm `pump`, `pumpAndSettle`, `enterText`, `ensureVisible`, `runAsync`, và [CommonFinders](https://api.flutter.dev/flutter/flutter_test/CommonFinders-class.html) (api.flutter.dev).
- [TestFlutterView](https://api.flutter.dev/flutter/flutter_test/TestFlutterView-class.html) và [WidgetController.hitTestWarningShouldBeFatal](https://api.flutter.dev/flutter/flutter_test/WidgetController/hitTestWarningShouldBeFatal.html) (api.flutter.dev).
- [Testing](https://riverpod.dev/docs/how_to/testing) và [Automatic Retry](https://riverpod.dev/docs/concepts2/retry) (riverpod.dev).
- [go_router](https://pub.dev/packages/go_router) và [flutter_test](https://api.flutter.dev/flutter/flutter_test/flutter_test-library.html) (pub.dev, api.flutter.dev).
