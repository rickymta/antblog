---
slug: flutter-state-inheritedwidget-changenotifier-bao-gia
title: "Quản lý state: InheritedWidget, ChangeNotifier và ranh giới rebuild"
summary: "Bài 17: đưa bản nháp báo giá và danh sách chuyến ra khỏi State của khung app bằng InheritedWidget, ChangeNotifier, ValueNotifier và ListenableBuilder. Test đếm số lần build chỉ ra ranh giới rebuild; so sánh ngắn provider, bloc, Riverpod và lý do lộ trình chọn Riverpod."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, state-management, changenotifier, inheritedwidget]
language: vi
---

Khách đặt xe trên app Tới Nơi kéo thanh quãng đường, và nhãn cước phải đổi ngay. Đặt chuyến xong, chuyến mới phải hiện ở lịch sử. Mở chuyến ra huỷ, danh sách phía sau phải thấy chữ "Đã huỷ". Ở bài 16 ta giữ `_trips` và `_replace` trong `State` của khung app, truyền xuống bằng tham số và báo ngược lên bằng callback.

Thêm màn thứ tư, ta lại thêm tham số cho từng tầng ở giữa, kể cả tầng không dùng dữ liệu đó. Bài này đưa dữ liệu dùng chung ra khỏi `State` và kiểm soát widget nào dựng lại khi nó đổi.

Đây là bài 17 của lộ trình **Flutter qua Tới Nơi**, bài đầu tiên của Phần 3, nối tiếp [bài 16 về accessibility, responsive và adaptive](/articles/flutter-accessibility-responsive-adaptive-ios-android). Bạn cần biết `setState` ở [bài 10](/articles/flutter-statefulwidget-lifecycle-setstate-chon-dich-vu) và `Navigator` ở [bài 12](/articles/flutter-navigator-route-truyen-du-lieu-xac-nhan-chuyen). Bối cảnh nghiệp vụ nằm ở hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); bảng giá trong ví dụ là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ba widget cùng hiển thị cước của một <code>QuoteDraft</code>. A đọc bằng <code>QuoteScope.of(context).fare</code>. B lấy draft bằng <code>QuoteScope.read(context)</code> rồi chỉ nghe riêng <code>fareListenable</code> qua <code>ValueListenableBuilder</code>. C nhận cả draft qua constructor và bọc <code>ListenableBuilder(listenable: draft)</code>. Ở quãng đường 1 km (cước 12.000 VND), khách đổi từ "Đặt xe" sang "Giao đồ ăn". Mỗi widget được dựng thêm bao nhiêu lần?</p>
<details><summary>Xem đáp án</summary><p>A dựng thêm 1 lần, B không lần nào, C dựng thêm 1 lần. B chỉ nghe thứ riêng của cước, và <code>ValueNotifier</code> không báo khi giá trị mới bằng giá trị cũ. A và C nghe cả draft nên bị đánh thức dù thứ đổi không liên quan. Test <code>rebuild_boundary_test.dart</code> đo đúng ba con số này.</p></details>
</div>

## State sống ở đâu: tạm thời hay của cả app

Tài liệu Flutter chia state làm hai loại. **Ephemeral state** gói gọn trong một widget, như tab đang mở hay tiến độ một animation, và chỉ cần `StatefulWidget`. **App state** là thứ nhiều phần của app cùng đọc hoặc đổi, hoặc cần giữ giữa các phiên dùng. Tài liệu nói rõ không có quy tắc cứng: bạn có thể dùng `setState` cho mọi thứ.

Với Tới Nơi, tab đang mở là ephemeral: không màn nào khác cần biết. Bản nháp báo giá (dịch vụ, quãng đường, cước) và danh sách chuyến là app state: màn báo giá, lịch sử và trang chi tiết đều đụng tới chúng. Bài 16 giữ chúng trong `State` của khung app:

### adaptive_shell.dart · 16-a11y-responsive-adaptive/lib/adaptive_shell.dart (trích)

```dart
  void _replace(Trip updated) => setState(() {
    _trips = [for (final t in _trips) t.isSameTrip(updated) ? updated : t];
  });

  // ...
        builder: (_) => TripDetailPage(trip: trip, onChanged: _replace),
```

