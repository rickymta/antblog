---
slug: flutter-navigator-route-truyen-du-lieu-xac-nhan-chuyen
title: "Navigator, route và truyền dữ liệu: từ báo giá sang xác nhận chuyến"
summary: "Bài 12: mở màn hình xác nhận chuyến bằng Navigator.push và nhận Trip? khi pop, truyền báo giá qua constructor, chọn phương thức trả tiền bằng showModalBottomSheet, chặn back bằng PopScope, kiểm mounted sau await và hiểu giới hạn của named route. Có 25 test."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, navigation, ui]
language: vi
---

Khách xem báo giá 5 km: 25.500 VND. Họ bấm "Đặt chuyến" và sang màn hình xác nhận, nơi họ đổi cách trả tiền từ tiền mặt sang ví. Rồi khách chạm nhầm vào nút back. Nếu màn hình đóng ngay, lựa chọn vừa đổi biến mất mà không ai hỏi. Còn khi khách bấm xác nhận, màn hình báo giá phải biết chuyến nào vừa được tạo để hiện "Đã đặt chuyến T-0001".

Đó là hai nửa của điều hướng: đưa dữ liệu đi và nhận kết quả về, rồi kiểm soát chuyện thoát khỏi màn hình. Đây là bài 12 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 11 về Form và validation](/articles/flutter-form-validation-focus-nhap-diem-don-diem-den) và [bài 10 về StatefulWidget](/articles/flutter-statefulwidget-lifecycle-setstate-chon-dich-vu) trước, vì ta dùng `mounted` và `State` ở đây. Mọi ví dụ dùng `Trip`, `Money` và `calculateFare` của package `toi_noi_core` ở [bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core), và bảng giá chỉ là số minh hoạ của hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Màn hình xác nhận được bọc trong <code>PopScope(canPop: false)</code>. Nút "Xác nhận đặt chuyến" bên trong gọi <code>Navigator.of(context).pop(trip)</code>. Lệnh pop này có bị <code>PopScope</code> chặn không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> <code>Navigator.pop</code> gỡ route ngay, không hỏi <code>PopScope</code>. Chỉ <code>Navigator.maybePop</code> mới hỏi, và đó là đường đi của nút back hệ thống và của <code>BackButton</code> trên AppBar. Vì vậy một lối ra có chủ đích như nút xác nhận dùng <code>pop</code>, còn lối ra của người dùng đi qua <code>PopScope</code>. Test <code>nút Xác nhận vẫn pop được khi có thay đổi chưa lưu</code> chứng minh điều này.</p></details>
</div>

## Navigator giữ một chồng route, push trả về Future

`Navigator` giữ một **chồng route** (stack). `push` đặt một route lên đỉnh chồng, `pop` gỡ route đỉnh. `MaterialPageRoute<T>` là route chuyển trang theo phong cách Material, và `T` là kiểu giá trị route trả về khi đóng.

Điểm đáng nhớ nhất là `push<T>` trả về `Future<T?>`. Theo tài liệu API của `Navigator`, Future này hoàn thành với giá trị truyền cho `pop` khi route bị gỡ. Có dấu `?` vì luôn có đường đóng không kèm giá trị, như nút back. Hình 1 vẽ vòng đời của một lần mở màn hình xác nhận.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 1. QuoteScreen gọi push để mở ConfirmTripScreen rồi await. ConfirmTripScreen gọi pop với trip để hoàn thành Future. Future hoàn thành với Trip khi xác nhận, với null khi back hoặc Rời đi, và pop sai kiểu ném lỗi.">
<defs><marker id="fl12-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl12-1-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="240" height="120" rx="10" class="f-phys-soft s-phys"/><text x="36" y="64" class="t-b">QuoteScreen</text><text x="36" y="92" class="t-small">await push&lt;Trip&gt;(...)</text><text x="36" y="116" class="t-small">Future&lt;Trip?&gt; đang chờ</text>
<rect x="620" y="30" width="240" height="120" rx="10" class="f-pk-soft s-pk"/><text x="636" y="64" class="t-b">ConfirmTripScreen</text><text x="636" y="92" class="t-small">MaterialPageRoute&lt;Trip&gt;</text><text x="636" y="116" class="t-small">nhận Quote ở constructor</text>
<path d="M262 72 L618 72" class="f-none s-phys" stroke-width="2" marker-end="url(#fl12-1-a)"/><text x="330" y="60" class="t-mono">push&lt;Trip&gt;(route)</text>
<path d="M618 120 L262 120" class="f-none s-pk" stroke-width="2" marker-end="url(#fl12-1-b)"/><text x="300" y="144" class="t-small">pop(trip): Future hoàn thành với trip</text>
<text x="20" y="200" class="t-b">Ba kết cục của một lần pop</text>
<rect x="20" y="214" width="260" height="96" rx="10" class="f-pk-soft s-pk"/><text x="36" y="244" class="t-b">Trip</text><text x="36" y="270" class="t-small">khách bấm xác nhận</text><text x="36" y="294" class="t-mono">pop(trip)</text>
<rect x="310" y="214" width="260" height="96" rx="10" class="f-phys-soft s-phys"/><text x="326" y="244" class="t-b">null</text><text x="326" y="270" class="t-small">back khi chưa đổi gì,</text><text x="326" y="294" class="t-small">hoặc Rời đi sau khi hỏi</text>
<rect x="600" y="214" width="260" height="96" rx="10" class="f-warn-soft s-warn"/><text x="616" y="244" class="t-b">Lỗi khi pop sai kiểu</text><text x="616" y="270" class="t-small">bật assert: FlutterError</text><text x="616" y="294" class="t-small">tắt assert: TypeError</text>
</svg>
</div>
<figcaption>Hình 1. Màn hình báo giá chờ trên Future của push. Màn hình xác nhận hoàn thành Future bằng pop. Giá trị là Trip hoặc null; pop sai kiểu ném FlutterError khi bật assertions, và vẫn là TypeError khi tắt.</figcaption>
</figure>