Hình 1: mọi tầng ở giữa phải mang `trips` hay `onChanged` dù không dùng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 1. Bên trái là bài 16: AdaptiveShell giữ _trips và _replace, truyền xuống HistoryScreen, TripDetailPage và TripDetailView bằng tham số, còn thay đổi đi ngược lên bằng callback qua từng tầng. Bên phải là bài 17: TripBookScope đặt phía trên MaterialApp, HistoryScreen và TripDetailPage tự tra cứu scope bằng of và read, không có tham số hay callback ở các tầng giữa."><defs><marker id="fl17-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl17-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="20" width="400" height="320" rx="10" class="f-surface s-line"/><text x="36" y="48" class="t-b">Bài 16: callback đi qua từng tầng</text>
<rect x="40" y="64" width="360" height="40" rx="8" class="f-pk-soft s-pk"/><text x="56" y="89" class="t-small">AdaptiveShell: giữ _trips, _replace</text>
<rect x="40" y="138" width="360" height="40" rx="8" class="f-phys-soft s-phys"/><text x="56" y="163" class="t-small">HistoryScreen(trips, onSelect)</text>
<rect x="40" y="212" width="360" height="40" rx="8" class="f-phys-soft s-phys"/><text x="56" y="237" class="t-small">TripDetailPage(trip, onChanged)</text>
<rect x="40" y="286" width="360" height="40" rx="8" class="f-phys-soft s-phys"/><text x="56" y="311" class="t-small">TripDetailView(trip, onCancelled)</text>
<path d="M120 104 L120 136" class="f-none s-pk" stroke-width="2" marker-end="url(#fl17-1-a)"/><text x="130" y="125" class="t-small">trips</text>
<path d="M120 178 L120 210" class="f-none s-pk" stroke-width="2" marker-end="url(#fl17-1-a)"/><text x="130" y="199" class="t-small">trip</text>
<path d="M120 252 L120 284" class="f-none s-pk" stroke-width="2" marker-end="url(#fl17-1-a)"/><text x="130" y="273" class="t-small">trip</text>
<path d="M330 284 L330 254" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl17-1-w)"/><text x="244" y="273" class="t-small">callback</text>
<path d="M330 210 L330 180" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl17-1-w)"/><text x="244" y="199" class="t-small">callback</text>
<path d="M330 136 L330 106" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl17-1-w)"/><text x="244" y="125" class="t-small">callback</text>
<rect x="460" y="20" width="400" height="320" rx="10" class="f-surface s-line"/><text x="476" y="48" class="t-b">Bài 17: widget tự tìm scope</text>
<rect x="480" y="64" width="360" height="44" rx="8" class="f-pk-soft s-pk"/><text x="496" y="92" class="t-small">TripBookScope (trên MaterialApp)</text>
<rect x="480" y="190" width="170" height="44" rx="8" class="f-phys-soft s-phys"/><text x="496" y="218" class="t-small">HistoryScreen</text>
<rect x="670" y="190" width="170" height="44" rx="8" class="f-phys-soft s-phys"/><text x="686" y="218" class="t-small">TripDetailPage</text>
<path d="M565 188 L565 110" class="f-none s-pk" stroke-width="2" marker-end="url(#fl17-1-a)"/><text x="575" y="155" class="t-small">of(context)</text>
<path d="M715 188 L715 110" class="f-none s-pk" stroke-width="2" marker-end="url(#fl17-1-a)"/><text x="690" y="155" class="t-small">of</text>
<path d="M795 188 L795 110" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl17-1-w)"/><text x="803" y="155" class="t-small">read</text>
<text x="480" y="284" class="t-small">Các tầng ở giữa không còn tham số trips</text><text x="480" y="306" class="t-small">hay callback onChanged.</text>
</svg>
</div>
<figcaption>Hình 1. Bên trái: dữ liệu đi xuống bằng tham số, thay đổi đi ngược lên bằng callback qua từng tầng. Bên phải: widget tra cứu thẳng scope ở trên. Mũi tên chỉ hướng tra cứu từ widget lên scope; nét đứt là <code>read</code>, tra cứu mà không đăng ký phụ thuộc.</figcaption>
</figure>

## InheritedWidget: tra cứu từ dưới lên và `updateShouldNotify`

`InheritedWidget` cho widget con lấy dữ liệu của tổ tiên mà không cần ai truyền qua tham số; `Theme.of(context)` là một ví dụ. Mỗi Element giữ bảng ánh xạ từ kiểu của `InheritedWidget` tới `InheritedElement` gần nhất phía trên, nên tra cứu là O(1). Kiểu phải là lớp cụ thể, vì thế tên API là `...OfExactType`.

Có hai cách tra cứu, khác nhau ở một điểm:

- `context.dependOnInheritedWidgetOfExactType<T>()` trả về widget và **đăng ký** `context` làm dependent. Khi widget đó được cập nhật và `updateShouldNotify` trả `true`, framework gọi `didChangeDependencies` của từng dependent, tức dựng lại chúng.
- `context.getInheritedWidgetOfExactType<T>()` chỉ trả về widget, **không đăng ký**. Context đó không bị dựng lại khi dữ liệu đổi.

Quy ước bọc chúng trong hàm tĩnh `of` và `maybeOf`. Lab có một `InheritedWidget` viết tay mang ảnh chụp bất biến của bản nháp:

### snapshot_scope.dart · 17-state-changenotifier/lib/snapshot_scope.dart (trích)

```dart
class QuoteSnapshotScope extends InheritedWidget {
  const QuoteSnapshotScope({
    super.key,
    required this.snapshot,
    required super.child,
  });

  final QuoteSnapshot snapshot;

  /// Đăng ký phụ thuộc rồi trả ảnh chụp hiện tại.
  static QuoteSnapshot of(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<QuoteSnapshotScope>();
    assert(scope != null, 'Thiếu QuoteSnapshotScope phía trên');
    return scope!.snapshot;
  }

  @override
  bool updateShouldNotify(QuoteSnapshotScope oldWidget) =>
      snapshot != oldWidget.snapshot;
}
```

`updateShouldNotify` chỉ chạy khi cha dựng lại và đưa vào một `InheritedWidget` mới thay cho cái cũ. Nó so hai ảnh chụp theo giá trị: bằng nhau thì không dependent nào bị đánh thức. Test kiểm cả hai chiều, kèm một scope đối chứng luôn trả `true`:

### snapshot_scope_test.dart · 17-state-changenotifier/test/snapshot_scope_test.dart (trích)

```dart
        // Một QuoteSnapshot mới tinh nhưng bằng giá trị cũ.
        rebuild(() => snapshot = _snapshot());
        await tester.pump();
        expect(builds, 1);

        // Đổi thật: dependent được đánh thức và thấy cước mới.
        rebuild(() => snapshot = _snapshot(km: 6));
        await tester.pump();
        expect(builds, 2);
        expect(find.text('30.000 VND'), findsOneWidget);
```

Hai lưu ý. Một, `child` truyền vào scope phải là cùng một đối tượng ở mọi lần dựng; cha tạo widget con mới mỗi lần thì con vẫn bị dựng lại dù `updateShouldNotify` trả `false`. Hai, `updateShouldNotify` là một quyết định chung cho mọi dependent. Đổi nó thành chỉ so cước thì widget đọc dịch vụ bị bỏ sót: test `chỉ so cước` cho thấy chip vẫn hiện "ride" sau khi dịch vụ đã đổi. Muốn đăng ký theo từng phần dữ liệu, Flutter có `InheritedModel` với tham số `aspect`; lab không đi hướng đó.

`InheritedWidget` bất biến và không biết lắng nghe, nên lab thêm một `StatefulWidget` đóng vai chủ:

### snapshot_scope.dart · 17-state-changenotifier/lib/snapshot_scope.dart (trích)

```dart
class _QuoteSnapshotHostState extends State<QuoteSnapshotHost> {
  @override
  void initState() {
    super.initState();
    widget.draft.addListener(_onChanged);
  }

  // ...

  @override
  void dispose() {
    widget.draft.removeListener(_onChanged);
    super.dispose();
  }

  void _onChanged() => setState(() {});

  @override
  Widget build(BuildContext context) => QuoteSnapshotScope(
    snapshot: QuoteSnapshot.of(widget.draft),
    child: widget.child,
  );
}
```

Đây là việc Flutter đã làm sẵn trong `InheritedNotifier`, nên ta sang đó.

## ChangeNotifier và InheritedNotifier: bỏ phần chủ viết tay

`ChangeNotifier` là một đối tượng có danh sách listener và phương thức `notifyListeners`. Phương thức đó được đánh dấu `@protected`, nên chỉ lớp con mới gọi được, và lớp con thường để lộ các hàm nghiệp vụ như `setKm`. `QuoteDraft` giữ dịch vụ, quãng đường và cước:

### quote_draft.dart · 17-state-changenotifier/lib/quote_draft.dart (trích)

```dart
  QuoteDraft({this._service = ServiceKind.ride, double km = 5})
    : _km = km,
      _fare = ValueNotifier(calculateFare(km));

  // ...

  void selectService(ServiceKind service) {
    if (service == _service) return;
    _service = service;
    notifyListeners();
  }

  /// Đổi quãng đường. Ném [ArgumentError] (từ `calculateFare`) khi ngoài miền
  /// 0..100 km, và khi đó trạng thái không đổi.
  void setKm(double km) {
    if (km == _km) return;
    final fare = calculateFare(km);
    _km = km;
    _fare.value = fare;
    notifyListeners();
  }
```

Ba quy tắc. Chỉ báo khi có gì đó thật sự đổi. Tính trước rồi mới gán, để lỗi không để lại trạng thái nửa vời (`setKm(101)` ném `ArgumentError` mà draft không đổi). Cước vẫn do `calculateFare` của `toi_noi_core` tính, không có công thức thứ hai trong UI. Notifier cũng phải được huỷ: sau `dispose`, thêm listener bị chặn ở chế độ debug.

Constructor viết `this._service` là tham số tên cho thành viên private (Dart 3.12): khi gọi, tên vẫn là `service`.

Để đưa draft xuống cây, ta dùng `InheritedNotifier<T>`, một `InheritedWidget` nhận một `Listenable`:

### quote_scope.dart · 17-state-changenotifier/lib/quote_scope.dart (trích)

```dart
class QuoteScope extends InheritedNotifier<QuoteDraft> {
  const QuoteScope({super.key, required QuoteDraft draft, required super.child})
    : super(notifier: draft);

  static QuoteDraft? maybeOf(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<QuoteScope>()?.notifier;

  static QuoteDraft of(BuildContext context) =>
      maybeOf(context) ?? (throw _missing(context));

  static QuoteDraft read(BuildContext context) =>
      context.getInheritedWidgetOfExactType<QuoteScope>()?.notifier ??
      (throw _missing(context));
```

Source `InheritedNotifier` trong Flutter 3.47 làm đúng việc của `QuoteSnapshotHost` ở tầng Element: đăng ký listener lên notifier ngay khi tạo; khi notifier báo, tự đánh dấu cần dựng lại; ở lần build kế tiếp gọi `notifyClients` để đánh thức mọi dependent. Vì chỉ đánh dấu rồi đợi build, nhiều thông báo giữa hai khung hình gộp thành một: test gọi `selectService`, `setKm`, `setKm` liền nhau và dependent chỉ dựng thêm một lần. `updateShouldNotify` của `InheritedNotifier` mặc định chỉ so danh tính notifier (`oldWidget.notifier != notifier`), tức chỉ cần khi đổi sang một draft khác.