Đây là phía gửi trong `QuoteScreen`:

### quote_screen.dart · 12-navigator-route/lib/quote_screen.dart (trích)

```dart
  Future<void> _book() async {
    final trip = await Navigator.of(context).push<Trip>(
      MaterialPageRoute<Trip>(
        builder: (_) =>
            ConfirmTripScreen(quote: widget.quote, clock: widget.clock),
      ),
    );
    // Await có thể kéo dài; nếu màn hình này đã bị gỡ thì dừng tại đây,
    // vì setState và ScaffoldMessenger.of(context) đều cần State còn sống.
    if (!mounted) return;
    setState(() {
      _trip = trip;
      _status = trip == null
          ? 'Bạn đã quay lại, chưa đặt chuyến'
          : 'Đã đặt chuyến ${trip.id}: ${trip.fare}';
    });
```

Ta khai báo kiểu ở hai chỗ: `push<Trip>` và `MaterialPageRoute<Trip>`. Nhờ đó `trip` có kiểu `Trip?` và compiler buộc ta xử lý `null`. Lab quy ước `Trip` là khách đã xác nhận, `null` là khách rời đi. Nếu bỏ kiểu, `push` suy ra `dynamic` và `trip.foo.bar` vẫn qua analyzer. Còn nếu route khai báo `Trip` mà ta `pop('đã đặt')` thì khi bật assertions (debug và test) `Navigator` ném `FlutterError` với thông báo rõ ràng, và test `pop sai kiểu kết quả bị chặn khi bật assertions` đã thử đúng tình huống đó. Chỉ có thông báo là nằm trong `assert`. Khi tắt assertions, kết quả sai kiểu vẫn nổ `TypeError` do kiểm tra kiểu covariant lúc chạy, chỉ mất thông báo dễ hiểu; ta chưa chạy trên bản release thật.

## Truyền dữ liệu bằng constructor

Màn hình xác nhận cần biết quãng đường, địa chỉ và cước. Có thể dùng một biến toàn cục kiểu `currentQuote` mà màn hình báo giá ghi và màn hình xác nhận đọc, hoặc truyền thẳng qua constructor. Lab chọn constructor, vì phụ thuộc hiện ngay trên chữ ký (`required this.quote`), compiler kiểm kiểu, và test chỉ cần dựng màn hình với một `Quote` bất kỳ.

### quote.dart · 12-navigator-route/lib/quote.dart (trích)

```dart
final class Quote {
  const Quote({
    required this.id,
    required this.serviceKind,
    required this.distanceKm,
    required this.pickup,
    required this.dropoff,
  });

  final String id;
  final ServiceKind serviceKind;
  final double distanceKm;
  final String pickup;
  final String dropoff;

  /// Cước theo bảng giá minh hoạ (5 km thì 25.500 VND).
  Money get fare => calculateFare(distanceKm);
}
```

`Quote` chỉ giữ dữ liệu đầu vào. Cước không được lưu mà tính lại bằng `calculateFare` của `toi_noi_core`, nên hai màn hình không thể hiển thị hai con số khác nhau. Test `cước lấy từ calculateFare: 2,001 km là 12.005 VND` kiểm cả hai màn hình với một quãng đường biên. Lab định nghĩa `Quote` cục bộ vì `toi_noi_core` chưa có kiểu này.

Phía nhận chỉ cần giữ đối tượng trong widget:

### confirm_trip_screen.dart · 12-navigator-route/lib/confirm_trip_screen.dart (trích)

```dart
class ConfirmTripScreen extends StatefulWidget {
  const ConfirmTripScreen({
    super.key,
    required this.quote,
    this.clock = systemClock,
  });

  final Quote quote;
  final Clock clock;

  // ...

  void _confirm() {
    final quote = widget.quote;
    final trip = Trip.request(
      id: 'T-${quote.id}',
      serviceKind: quote.serviceKind,
      distanceKm: quote.distanceKm,
      requestedAt: widget.clock(),
    );
    // `Navigator.pop` không hỏi PopScope: đây là lối ra có chủ đích,
    // nên không bị chặn dù `_hasChanges` đang đúng.
    Navigator.of(context).pop(trip);
  }
```

`Trip` chỉ được tạo lúc khách xác nhận, vì `requestedAt` là thời điểm đặt chứ không phải thời điểm báo giá. `Clock` là hàm trả giờ UTC, mặc định là đồng hồ thật, còn trong test ta thay bằng một giờ cố định, nên test so sánh được cả đối tượng `Trip` bằng `==`.

## Bottom sheet và dialog cũng là route

Khách chọn cách trả tiền bằng một bottom sheet. `showModalBottomSheet<T>` và `showDialog<T>` trả về `Future<T?>`, và theo source chúng đẩy một route lên `Navigator` (`ModalBottomSheetRoute`, `DialogRoute`). Vì vậy mọi điều ở mục trên áp dụng nguyên vẹn: kết quả đi qua `pop`, và đóng sheet mà không chọn thì Future cho ra `null`.

### payment_method_sheet.dart · 12-navigator-route/lib/payment_method_sheet.dart

```dart
Future<PaymentMethod?> showPaymentMethodSheet(
  BuildContext context, {
  required PaymentMethod selected,
}) {
  return showModalBottomSheet<PaymentMethod>(
    context: context,
    showDragHandle: true,
    builder: (sheetContext) => SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (final method in PaymentMethod.values)
            ListTile(
              leading: Icon(method.icon),
              title: Text(method.label),
              trailing: method == selected ? const Icon(Icons.check) : null,
              // Đóng đúng route của sheet và mang lựa chọn về cho nơi gọi.
              onTap: () => Navigator.of(sheetContext).pop(method),
            ),
        ],
      ),
    ),
  );
}
```

`sheetContext` là context nằm bên trong route của sheet, và `pop(method)` đưa lựa chọn về cho `await` ở nơi gọi. Nơi gọi phải coi `null` là "giữ nguyên lựa chọn cũ", vì khách có thể chạm ra vùng mờ, vuốt xuống hoặc bấm back:

### confirm_trip_screen.dart · 12-navigator-route/lib/confirm_trip_screen.dart (trích)

```dart
  Future<void> _pickMethod() async {
    final picked = await showPaymentMethodSheet(context, selected: _method);
    // Trong lúc sheet mở, màn hình này có thể đã bị gỡ khỏi cây.
    if (!mounted || picked == null) return;
    setState(() => _method = picked);
  }
```

Test `đóng sheet không chọn thì giữ lựa chọn cũ và back không bị chặn` chạm vùng mờ, rồi kiểm lựa chọn vẫn là "Tiền mặt". Một chi tiết cần nhớ khi có nhiều `Navigator` lồng nhau: theo chữ ký trong source, `showModalBottomSheet` mặc định `useRootNavigator: false` còn `showDialog` mặc định `true`.

## PopScope: chặn back khi có thay đổi chưa lưu