Hình 2 vẽ đường đi của một thông báo.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. QuoteDraft gọi notifyListeners, QuoteScope nhận thông báo, đánh dấu cần build rồi báo các dependent đã đăng ký bằng of: _ServiceChips và _KmSlider. _BookButton chỉ tra cứu bằng read nên không được báo. Bên trong QuoteDraft có ValueNotifier fare; FareFromValue nghe trực tiếp nó nên chỉ dựng lại khi cước đổi."><defs><marker id="fl17-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl17-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker><marker id="fl17-2-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="50" width="210" height="130" rx="10" class="f-pk-soft s-pk"/><text x="36" y="76" class="t-b">QuoteDraft</text><text x="36" y="96" class="t-small">ChangeNotifier: service, km</text>
<rect x="36" y="110" width="178" height="56" rx="8" class="f-phys-soft s-phys"/><text x="48" y="132" class="t-b">fare</text><text x="48" y="152" class="t-small">ValueNotifier&lt;Money&gt;</text>
<rect x="330" y="50" width="230" height="130" rx="10" class="f-pk-soft s-pk"/><text x="346" y="76" class="t-b">QuoteScope</text><text x="346" y="98" class="t-small">InheritedNotifier&lt;QuoteDraft&gt;</text><text x="346" y="122" class="t-small">nghe draft, đánh dấu dirty,</text><text x="346" y="142" class="t-small">build kế tiếp: báo dependent</text>
<rect x="640" y="30" width="220" height="48" rx="8" class="f-phys-soft s-phys"/><text x="656" y="52" class="t-b">_ServiceChips</text><text x="656" y="68" class="t-small">of(context)</text>
<rect x="640" y="92" width="220" height="48" rx="8" class="f-phys-soft s-phys"/><text x="656" y="114" class="t-b">_KmSlider</text><text x="656" y="130" class="t-small">of(context)</text>
<rect x="640" y="154" width="220" height="48" rx="8" class="f-surface s-line"/><text x="656" y="176" class="t-b">_BookButton</text><text x="656" y="192" class="t-small">read(context), không nghe</text>
<rect x="640" y="270" width="220" height="64" rx="8" class="f-phys-soft s-phys"/><text x="656" y="294" class="t-b">FareFromValue</text><text x="656" y="314" class="t-small">read + ValueListenableBuilder</text>
<path d="M232 100 L328 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl17-2-a)"/><text x="248" y="92" class="t-small">notify</text>
<path d="M562 84 L638 56" class="f-none s-pk" stroke-width="2" marker-end="url(#fl17-2-a)"/>
<path d="M562 110 L638 116" class="f-none s-pk" stroke-width="2" marker-end="url(#fl17-2-a)"/>
<path d="M638 178 L562 150" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl17-2-w)"/>
<path d="M132 166 L132 302 L638 302" class="f-none s-phys" stroke-width="2" marker-end="url(#fl17-2-p)"/><text x="300" y="292" class="t-small">chỉ khi cước đổi</text>
<text x="20" y="364" class="t-small">Nét liền: thông báo thay đổi. Nét đứt: chỉ tra cứu, không đăng ký phụ thuộc.</text>
</svg>
</div>
<figcaption>Hình 2. Một thông báo của <code>QuoteDraft</code> đi tới đâu. Widget gọi <code>of</code> được đánh thức mỗi lần draft báo; widget gọi <code>read</code> thì không; widget nghe riêng <code>ValueNotifier</code> của cước chỉ được đánh thức khi cước đổi.</figcaption>
</figure>

Ai tạo notifier thì người đó huỷ nó. `State` của `ToiNoiApp` tạo hai notifier, huỷ trong `dispose`, và đặt hai scope **trên** `MaterialApp`:

### app.dart · 17-state-changenotifier/lib/app.dart (trích)

```dart
  @override
  void dispose() {
    _draft.dispose();
    _book.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return QuoteScope(
      draft: _draft,
      child: TripBookScope(
        book: _book,
        child: MaterialApp(
```

Vị trí đó quan trọng. Route được đẩy bằng `Navigator` nằm dưới `MaterialApp`, không dưới `home`. Test đặt scope trong `home` thì trang được đẩy ném `FlutterError` "Không tìm thấy QuoteScope"; đặt trên `MaterialApp` thì trang đọc được.

## ListenableBuilder, ValueNotifier và ranh giới rebuild

Dựng lại một `Text` rất rẻ; điều cần là biết **ranh giới** rebuild để cắt đúng chỗ khi màn hình nặng lên. Lab có ba widget cùng hiển thị cước, mỗi cái đọc một kiểu, và bộ đếm `BuildCounts` ghi số lần build.

Cách 1 là `QuoteScope.of`. Cách 3 là truyền cả notifier rồi bọc `ListenableBuilder`, dựng lại mỗi lần `listenable` báo. Cách 2 mới thú vị:

### fare_readers.dart · 17-state-changenotifier/lib/fare_readers.dart (trích)

```dart
  @override
  Widget build(BuildContext context) {
    counts?.hit('value:widget');
    final draft = QuoteScope.read(context);
    return ValueListenableBuilder<Money>(
      valueListenable: draft.fareListenable,
      child: _Caption(counts),
      builder: (context, fare, caption) {
        counts?.hit('value');
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [caption!, MoneyText(fare)],
        );
      },
    );
  }
```

Widget này lấy draft bằng `read`, nên không phụ thuộc scope và `build` của nó chạy đúng một lần. Nó chỉ nghe `fareListenable`. `ValueNotifier` chỉ báo khi giá trị mới khác giá trị cũ theo `==`, và `Money` so sánh theo số đồng. Hậu quả: đổi từ 1 km sang 1,5 km làm draft báo nhưng cước vẫn 12.000 nên không ai nghe cước bị gọi. Tham số `child` là phần không đổi ("Cước dự kiến"), dựng một lần rồi dùng lại mỗi lần `builder` chạy.

Test dựng ba widget trong cùng một scope, bắt đầu ở 1 km, rồi bắn từng loại thay đổi:

### rebuild_boundary_test.dart · 17-state-changenotifier/test/rebuild_boundary_test.dart (trích)

```dart
    testWidgets('đổi dịch vụ (thứ không liên quan tới cước)', (tester) async {
      final (draft, counts) = await _pump(tester);

      draft.selectService(ServiceKind.food);
      await tester.pump();

      // scope và whole bị đánh thức oan; value đứng yên.
      expect(_tally(counts), (2, 1, 2));
    });
```

Hình 3 tổng hợp kết quả của ba ca thay đổi. Mỗi ô là số lần build **thêm** sau lần dựng đầu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Bảng số lần build thêm của ba cách đọc cước. FareFromScope: thêm 1 khi đổi dịch vụ, thêm 1 khi đổi km mà cước vẫn 12.000, thêm 1 khi cước đổi. FareFromValue: thêm 0, thêm 0, thêm 1. FareFromWholeNotifier: thêm 1, thêm 1, thêm 1. Ô thêm 1 khi cước không đổi là dựng lại oan.">
<rect x="20" y="20" width="280" height="52" class="f-surface s-line"/><text x="36" y="52" class="t-b">Cách đọc cước</text>
<rect x="300" y="20" width="190" height="52" class="f-surface s-line"/><text x="316" y="42" class="t-b">Đổi dịch vụ</text><text x="316" y="60" class="t-small">cước vẫn 12.000</text>
<rect x="490" y="20" width="190" height="52" class="f-surface s-line"/><text x="506" y="42" class="t-b">Đổi km 1 sang 1,5</text><text x="506" y="60" class="t-small">cước vẫn 12.000</text>
<rect x="680" y="20" width="190" height="52" class="f-surface s-line"/><text x="696" y="42" class="t-b">Đổi km 1 sang 5</text><text x="696" y="60" class="t-small">cước thành 25.500</text>
<rect x="20" y="72" width="280" height="68" class="f-surface s-line"/><text x="36" y="102" class="t-b">FareFromScope</text><text x="36" y="122" class="t-small">QuoteScope.of(context)</text>
<rect x="300" y="72" width="190" height="68" class="f-warn-soft s-warn"/><text x="316" y="116" class="t-big t-warn">+1</text><text x="372" y="114" class="t-small">dựng lại oan</text>
<rect x="490" y="72" width="190" height="68" class="f-warn-soft s-warn"/><text x="506" y="116" class="t-big t-warn">+1</text><text x="562" y="114" class="t-small">dựng lại oan</text>
<rect x="680" y="72" width="190" height="68" class="f-pk-soft s-pk"/><text x="696" y="116" class="t-big t-pk">+1</text><text x="752" y="114" class="t-small">đúng lúc</text>
<rect x="20" y="140" width="280" height="68" class="f-surface s-line"/><text x="36" y="170" class="t-b">FareFromValue</text><text x="36" y="190" class="t-small">read + ValueListenableBuilder</text>
<rect x="300" y="140" width="190" height="68" class="f-phys-soft s-phys"/><text x="316" y="184" class="t-big t-phys">+0</text><text x="372" y="182" class="t-small">không dựng lại</text>
<rect x="490" y="140" width="190" height="68" class="f-phys-soft s-phys"/><text x="506" y="184" class="t-big t-phys">+0</text><text x="562" y="182" class="t-small">không dựng lại</text>
<rect x="680" y="140" width="190" height="68" class="f-pk-soft s-pk"/><text x="696" y="184" class="t-big t-pk">+1</text><text x="752" y="182" class="t-small">đúng lúc</text>
<rect x="20" y="208" width="280" height="68" class="f-surface s-line"/><text x="36" y="238" class="t-b">FareFromWholeNotifier</text><text x="36" y="258" class="t-small">ListenableBuilder(draft)</text>
<rect x="300" y="208" width="190" height="68" class="f-warn-soft s-warn"/><text x="316" y="252" class="t-big t-warn">+1</text><text x="372" y="250" class="t-small">dựng lại oan</text>
<rect x="490" y="208" width="190" height="68" class="f-warn-soft s-warn"/><text x="506" y="252" class="t-big t-warn">+1</text><text x="562" y="250" class="t-small">dựng lại oan</text>
<rect x="680" y="208" width="190" height="68" class="f-pk-soft s-pk"/><text x="696" y="252" class="t-big t-pk">+1</text><text x="752" y="250" class="t-small">đúng lúc</text>
<rect x="20" y="298" width="16" height="16" class="f-pk-soft s-pk"/><text x="44" y="311" class="t-small">dựng lại đúng lúc</text>
<rect x="220" y="298" width="16" height="16" class="f-warn-soft s-warn"/><text x="244" y="311" class="t-small">dựng lại oan</text>
<rect x="400" y="298" width="16" height="16" class="f-phys-soft s-phys"/><text x="424" y="311" class="t-small">không dựng lại</text>
</svg>
</div>
<figcaption>Hình 3. Số lần build thêm, đo bằng <code>rebuild_boundary_test.dart</code>. Chỉ cách nghe riêng <code>ValueNotifier</code> của cước thoát khỏi hai ô "oan".</figcaption>
</figure>