Màn hình xác nhận coi việc đổi khỏi "Tiền mặt" là thay đổi chưa lưu. Khi có thay đổi, ta muốn back không đóng màn hình ngay mà hỏi lại. Đó là việc của `PopScope`. Hình 2 vẽ đường đi của một lần back.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 390" role="img" aria-label="Hình 2. Back hệ thống, BackButton hoặc handlePopRoute đều đi vào Navigator.maybePop, hàm này hỏi popDisposition. Nếu canPop đúng thì pop thật. Nếu canPop sai thì không pop và gọi onPopInvokedWithResult với didPop sai, sau đó hộp thoại hỏi Rời đi hay Ở lại. Rời đi gọi Navigator.pop trực tiếp nên không bị chặn lần hai.">
<defs><marker id="fl12-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl12-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker><marker id="fl12-2-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="30" width="240" height="150" rx="10" class="f-phys-soft s-phys"/><text x="36" y="64" class="t-b">Back của người dùng</text><text x="36" y="94" class="t-small">back hệ thống</text><text x="36" y="118" class="t-small">BackButton trên AppBar</text><text x="36" y="142" class="t-mono">handlePopRoute (test)</text>
<rect x="320" y="60" width="220" height="90" rx="10" class="f-pk-soft s-pk"/><text x="336" y="94" class="t-b">maybePop()</text><text x="336" y="122" class="t-small">hỏi popDisposition</text>
<path d="M262 105 L318 105" class="f-none s-pk" stroke-width="2" marker-end="url(#fl12-2-a)"/>
<rect x="640" y="20" width="220" height="80" rx="10" class="f-pk-soft s-pk"/><text x="656" y="52" class="t-b">canPop == true</text><text x="656" y="78" class="t-small">pop thật, didPop = true</text>
<path d="M542 95 L590 95 L590 60 L638 60" class="f-none s-pk" stroke-width="2" marker-end="url(#fl12-2-a)"/>
<rect x="640" y="130" width="220" height="116" rx="10" class="f-warn-soft s-warn"/><text x="656" y="160" class="t-b">canPop == false</text><text x="656" y="186" class="t-small">không pop, gọi</text><text x="656" y="208" class="t-small">onPopInvokedWithResult</text><text x="656" y="230" class="t-small">với didPop = false</text>
<path d="M542 115 L590 115 L590 188 L638 188" class="f-none s-warn" stroke-width="2" marker-end="url(#fl12-2-w)"/>
<rect x="640" y="286" width="220" height="80" rx="10" class="f-phys-soft s-phys"/><text x="656" y="318" class="t-b">askToLeave</text><text x="656" y="344" class="t-small">hộp thoại Rời đi / Ở lại</text>
<path d="M750 248 L750 284" class="f-none s-warn" stroke-width="2" marker-end="url(#fl12-2-w)"/>
<rect x="20" y="286" width="500" height="80" rx="10" class="f-surface s-muted"/><text x="36" y="314" class="t-b">Navigator.pop() gọi thẳng không hỏi PopScope</text><text x="36" y="338" class="t-small">Nút Xác nhận và nút Rời đi dùng lối này,</text><text x="36" y="358" class="t-small">nên không bị chặn lần hai.</text>
<path d="M638 326 L522 326" class="f-none s-phys" stroke-width="2" marker-end="url(#fl12-2-p)"/><text x="550" y="316" class="t-small">Rời đi</text>
</svg>
</div>
<figcaption>Hình 2. Back đi qua maybePop và PopScope quyết định bằng canPop. Khi bị chặn, callback vẫn được gọi với didPop sai để ta hỏi khách; lựa chọn Rời đi dùng Navigator.pop nên đi thẳng, không qua PopScope.</figcaption>
</figure>

Phần `PopScope` nằm ngay trong `build`:

### confirm_trip_screen.dart · 12-navigator-route/lib/confirm_trip_screen.dart (trích)

```dart
  /// Có thay đổi chưa lưu khi khách đã đổi khỏi lựa chọn mặc định.
  bool get _hasChanges => _method != _defaultMethod;

  // ...

    return PopScope<Trip>(
      canPop: !_hasChanges,
      onPopInvokedWithResult: (didPop, result) {
        if (didPop) return;
        askToLeave(context);
      },
      child: Scaffold(
```

Chữ ký hiện tại, theo source của Flutter 3.47, là `onPopInvokedWithResult(bool didPop, T? result)`. Callback cũ `onPopInvoked` và widget `WillPopScope` đều được đánh dấu deprecated, nên đừng dùng chúng trong code mới. Kiểu `T` của `PopScope<T>` nên khớp hoặc là kiểu cha của kiểu route, vì thế ta viết `PopScope<Trip>` cho route `MaterialPageRoute<Trip>`.

Có hai điều cần hiểu đúng. Thứ nhất, khi `canPop` sai, callback vẫn chạy với `didPop == false`, và đó là lúc ta mở hộp thoại; test `onPopInvokedWithResult báo didPop false khi bị chặn` ghi lại đúng chuỗi `[false]`. Thứ hai, callback chạy sau khi lần pop đã được xử lý, nên không thể huỷ một pop đang diễn ra; muốn chặn phải đặt `canPop` từ trước. Gốc của vấn đề nằm ở cử chỉ back: theo `popGestureEnabled` trong source, route có `popDisposition` là `doNotPop` thì cử chỉ back của người dùng không được bắt đầu, nên predictive back của Android cũng không chạy animation khi `canPop` sai. Đó cũng là lý do `WillPopScope`, vốn quyết định bằng callback bất đồng bộ lúc back đã diễn ra, bị đánh dấu deprecated vì không hoạt động với predictive back. Vì `canPop` đọc từ `_hasChanges`, nó cập nhật theo `setState`: chọn ví thì back bị chặn, chọn lại tiền mặt thì back đi thẳng.