Ba điều cần nói thẳng. Một, cách 2 không miễn phí: cước nằm ở hai nơi (km và `ValueNotifier` suy ra từ km) và ta tự đồng bộ trong `setKm`. Hai, `read` không theo dõi danh tính notifier: đổi sang draft khác thì widget dùng `of` thấy cước mới, widget dùng `read` vẫn nghe draft cũ (test kiểm). Ba, trong app lab, chip dịch vụ và thanh trượt dùng `of`, nên kéo thanh trượt cũng dựng lại chip. Ba chip thì đủ rẻ; ta chỉ tối ưu khi đo được vấn đề.

Phép đo ở mức app dùng `debugOnRebuildDirtyWidget`, callback Flutter gọi cho mỗi widget được dựng lại (chỉ ở chế độ debug):

### app_test.dart · 17-state-changenotifier/test/app_test.dart (trích)

```dart
Future<Set<String>> _rebuiltBy(WidgetTester tester, VoidCallback action) async {
  final rebuilt = <String>{};
  debugOnRebuildDirtyWidget = (element, _) =>
      rebuilt.add(element.widget.runtimeType.toString());
  try {
    action();
    await tester.pump();
  } finally {
    debugOnRebuildDirtyWidget = null;
  }
  return rebuilt;
}
```

Kéo từ 5 sang 8 km thì `_KmSlider`, `_ServiceChips` và `ValueListenableBuilder<Money>` bị dựng lại, còn `HomeShell`, `QuoteScreen`, `_BookButton`, `HistoryScreen` thì không. Đổi dịch vụ không dựng lại nhãn cước. Đặt chuyến mới dựng lại `HistoryScreen` mà không đụng chip hay thanh trượt.

## Dựng lại bài 16: sổ chuyến không còn callback

Bài toán xuất phát giờ giải được bằng một `ChangeNotifier` thứ hai. `TripBook` giữ danh sách chuyến và hai thao tác `book`, `cancel`; `TripBookScope` cùng khuôn với `QuoteScope`:

### trip_book.dart · 17-state-changenotifier/lib/trip_book.dart (trích)

```dart
  void cancel(String id) {
    final trip = byId(id);
    if (trip == null) {
      throw ArgumentError.value(id, 'id', 'không có chuyến này trong sổ');
    }
    final cancelled = trip.apply(const Cancelled(by: CancelledBy.customer));
    _trips = List.unmodifiable([
      for (final t in _trips) t.isSameTrip(cancelled) ? cancelled : t,
    ]);
    notifyListeners();
  }
```

`Trip.apply` của `toi_noi_core` vẫn quyết định huỷ có hợp lệ không: huỷ chuyến đã hoàn thành ném `StateError` và sổ không báo gì. Mỗi lần đổi, sổ thay cả danh sách bằng một danh sách chỉ đọc mới thay vì sửa tại chỗ. Phía màn hình, mọi callback biến mất:

### trip_detail_page.dart · 17-state-changenotifier/lib/trip_detail_page.dart (trích)

```dart
    final trip = TripBookScope.of(context).byId(tripId);
    // ...
            OutlinedButton(
              onPressed: () => TripBookScope.read(context).cancel(trip.id),
              child: const Text('Huỷ chuyến'),
            ),
```

Trang chi tiết chỉ nhận `tripId`, đọc chuyến bằng `of` nên tự dựng lại sau khi huỷ; nút huỷ chỉ gọi hành động nên dùng `read`. Test đi qua đúng luồng người dùng:

### app_test.dart · 17-state-changenotifier/test/app_test.dart (trích)

```dart
      await tester.tap(find.text('Huỷ chuyến'));
      await tester.pump();

      // Trang chi tiết dựng lại vì nó phụ thuộc TripBookScope.
      expect(find.text('Huỷ chuyến'), findsNothing);
      expect(find.text('Đã huỷ'), findsOneWidget);

      // Quay lại: danh sách phía sau đã thấy trạng thái mới.
      await tester.pageBack();
      await tester.pumpAndSettle();
```

Tab đang mở vẫn là `setState` trong `HomeShell` vì nó ephemeral. Đổi tab rồi quay lại, bản nháp báo giá không mất dù `QuoteScreen` là `StatelessWidget`: nó sống ở `QuoteDraft`.

## Vì sao cần thư viện, và vì sao lộ trình chọn Riverpod

`QuoteScope` và `TripBookScope` gần giống hệt nhau: mỗi notifier mới cần thêm một scope, một cặp `of`/`read`, một thông báo lỗi, và các scope lồng nhau thành tầng. Chỗ yếu của cách viết tay còn có:

- Thiếu scope chỉ lộ **lúc chạy** bằng `FlutterError`, không lộ lúc biên dịch.
- Muốn nghe một phần state, ta tự dựng `ValueNotifier` rồi tự đồng bộ như `fare`.
- Không có khái niệm loading, lỗi, thử lại cho dữ liệu bất đồng bộ. Bài 19 và 20 cần đúng thứ này.
- Tra cứu qua scope cần `BuildContext`; dùng ngoài cây (ví dụ interceptor gọi mạng ở bài 24) ta phải tự chuyền instance.

Tài liệu Flutter nói lựa chọn phụ thuộc độ phức tạp của app và thói quen của đội. Bảng dưới so sánh ba thư viện hay gặp theo tài liệu của từng thư viện; số phiên bản lấy từ pub.dev tháng 11/2026, hãy kiểm lại khi đọc.