Hộp thoại xác nhận rời đi:

### confirm_trip_screen.dart · 12-navigator-route/lib/confirm_trip_screen.dart (trích)

```dart
Future<void> askToLeave(BuildContext context) async {
  final leave = await showDialog<bool>(
    context: context,
    builder: (dialogContext) => AlertDialog(
      title: const Text('Rời màn hình?'),
      content: const Text('Lựa chọn thanh toán chưa được lưu.'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(dialogContext).pop(false),
          child: const Text('Ở lại'),
        ),
        TextButton(
          onPressed: () => Navigator.of(dialogContext).pop(true),
          child: const Text('Rời đi'),
        ),
      ],
    ),
  );
  // Sau await, context có thể thuộc về màn hình đã bị gỡ.
  if (leave != true || !context.mounted) return;
  Navigator.of(context).pop();
}
```

Ta so sánh `leave != true` thay vì `leave == false`, vì chạm ra ngoài hộp thoại cũng cho `null` và cũng phải hiểu là "ở lại" (test `chạm ra ngoài hộp thoại (kết quả null) thì ở lại`). Khi khách chọn "Rời đi", `Navigator.of(context).pop()` đóng màn hình xác nhận và vì không kèm giá trị, Future ở `QuoteScreen` nhận `null`.

Trên iOS có một khác biệt đáng biết. Ở đó `MaterialPageRoute` dùng `CupertinoPageTransitionsBuilder` nên có cử chỉ vuốt từ mép trái như app iOS. Tài liệu API của `PopScope` nói rằng với `canPop` sai, cử chỉ vuốt từ mép trái của `CupertinoRouteTransitionMixin` không được nhận ra, nên callback không được gọi. Hai test chạy với nền tảng giả lập iOS xác nhận điều đó:

### ios_swipe_test.dart · 12-navigator-route/test/ios_swipe_test.dart (trích)

```dart
      await tester.dragFrom(const Offset(2, 300), const Offset(500, 0));
      await tester.pumpAndSettle();

      expect(find.byType(ConfirmTripScreen), findsOneWidget);
      expect(find.text('Rời màn hình?'), findsNothing);

      // Nút back trên AppBar vẫn đi qua maybePop và hiện hộp thoại.
      await tester.tap(find.byType(BackButton));
      await tester.pumpAndSettle();
      expect(find.text('Rời màn hình?'), findsOneWidget);
```

Nghĩa là trên iPhone, khi có thay đổi chưa lưu, vuốt từ mép trái không làm gì và chỉ nút back trên AppBar mới hiện hộp thoại. Test này chạy trong môi trường test của Flutter, chưa chạy trên iPhone thật.

## Kiểm mounted sau await

Mỗi `await` là một điểm mà thế giới có thể đã đổi. Trong `_book`, nếu `QuoteScreen` bị gỡ khỏi cây trong lúc màn hình xác nhận còn mở (ví dụ phiên đăng nhập hết hạn), `setState` trên một `State` đã `dispose` sẽ ném lỗi. Quy tắc gọn: **sau mỗi `await`, kiểm tra trước khi dùng `State` hoặc `BuildContext`**. Trong lớp `State` ta dùng `mounted`. Trong hàm ngoài lớp `State`, chỉ có một `BuildContext`, ta dùng `context.mounted`.

Lab có cả hai dạng: `if (!mounted) return;` ở `_book` và `_pickMethod`, còn `askToLeave` dùng `!context.mounted`. Test dưới đây dựng `QuoteScreen` trong một widget cha, mở màn hình xác nhận, rồi cho cha gỡ `QuoteScreen` trước khi khách bấm xác nhận:

### navigation_test.dart · 12-navigator-route/test/navigation_test.dart (trích)

```dart
      await _openConfirm(tester);

      showQuote.value = false;
      await tester.pump();
      expect(find.byType(QuoteScreen), findsNothing);

      await tester.tap(find.text('Xác nhận đặt chuyến'));
      await tester.pumpAndSettle();

      expect(tester.takeException(), isNull);
      expect(find.text('đã gỡ báo giá'), findsOneWidget);
```

Test chạy sạch nhờ guard. Khi ta thử bỏ `if (!mounted) return;` trong `_book`, test này đỏ với `setState() called after dispose()`. Phiên bản sai cố ý `_UnguardedButton` nằm trong file test, và test `cùng tình huống nhưng không kiểm mounted thì setState ném lỗi` bắt đúng `FlutterError` đó. Với `context.mounted`, bỏ guard trong `askToLeave` làm test `hộp thoại rời đi: màn hình bị gỡ trước khi bấm thì không pop nhầm` đỏ, vì `Navigator.of(context)` trên context đã bị gỡ ném `TypeError` ("Null check operator used on a null value") trong test. Lint `use_build_context_synchronously` của `flutter_lints` báo chỗ dùng `context` này, còn `setState` đứng một mình sau `await` thì lint không báo, nên `mounted` là việc ta phải tự nhớ.