| | provider | bloc (`flutter_bloc`) | Riverpod (`flutter_riverpod`) |
|---|---|---|---|
| Ý tưởng | Bọc `InheritedWidget` cho đỡ boilerplate: cấp phát và huỷ tài nguyên, tải lười, `Consumer` | `Cubit` gọi hàm rồi `emit` state; `Bloc` nhận event; widget dùng `BlocBuilder` | Khai báo provider, state nằm trong container ngoài cây widget, đọc bằng `ref.watch` |
| Nghe một phần | `Selector` | `buildWhen`, `BlocSelector` | `select` |
| Bất đồng bộ | `FutureProvider`, `StreamProvider` | tự mô hình hoá loading, success, failure | `AsyncValue` có sẵn |
| Điểm mạnh | Gần `ChangeNotifier` nhất, tài liệu Flutter dùng làm ví dụ | Luồng event rõ ràng, hợp đội muốn quy ước chặt; `Cubit`/`Bloc` là lớp Dart, package `bloc` là Dart thuần nên test không cần cây widget | Provider đọc provider khác qua `ref`, không cần `BuildContext`; chạy được trong Dart thuần |
| Đánh đổi | Vẫn gắn với cây widget | Nhiều file hơn cho state đơn giản | Khái niệm mới; API đổi giữa các bản chính |
| Bản mới nhất | 6.1.5+1 | 9.1.1 | 3.4.3 |

`flutter_bloc` phụ thuộc vào chính `provider`, và tài liệu Flutter ghi `InheritedWidget` là thứ `provider` cùng nhiều cách tiếp cận khác dùng bên dưới. Bảng không có test vì lab không dùng ba thư viện này.

Lộ trình chọn Riverpod làm trục vì bốn việc sắp tới trúng điểm mạnh của nó: luồng đặt chuyến bất đồng bộ cần `AsyncValue` (bài 18 đến 20); test ở Phần 4 cần ghi đè provider mà không dựng cây widget (bài 30); cấu hình theo môi trường cần thay phần cài đặt theo flavor (bài 25); provider đọc provider khác qua `ref`, hợp với use case và realtime nằm ngoài cây widget (bloc cũng làm được việc này). Cái giá là học khái niệm mới và theo dõi thay đổi giữa các bản chính, nên lab ghim phiên bản.