## Named route và giới hạn của nó

`MaterialApp` còn có cách điều hướng theo tên: `Navigator.pushNamed(context, '/history')`. Các tên khai báo trong bảng `routes`, hoặc được dựng trong `onGenerateRoute` khi cần tham số. Lab dùng đủ cả hai để ta thấy rõ giới hạn.

### routes.dart · 12-navigator-route/lib/routes.dart (trích)

```dart
Route<dynamic>? onGenerateAppRoute(
  RouteSettings settings, {
  Clock clock = systemClock,
}) {
  if (settings.name != AppRoutes.confirm) return null;
  final arguments = settings.arguments;
  if (arguments is! Quote) {
    // Cùng tên route thì mọi nhánh phải cùng kiểu kết quả `Trip`, kể cả
    // trang lỗi: `pushNamed<Trip>` ép kiểu route lúc chạy.
    return MaterialPageRoute<Trip>(
      settings: settings,
      builder: (_) => RouteErrorScreen(
        message:
            '${settings.name} cần một Quote, nhận ${arguments.runtimeType}',
      ),
    );
  }
  return MaterialPageRoute<Trip>(
    settings: settings,
    builder: (_) => ConfirmTripScreen(quote: arguments, clock: clock),
  );
}
```

So với `push` có kiểu, named route mất ba lớp bảo vệ của compiler (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. pushNamed truyền tên và arguments kiểu Object. onGenerateAppRoute kiểm arguments lúc chạy. Route trả về được ép sang Route của T lúc chạy. Ba chỗ lỗi: tên sai đi vào onUnknownRoute, arguments sai hiện trang lỗi, T sai ném TypeError.">
<defs><marker id="fl12-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl12-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="36" y="64" class="t-b">pushNamed&lt;Trip&gt;</text><text x="36" y="92" class="t-small">tên là chuỗi</text><text x="36" y="116" class="t-small">arguments: Object?</text>
<rect x="315" y="30" width="250" height="110" rx="10" class="f-pk-soft s-pk"/><text x="331" y="64" class="t-b">onGenerateAppRoute</text><text x="331" y="92" class="t-small">so tên với AppRoutes</text><text x="331" y="116" class="t-small">arguments is! Quote ?</text>
<rect x="610" y="30" width="250" height="110" rx="10" class="f-phys-soft s-phys"/><text x="626" y="64" class="t-b">Route&lt;dynamic&gt;</text><text x="626" y="92" class="t-small">ép sang Route&lt;Trip?&gt;</text><text x="626" y="116" class="t-small">ngay lúc push</text>
<path d="M272 85 L313 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl12-3-a)"/>
<path d="M567 85 L608 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl12-3-a)"/>
<path d="M145 142 L145 188" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl12-3-w)"/>
<path d="M440 142 L440 188" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl12-3-w)"/>
<path d="M735 142 L735 188" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl12-3-w)"/>
<rect x="20" y="190" width="250" height="90" rx="10" class="f-warn-soft s-warn"/><text x="36" y="222" class="t-b">Tên sai chính tả</text><text x="36" y="250" class="t-small">đi vào onUnknownRoute,</text><text x="36" y="270" class="t-small">compiler không biết</text>
<rect x="315" y="190" width="250" height="90" rx="10" class="f-warn-soft s-warn"/><text x="331" y="222" class="t-b">arguments sai</text><text x="331" y="250" class="t-small">sai kiểu hoặc thiếu: ra</text><text x="331" y="270" class="t-small">trang lỗi lúc chạy</text>
<rect x="610" y="190" width="250" height="90" rx="10" class="f-warn-soft s-warn"/><text x="626" y="222" class="t-b">T sai kiểu</text><text x="626" y="250" class="t-small">pushNamed&lt;int&gt; trên route</text><text x="626" y="270" class="t-small">Trip: TypeError lúc push</text>
</svg>
</div>
<figcaption>Hình 3. Ba chỗ named route chuyển việc kiểm tra từ compiler sang lúc chạy. Cả ba đều có test trong named_route_test.dart.</figcaption>
</figure>

Mỗi chỗ có một test riêng. `arguments` là `Object?`, nên `pushNamed<Trip>('/confirm', arguments: '5 km')` biên dịch bình thường và chỉ hiện trang "cần một Quote, nhận String" khi chạy; thiếu `arguments` thì ra "nhận Null". Còn `T` của `pushNamed` thì `Navigator` ép kiểu route lúc push: `pushNamed<int>` trên route `MaterialPageRoute<Trip>` ném `TypeError`. Lúc viết lab ta vấp đúng lỗi này: trang lỗi ban đầu là `MaterialPageRoute<void>`, nên `pushNamed<Trip>` với `arguments` sai ném `TypeError` thay vì hiện trang lỗi. Vì thế nhánh lỗi cũng ghi `MaterialPageRoute<Trip>`.

Ngoài ra, trang "Navigation and routing" của docs.flutter.dev khuyên không dùng named route cho hầu hết ứng dụng, và nói khi deep link tới, Flutter đẩy một route mới lên `Navigator` bất kể người dùng đang ở đâu. Vì thế luồng chính của lab dùng `push` có kiểu với constructor, còn named route chỉ để minh hoạ. Khi app cần deep link và bảo vệ route theo đăng nhập, bài 23 của loạt bài chuyển sang `go_router`.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi điều hướng</span>
<ul>
<li><strong>Dùng <code>setState</code> hoặc <code>context</code> sau <code>await</code> mà không kiểm <code>mounted</code>.</strong> Dấu hiệu: <code>setState() called after dispose()</code>, hoặc lỗi tra cứu <code>Navigator</code> từ context đã bị gỡ; lint <code>use_build_context_synchronously</code> báo phần dùng context. Cách sửa: <code>if (!mounted) return;</code> trong <code>State</code>, <code>if (!context.mounted) return;</code> trong hàm ngoài lớp.</li>
<li><strong>Nghĩ <code>PopScope</code> chặn mọi lần pop.</strong> Dấu hiệu: nút "Xác nhận" vẫn đóng được màn hình dù <code>canPop</code> sai. Nguyên nhân: <code>pop</code> không hỏi <code>PopScope</code>, chỉ <code>maybePop</code> hỏi. Cách sửa: lối ra của người dùng đi qua back, lối ra có chủ đích dùng <code>pop</code> kèm kết quả.</li>
<li><strong><code>canPop: false</code> mà không có lối ra.</strong> Dấu hiệu: back không làm gì, trên iOS vuốt từ mép trái cũng không có phản ứng nào. Nguyên nhân: cử chỉ iOS không gọi callback khi <code>canPop</code> sai. Cách sửa: gắn <code>canPop</code> với điều kiện thật như <code>!_hasChanges</code> và luôn có nút rõ ràng để thoát.</li>
<li><strong>Bỏ kiểu của route.</strong> Dấu hiệu: <code>await Navigator.of(context).push(MaterialPageRoute(...))</code> cho kết quả <code>dynamic</code> và <code>r.foo.bar</code> vẫn qua analyzer. Cách sửa: viết <code>push&lt;Trip&gt;</code> cùng <code>MaterialPageRoute&lt;Trip&gt;</code>, rồi xử lý đủ hai nhánh của <code>Trip?</code>.</li>
<li><strong>Truyền dữ liệu qua biến toàn cục hoặc <code>arguments</code> không kiểm.</strong> Dấu hiệu: màn hình xác nhận hiện báo giá cũ, hoặc test phải dựng trạng thái ở nơi khác mới chạy được. Cách sửa: nhận đối tượng qua constructor; nếu dùng named route thì kiểm <code>arguments is Quote</code> và cho trang lỗi cùng kiểu kết quả.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/12-navigator-route
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` 0 issue, `flutter test` 25 test xanh. Mức: **Kiểm chứng tự động**, không cần simulator hay thiết bị.

| Test | Chứng minh điều gì |
|---|---|
| `navigation_test.dart` (17) | `Quote` đi qua constructor và cước khớp `calculateFare`; `push<Trip>` hoàn thành với đúng `Trip`, chưa pop thì Future chưa xong; pop sai kiểu bị assert; back không có thay đổi trả `null`; sheet cập nhật hoặc giữ nguyên lựa chọn; `PopScope` chặn back, hỏi, ở lại, rời đi, chạm ra ngoài; `onPopInvokedWithResult` nhận `didPop == false`; kiểm `mounted` và `context.mounted` |
| `named_route_test.dart` (6) | `/history` từ bảng routes; `/confirm` với `Quote` trả `Trip`; `arguments` sai kiểu hoặc thiếu ra trang lỗi; `pushNamed<int>` ném `TypeError`; tên lạ vào `onUnknownRoute` |
| `ios_swipe_test.dart` (2) | Trên nền tảng iOS giả lập, vuốt từ mép trái quay lại được khi `canPop` đúng và không làm gì khi `canPop` sai, còn nút back vẫn hiện hộp thoại |

Ta còn thử sửa code để chắc test bắt được lỗi: bỏ `!context.mounted`, bỏ `!mounted` ở `QuoteScreen`, hoặc đặt `canPop: true` đều làm ít nhất một test đỏ.

**Chưa làm**: chạy trên iPhone hay Android thật, cả cử chỉ back của hệ thống lẫn predictive back của Android; `go_router`, deep link và `Router` (bài 23); gửi chuyến lên API; lưu phương thức thanh toán vào `Trip`, vì `Trip` của `toi_noi_core` chưa có trường này; animation chuyển trang tuỳ chỉnh.

## Bài tập

1. Màn hình xác nhận trả về `Trip?`. Ta muốn màn hình báo giá biết cả phương thức trả tiền khách đã chọn. Cần sửa những chỗ nào để route trả về một record `(Trip, PaymentMethod)?` mà vẫn an toàn kiểu?
2. Khách chọn ví rồi chọn lại tiền mặt, sau đó bấm back. Hộp thoại "Rời màn hình?" có hiện không, và vì sao?
3. Một đồng nghiệp viết `Navigator.of(context).pushNamed<Trip>('/confirm')` trong màn hình báo giá và quên `arguments`. Compiler có báo gì không, và khi chạy chuyện gì xảy ra?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Đổi kiểu ở nhiều nơi: <code>push&lt;(Trip, PaymentMethod)&gt;</code> cùng <code>MaterialPageRoute&lt;(Trip, PaymentMethod)&gt;</code> ở <code>QuoteScreen</code>, <code>pop((trip, _method))</code> ở <code>_confirm</code>, <code>PopScope&lt;(Trip, PaymentMethod)&gt;</code> cho khớp kiểu route, và hai <code>MaterialPageRoute&lt;Trip&gt;</code> của <code>/confirm</code> trong <code>routes.dart</code> (cả nhánh lỗi), nếu không <code>pushNamed</code> sẽ ném <code>TypeError</code>. Cập nhật luôn <code>named_route_test.dart</code> và các test đọc kết quả. Phía nhận dùng pattern <code>if (result case (final trip, final method))</code> và vẫn xử lý nhánh <code>null</code>.</p>
<p><strong>2.</strong> Không hiện. <code>_hasChanges</code> so lựa chọn hiện tại với mặc định, nên khi đã về "Tiền mặt" thì <code>canPop</code> thành <code>true</code> và back pop thẳng. Test <code>đổi lại về Tiền mặt thì back không còn bị chặn</code> kiểm đúng chuỗi thao tác này.</p>
<p><strong>3.</strong> Compiler không báo gì, vì <code>arguments</code> là <code>Object?</code> và tên route là chuỗi. Khi chạy, <code>onGenerateAppRoute</code> thấy <code>arguments is! Quote</code> và dựng trang lỗi "/confirm cần một Quote, nhận Null". Trang lỗi cùng kiểu <code>MaterialPageRoute&lt;Trip&gt;</code> nên không có <code>TypeError</code>.</p>
</details>

Bài trước: [Form, TextFormField và validation](/articles/flutter-form-validation-focus-nhap-diem-don-diem-den). Bài tiếp: [Theme Material 3, màu, chữ và dark mode](/articles/flutter-theme-material-3-dark-mode-design-system-toi-noi).

## Tài liệu gốc

- [Navigation and routing](https://docs.flutter.dev/ui/navigation) và [Send data to a new screen](https://docs.flutter.dev/cookbook/navigation/passing-data) (docs.flutter.dev).
- [Return data from a screen](https://docs.flutter.dev/cookbook/navigation/returning-data) và [Navigate with named routes](https://docs.flutter.dev/cookbook/navigation/named-routes) (docs.flutter.dev).
- [Navigator](https://api.flutter.dev/flutter/widgets/Navigator-class.html), [MaterialPageRoute](https://api.flutter.dev/flutter/material/MaterialPageRoute-class.html) và [PopScope](https://api.flutter.dev/flutter/widgets/PopScope-class.html) (api.flutter.dev).
- [showModalBottomSheet](https://api.flutter.dev/flutter/material/showModalBottomSheet.html) và [showDialog](https://api.flutter.dev/flutter/material/showDialog.html) (api.flutter.dev).
- [BuildContext.mounted](https://api.flutter.dev/flutter/widgets/BuildContext/mounted.html) và [use_build_context_synchronously](https://dart.dev/tools/linter-rules/use_build_context_synchronously) (api.flutter.dev, dart.dev).
- [Android predictive back](https://docs.flutter.dev/platform-integration/android/predictive-back) (docs.flutter.dev).