Đây là quyết định của lộ trình, không phải chân lý. Đội đã dùng bloc ổn định thì cứ dùng: chỉ bài 18 và Phần 6 cần viết lại.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi tự quản lý state bằng InheritedWidget và ChangeNotifier</span>
<ul>
<li><strong>Đặt scope quá thấp.</strong> Dấu hiệu: <code>FlutterError</code> "Không tìm thấy QuoteScope" khi mở trang mới. Nguyên nhân: scope nằm trong <code>home</code>, còn route được đẩy nằm dưới <code>MaterialApp</code>. Cách sửa: đặt scope trên <code>MaterialApp</code>. Gọi <code>of</code> bằng chính <code>context</code> đã dựng ra scope cũng lỗi như vậy; dùng <code>Builder</code>.</li>
<li><strong>Gọi <code>of</code> trong <code>initState</code>.</strong> Dấu hiệu: debug báo "was called before ... initState() completed". Nguyên nhân: <code>initState</code> chỉ chạy một lần nên giá trị lấy ở đó sẽ cũ. Cách sửa: lấy trong <code>build</code> hoặc <code>didChangeDependencies</code>.</li>
<li><strong>Dùng <code>read</code> để hiển thị dữ liệu.</strong> Dấu hiệu: nhãn đứng yên dù state đã đổi (test: "ride-read" không đổi khi dịch vụ thành food). Nguyên nhân: <code>read</code> không đăng ký phụ thuộc. Cách sửa: dùng <code>of</code>, hoặc nghe một <code>ValueListenable</code> như <code>FareFromValue</code>; để <code>read</code> cho trình xử lý sự kiện.</li>
<li><strong>Tạo notifier trong <code>build</code>.</strong> Dấu hiệu: chọn "Giao đồ ăn" xong lại tự về "Đặt xe" khi widget cha dựng lại. Nguyên nhân: mỗi lần <code>build</code> tạo một <code>QuoteDraft</code> mới và không huỷ cái cũ. Cách sửa: tạo trong <code>State</code>, huỷ trong <code>dispose</code>, như <code>ToiNoiApp</code>.</li>
<li><strong>Sửa tại chỗ giá trị trong <code>ValueNotifier</code>.</strong> Dấu hiệu: <code>trips.value.add(trip)</code> mà giao diện không đổi. Nguyên nhân: <code>ValueNotifier</code> chỉ báo khi gán giá trị khác theo <code>==</code>, còn <code>add</code> giữ nguyên một <code>List</code>. Cách sửa: gán bản sao, <code>value = [...value, trip]</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/17-state-changenotifier
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` báo 0 issue, 43 test xanh. Mức: **Kiểm chứng tự động**, không cần thiết bị. Mỗi file test tập trung vào một mảng:

| File test | Chứng minh điều gì |
|---|---|
| `quote_draft_test.dart` (8) | Một thông báo mỗi thay đổi; cước 12.005 ở 2,001 km, 453.000 ở 100 km; km ngoài miền và NaN ném `ArgumentError`, trạng thái không đổi; đổi km trong 2 km đầu thì cước không báo; `dispose` |
| `trip_book_test.dart` (7) | Đặt chuyến, huỷ ở `searching`, huỷ chuyến hoàn thành ném `StateError`, danh sách chỉ đọc |
| `snapshot_scope_test.dart` (5) | `updateShouldNotify` đúng, luôn `true`, chỉ so cước; host bỏ nghe khi gỡ khỏi cây |
| `quote_scope_test.dart` (9) | `of`, `maybeOf`, `read`; gộp thông báo; đổi draft; vị trí scope; `of` trong `initState`; notifier tạo trong `build` |
| `rebuild_boundary_test.dart` (6) | Ma trận ở Hình 3 và `child` của `ValueListenableBuilder` |
| `app_test.dart` (8) | Luồng chọn, đặt, huỷ không callback; tập widget bị dựng lại khi kéo km, đổi dịch vụ, đặt chuyến |

Thử đổi tạm `read` thành `dependOnInheritedWidgetOfExactType` thì 8 test đỏ, gồm hai ca của ma trận; lab đã trả về như cũ.

**Chưa làm**: chạy trên simulator, emulator hay máy thật; đo hiệu năng thật (số lần build ở trên đo trong chế độ debug, DevTools thuộc bài 41); `InheritedModel`; provider, bloc, Riverpod (chỉ so sánh qua tài liệu); lưu cục bộ, gọi mạng, xử lý lỗi bất đồng bộ (bài 19, 20, 22); accessibility và responsive của bài 16, vì màn hình lab này là bản rút gọn.

## Bài tập

1. Một nhãn "Đặt xe · 25.500 VND" cần cả dịch vụ lẫn cước. Chọn cách đọc nào, và có dựng lại oan không?
2. Một bạn viết `QuoteScope(draft: QuoteDraft(), child: ...)` ngay trong `build` của widget cha. Triệu chứng là gì, sửa thế nào?
3. Ta đổi `updateShouldNotify` của `QuoteSnapshotScope` thành `snapshot.fare != oldWidget.snapshot.fare`. Khách đổi dịch vụ, cước giữ nguyên. Chip dịch vụ hiện gì, vì sao?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Dùng <code>of</code> hoặc <code>ListenableBuilder(draft)</code>, vì <code>fareListenable</code> chỉ có cước. Nhãn dựng lại mỗi lần draft báo, kể cả khi đổi km mà cước vẫn 12.000: có dựng lại oan, chấp nhận được với một dòng chữ. Muốn chặn hẳn cần "chọn" cặp (dịch vụ, cước), việc Riverpod làm bằng <code>select</code>.</p>
<p><strong>2.</strong> Mỗi lần cha dựng lại là một draft mới, nên khách chọn "Giao đồ ăn" xong state tự về "Đặt xe", và draft cũ không được huỷ. Sửa: tạo draft trong <code>State</code> của widget sống lâu, huỷ trong <code>dispose</code>.</p>
<p><strong>3.</strong> Chip vẫn hiện dịch vụ cũ: <code>updateShouldNotify</code> trả <code>false</code> vì cước bằng nhau nên dependent không được đánh thức (test <code>chỉ so cước</code> kiểm đúng điều này). Sửa: so cả ảnh chụp, hoặc dùng <code>InheritedModel</code>.</p>
</details>

Bài trước: [Accessibility, responsive và adaptive: một giao diện cho iOS và Android](/articles/flutter-accessibility-responsive-adaptive-ios-android). Bài tiếp: [Riverpod: Provider, Notifier và AsyncNotifier cho luồng đặt chuyến](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen).

## Tài liệu gốc

- [Introduction to state management](https://docs.flutter.dev/data-and-backend/state-mgmt/intro), [Differentiate between ephemeral state and app state](https://docs.flutter.dev/data-and-backend/state-mgmt/ephemeral-vs-app) và [Approaches to state management](https://docs.flutter.dev/data-and-backend/state-mgmt/options) (docs.flutter.dev).
- [InheritedWidget](https://api.flutter.dev/flutter/widgets/InheritedWidget-class.html), [InheritedNotifier](https://api.flutter.dev/flutter/widgets/InheritedNotifier-class.html) và [InheritedModel](https://api.flutter.dev/flutter/widgets/InheritedModel-class.html) (api.flutter.dev).
- [BuildContext.dependOnInheritedWidgetOfExactType](https://api.flutter.dev/flutter/widgets/BuildContext/dependOnInheritedWidgetOfExactType.html) và [BuildContext.getInheritedWidgetOfExactType](https://api.flutter.dev/flutter/widgets/BuildContext/getInheritedWidgetOfExactType.html) (api.flutter.dev).
- [ChangeNotifier](https://api.flutter.dev/flutter/foundation/ChangeNotifier-class.html), [ValueNotifier](https://api.flutter.dev/flutter/foundation/ValueNotifier-class.html), [ListenableBuilder](https://api.flutter.dev/flutter/widgets/ListenableBuilder-class.html) và [ValueListenableBuilder](https://api.flutter.dev/flutter/widgets/ValueListenableBuilder-class.html) (api.flutter.dev).
- [Constructors, private named parameters](https://dart.dev/language/constructors) và [Language evolution](https://dart.dev/resources/language/evolution) (dart.dev).
- [provider](https://pub.dev/packages/provider), [flutter_bloc](https://pub.dev/packages/flutter_bloc) và [flutter_riverpod](https://pub.dev/packages/flutter_riverpod) (pub.dev); [Riverpod](https://riverpod.dev) (riverpod.dev) và [Bloc](https://bloclibrary.dev) (bloclibrary.dev).
