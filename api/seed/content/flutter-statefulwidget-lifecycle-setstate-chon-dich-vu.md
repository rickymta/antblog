---
slug: flutter-statefulwidget-lifecycle-setstate-chon-dich-vu
title: "StatefulWidget, lifecycle và setState: chọn dịch vụ và đếm ngược tìm tài xế"
summary: "Bài 10: tại sao widget bất biến mà màn hình vẫn đổi; vòng đời State (initState, didUpdateWidget, dispose), phạm vi rebuild của setState, Timer.periodic huỷ đúng chỗ, mounted sau await và khi nào cần GlobalKey. Có ServicePicker, SearchingCountdown và 37 test dùng pump."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, state, lifecycle, testing]
language: vi
---

Khách mở app [Tới Nơi](/articles/du-an-mau-toi-noi), chọn "Giao đồ ăn" rồi bấm "Tìm tài xế". Màn hình hiện dòng "Đang tìm tài xế, còn 44 giây" và số giây giảm mỗi giây. Nếu khách đổi sang "Giao bưu kiện" khi đang chờ, thời gian chờ đổi theo và đếm lại. Nếu khách bấm back, đồng hồ phải dừng hẳn, không được chạy ngầm sau lưng màn hình đã biến mất.

Dòng đếm ngược nhớ một con số thay đổi theo thời gian, trong khi `StatelessWidget` của các bài trước chỉ biết đầu vào. Chỗ để nhớ con số đó là **State**. Bài này dạy vòng đời của State và những việc phải dọn khi nó chết. Đây là bài 10 của lộ trình **Flutter qua Tới Nơi**. Bạn nên đọc [bài 09 về ListView và Sliver](/articles/flutter-listview-sliver-scroll-lich-su-chuyen) và [bài 07 về cây widget](/articles/flutter-widget-cay-widget-hot-reload-man-hinh-bao-gia) trước, và nhớ `Timer`, `Future` từ [bài 05](/articles/dart-future-async-await-stream-bao-gia-va-vi-tri). Thời gian chờ 30, 45, 60 giây là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một widget bật <code>Timer.periodic</code> mỗi giây gọi <code>setState</code>. Người dùng bấm back, widget bị gỡ khỏi cây nhưng ta quên huỷ timer. Điều gì xảy ra ở lần timer chạy kế tiếp?</p>
<details><summary>Xem đáp án</summary><p>Timer vẫn sống vì không ai huỷ nó, và nó gọi <code>setState</code> trên một <code>State</code> đã bị <code>dispose</code>. Ở chế độ debug và trong test, <code>setState</code> ném <code>FlutterError</code> "setState() called after dispose()"; kiểm tra này nằm trong <code>assert</code>, nên bản release không có thông báo rõ ràng như vậy. Timer còn giữ tham chiếu tới State nên bộ nhớ không được giải phóng. Trong <code>flutter test</code>, một Timer còn treo sau khi cây widget bị gỡ làm test đỏ.</p></details>
</div>

## Widget bất biến, State ở lại

Widget chỉ là bản **cấu hình** bất biến mà cha đưa xuống; mỗi lần cha `build`, ta nhận một đối tượng widget mới. Dữ liệu cần sống qua nhiều lần build nằm ở `State`, một đối tượng riêng mà `StatefulElement` giữ suốt thời gian widget còn ở vị trí đó trong cây (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 1. Qua ba lần widget cha build, cha đưa xuống ba đối tượng widget mới, nhưng một StatefulElement và một State duy nhất vẫn ở lại; State giữ _selected, ban đầu là ride, sau đó người dùng chọn food rồi parcel."><defs><marker id="fl10-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="345" y="20" text-anchor="middle" class="t-small">cha build lần 1</text><text x="555" y="20" text-anchor="middle" class="t-small">cha build lần 2</text><text x="765" y="20" text-anchor="middle" class="t-small">cha build lần 3</text>
<text x="20" y="54" class="t-b">Widget</text><text x="20" y="72" class="t-small">bất biến, tạo lại mỗi build</text>
<rect x="250" y="30" width="190" height="56" rx="10" class="f-phys-soft s-phys"/><text x="266" y="54" class="t-b">widget #1</text><text x="266" y="73" class="t-small">cấu hình, rồi bị bỏ</text>
<rect x="460" y="30" width="190" height="56" rx="10" class="f-phys-soft s-phys"/><text x="476" y="54" class="t-b">widget #2</text><text x="476" y="73" class="t-small">cấu hình, rồi bị bỏ</text>
<rect x="670" y="30" width="190" height="56" rx="10" class="f-phys-soft s-phys"/><text x="686" y="54" class="t-b">widget #3</text><text x="686" y="73" class="t-small">cấu hình hiện tại</text>
<path d="M345 88 L345 128" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-1-a)"/><text x="353" y="113" class="t-small">thay vào</text>
<path d="M555 88 L555 128" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-1-a)"/><text x="563" y="113" class="t-small">thay vào</text>
<path d="M765 88 L765 128" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-1-a)"/><text x="773" y="113" class="t-small">thay vào</text>
<text x="20" y="154" class="t-b">Element</text><text x="20" y="172" class="t-small">bền theo vị trí trong cây</text>
<rect x="250" y="130" width="610" height="56" rx="10" class="f-pk-soft s-pk"/><text x="266" y="154" class="t-b">StatefulElement (một đối tượng duy nhất)</text><text x="266" y="174" class="t-small">trỏ tới widget hiện tại và giữ State</text>
<path d="M345 188 L345 228" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-1-a)"/><text x="353" y="213" class="t-small">giữ</text>
<path d="M555 188 L555 228" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-1-a)"/><text x="563" y="213" class="t-small">giữ</text>
<path d="M765 188 L765 228" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-1-a)"/><text x="773" y="213" class="t-small">giữ</text>
<text x="20" y="254" class="t-b">State</text><text x="20" y="272" class="t-small">bền, giữ dữ liệu thay đổi</text>
<rect x="250" y="230" width="610" height="76" rx="10" class="f-pk-soft s-pk"/><text x="266" y="254" class="t-b">_ServicePickerState (initState chạy một lần)</text>
<text x="266" y="286" class="t-mono">_selected = ride</text><text x="476" y="286" class="t-mono">_selected = food</text><text x="686" y="286" class="t-mono">_selected = parcel</text>
</svg>
</div>
<figcaption>Hình 1. Mỗi lần widget cha build, cha đưa xuống một widget mới; Element và State vẫn là một đối tượng duy nhất. Khi người dùng chọn dịch vụ, State tự gọi setState và đổi _selected mà không cần widget mới. Dữ liệu cần sống lâu nằm ở State.</figcaption>
</figure>

`ServicePicker` là ví dụ nhỏ nhất. Cha cho giá trị ban đầu qua `initial` và nghe thay đổi qua `onChanged`; lựa chọn hiện tại là field của State.

### service_picker.dart · 10-stateful-lifecycle/lib/service_picker.dart (trích)

```dart
class _ServicePickerState extends State<ServicePicker> {
  late ServiceKind _selected;

  @override
  void initState() {
    super.initState();
    _selected = widget.initial;
  }

  void _select(Set<ServiceKind> selection) {
    final kind = selection.first;
    setState(() => _selected = kind);
    widget.onChanged?.call(kind);
  }

  @override
  Widget build(BuildContext context) {
    final summaryBuilder = widget.summaryBuilder;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SegmentedButton<ServiceKind>(
          segments: [
            for (final kind in ServiceKind.values)
              ButtonSegment<ServiceKind>(value: kind, label: Text(kind.label)),
          ],
          selected: {_selected},
          onSelectionChanged: _select,
        ),
        const SizedBox(height: 12),
        if (summaryBuilder != null)
          summaryBuilder(context, _selected)
        else
          Text('Dịch vụ: ${_selected.label}'),
      ],
    );
  }
}
```

Ba điểm cần để ý. `late ServiceKind _selected` chưa có giá trị lúc khai báo vì phải đọc `widget.initial` ở `initState`. `widget` trong State luôn là widget **hiện tại**, nên khi cha đưa widget mới xuống, `widget.onChanged` tự trỏ sang hàm mới. Và `initial` chỉ được đọc một lần: cha đổi `initial` về sau thì lựa chọn đã có không đổi, giống `initialValue` của `TextFormField`; test `đổi initial ở cha không đổi lựa chọn đã có` chứng minh điều này.

`SegmentedButton<ServiceKind>` là widget Material 3 để chọn một trong vài phương án. `emptySelectionAllowed` mặc định `false` và `multiSelectionEnabled` mặc định `false` (đã đọc source), nên luôn có đúng một lựa chọn và `selection.first` an toàn. Chạm lại đoạn đang chọn thì `SegmentedButton` không gọi `onSelectionChanged`, nên `_select` không cần tự chặn. Nhãn lấy từ `ServiceKind.label` của `toi_noi_core`. Dữ liệu do cha quyết định, như danh sách chuyến, thì giữ ở cha; State chỉ dành cho thứ mà chính widget này sở hữu.

## Vòng đời của State

Framework gọi các phương thức của State theo thứ tự cố định (Hình 2). Ta chỉ ghi đè chỗ nào có việc thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 2. Vòng đời State: createState, initState, didChangeDependencies, build. Sau đó build chạy lại khi setState được gọi hoặc khi cha đưa widget mới (didUpdateWidget chạy trước build). Khi widget bị gỡ khỏi cây: deactivate rồi dispose, sau đó mounted là false."><defs><marker id="fl10-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl10-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="40" y="36" width="170" height="50" rx="10" class="f-phys-soft s-phys"/><text x="52" y="58" class="t-mono t-b">createState</text><text x="52" y="76" class="t-small">tạo đối tượng State</text>
<rect x="250" y="36" width="170" height="50" rx="10" class="f-pk-soft s-pk"/><text x="262" y="58" class="t-mono t-b">initState</text><text x="262" y="76" class="t-small">chạy đúng một lần</text>
<rect x="460" y="36" width="170" height="50" rx="10" class="f-pk-soft s-pk"/><text x="472" y="58" class="t-mono t-b">didChangeDependencies</text><text x="472" y="76" class="t-small">ngay sau initState</text>
<rect x="670" y="36" width="170" height="50" rx="10" class="f-phys-soft s-phys"/><text x="682" y="58" class="t-mono t-b">build</text><text x="682" y="76" class="t-small">dựng giao diện</text>
<path d="M210 61 L248 61" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-2-a)"/>
<path d="M420 61 L458 61" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-2-a)"/>
<path d="M630 61 L668 61" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-2-a)"/>
<text x="250" y="140" class="t-b">build chạy lại khi:</text>
<rect x="470" y="150" width="150" height="50" rx="10" class="f-pk-soft s-pk"/><text x="482" y="172" class="t-mono t-b">setState</text><text x="482" y="190" class="t-small">code của ta gọi</text>
<rect x="680" y="150" width="170" height="50" rx="10" class="f-pk-soft s-pk"/><text x="692" y="172" class="t-mono t-b">didUpdateWidget</text><text x="692" y="190" class="t-small">cha đưa widget mới</text>
<path d="M545 150 L545 120 L720 120 L720 88" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-2-a)"/>
<path d="M800 150 L800 88" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-2-a)"/>
<text x="40" y="240" class="t-small">khi widget bị gỡ khỏi cây:</text>
<rect x="40" y="250" width="170" height="50" rx="10" class="f-phys-soft s-phys"/><text x="52" y="272" class="t-mono t-b">deactivate</text><text x="52" y="290" class="t-small">rời cây</text>
<rect x="250" y="250" width="170" height="50" rx="10" class="f-warn-soft s-warn"/><text x="262" y="272" class="t-mono t-b t-warn">dispose</text><text x="262" y="290" class="t-small">dọn dẹp, State chết</text>
<path d="M210 275 L248 275" class="f-none s-warn" stroke-width="2" marker-end="url(#fl10-2-w)"/>
<text x="450" y="268" class="t-small">dispose: huỷ Timer, controller, subscription</text>
<text x="450" y="290" class="t-small t-warn">sau đó mounted == false: không được setState nữa</text>
</svg>
</div>
<figcaption>Hình 2. Thứ tự gọi của State. Mũi tên vào build cho thấy hai nguồn hay gặp khiến build chạy lại: setState từ code của ta, và didUpdateWidget khi widget cha đưa cấu hình mới xuống.</figcaption>
</figure>

Ta đã đọc source framework để chắc thứ tự: `initState` rồi `didChangeDependencies` chạy trước lần `build` đầu; `didUpdateWidget` luôn được theo sau bởi `build`; `dispose` đến sau `deactivate`. Nhiệm vụ từng phương thức:

- **`initState`**: khởi tạo state từ `widget`, bật timer hay subscription. Phải đồng bộ: khai báo `async` thì debug ném lỗi.
- **`didUpdateWidget(oldWidget)`**: so `oldWidget` với `widget` để phản ứng khi tham số đổi. Không cần `setState` vì `build` chạy ngay sau.
- **`dispose`**: trả lại mọi thứ đã mượn. Tài liệu framework lưu ý nó không được gọi khi app tắt, nên đừng đặt việc sống còn ở đây.
- **`didChangeDependencies`**: chạy khi một `InheritedWidget` mà State đã đọc thay đổi (như `Theme`). Ta chưa cần ghi đè nó.

`SearchingCountdown` dùng ba phương thức đầu:

### searching_countdown.dart · 10-stateful-lifecycle/lib/searching_countdown.dart (trích)

```dart
  @override
  void initState() {
    super.initState();
    _remaining = widget.seconds;
    _startTimer();
  }

  @override
  void didUpdateWidget(SearchingCountdown oldWidget) {
    super.didUpdateWidget(oldWidget);
    // Không cần setState: framework luôn gọi build ngay sau đây.
    if (oldWidget.seconds != widget.seconds) {
      _remaining = widget.seconds;
      _cancelled = false;
      _cancelFailed = false;
      _startTimer();
    }
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  void _startTimer() {
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(seconds: 1), (_) => _tick());
  }

  void _tick() {
    setState(() {
      _remaining--;
      _cancelFailed = false;
    });
    if (_remaining == 0) {
      _timer?.cancel();
      widget.onTimeout?.call();
    }
```

`didUpdateWidget` chỉ làm gì khi `seconds` thực sự đổi: đặt lại số giây rồi bật timer mới. `_startTimer` luôn `cancel` timer cũ trước, nếu không hai timer cùng đếm và số giây giảm hai lần mỗi giây. Khi về 0, `_tick` tự huỷ timer và gọi `onTimeout` một lần.

## setState và phạm vi rebuild

`setState(fn)` chạy `fn` đồng bộ rồi đánh dấu Element của State này là "dirty". Ở khung hình kế tiếp, framework gọi lại `build` của chính State đó. Hình 3 vẽ đúng chuyện xảy ra mỗi giây khi `SearchingCountdown` gọi `setState`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Cây widget của FindDriverScreen. Khi SearchingCountdown gọi setState, build của nó chạy lại và các widget con Text, LinearProgressIndicator, TextButton được dựng lại; FindDriverScreen, ServicePicker và SizedBox const không chạy lại."><defs><marker id="fl10-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="330" y="16" width="220" height="40" rx="8" class="f-surface s-muted"/><text x="440" y="41" text-anchor="middle" class="t-b">FindDriverScreen</text><text x="566" y="40" class="t-small">cha cũng không dựng lại</text>
<path d="M440 56 L440 86" class="f-none s-muted" stroke-width="1.6"/>
<rect x="380" y="86" width="120" height="36" rx="8" class="f-surface s-muted"/><text x="440" y="109" text-anchor="middle" class="t-mono">Column</text>
<path d="M440 122 L440 140 L150 140 L150 160" class="f-none s-muted" stroke-width="1.6"/>
<path d="M440 140 L560 140 L560 160" class="f-none s-muted" stroke-width="1.6"/>
<rect x="50" y="160" width="200" height="40" rx="8" class="f-surface s-muted"/><text x="150" y="185" text-anchor="middle" class="t-b">ServicePicker</text><text x="150" y="220" text-anchor="middle" class="t-small">State riêng, không bị chạm</text>
<rect x="430" y="160" width="260" height="40" rx="8" class="f-pk-soft s-pk"/><text x="560" y="185" text-anchor="middle" class="t-b">SearchingCountdown</text>
<text x="730" y="176" class="t-b t-pk">setState()</text><path d="M726 180 L692 180" class="f-none s-pk" stroke-width="2" marker-end="url(#fl10-3-a)"/>
<path d="M560 200 L560 224" class="f-none s-muted" stroke-width="1.6"/>
<path d="M295 224 L745 224" class="f-none s-muted" stroke-width="1.6"/>
<path d="M295 224 L295 240 M445 224 L445 240 M605 224 L605 240 M745 224 L745 240" class="f-none s-muted" stroke-width="1.6"/>
<rect x="250" y="240" width="90" height="40" rx="8" class="f-pk-soft s-pk"/><text x="295" y="265" text-anchor="middle" class="t-mono">Text</text>
<rect x="350" y="240" width="190" height="40" rx="8" class="f-pk-soft s-pk"/><text x="445" y="265" text-anchor="middle" class="t-mono">LinearProgressIndicator</text>
<rect x="550" y="240" width="110" height="40" rx="8" class="f-pk-soft s-pk"/><text x="605" y="265" text-anchor="middle" class="t-mono">TextButton</text>
<rect x="670" y="240" width="150" height="40" rx="8" class="f-surface s-muted"/><text x="745" y="265" text-anchor="middle" class="t-mono">const SizedBox</text>
<rect x="250" y="298" width="22" height="16" rx="3" class="f-pk-soft s-pk"/><text x="280" y="311" class="t-small">build chạy lại</text>
<rect x="420" y="298" width="22" height="16" rx="3" class="f-surface s-muted"/><text x="450" y="311" class="t-small">không chạy lại</text>
</svg>
</div>
<figcaption>Hình 3. Phạm vi rebuild của một setState. Mũi tên chỉ State phát lệnh; chỉ cây con của chính State đó dựng lại, và widget const hoặc cùng một instance thì bị bỏ qua.</figcaption>
</figure>

Hai quy tắc rút ra từ hình. Một: `setState` chỉ dựng lại cây con của State gọi nó, cha và anh em không bị đụng tới. Hai: trong cây con đó, widget `const` (hoặc đúng đối tượng cũ) bị bỏ qua vì `updateChild` thấy widget mới bằng widget cũ, còn widget tạo mới mỗi lần `build`, như `Text` của dòng đếm, được cập nhật.

Test của lab đếm số lần `build` bằng một widget `Probe` ghi vào bộ đếm:

### service_picker_test.dart · 10-stateful-lifecycle/test/service_picker_test.dart (trích)

```dart
      testWidgets('setState chỉ dựng lại cây con của chính state đó', (
        tester,
      ) async {
        await tester.pumpWidget(
          _app(
            Column(
              children: [
                ServicePicker(
                  summaryBuilder: (context, selected) {
                    _builds['summary'] = (_builds['summary'] ?? 0) + 1;
                    return Text(selected.name);
                  },
                ),
                const Probe('anh em'),
              ],
            ),
          ),
        );
        expect(_builds, {'summary': 1, 'anh em': 1});

        await tester.tap(find.text('Giao đồ ăn'));
        await tester.pump();

        // build của state chạy lại; widget anh em và cha không bị đụng tới.
        expect(_builds, {'summary': 2, 'anh em': 1});
      });
```

Sau khi chạm "Giao đồ ăn", `summaryBuilder` (nằm trong `build` của State) chạy hai lần, còn `Probe` anh em vẫn chỉ một. Vì thế đặt `setState` **càng thấp càng tốt**: đồng hồ nằm riêng trong `SearchingCountdown`, nên mỗi giây chỉ một nhánh nhỏ dựng lại, không phải cả `FindDriverScreen`.

Hai lưu ý nhỏ. Sửa field mà không bọc trong `setState` thì giá trị đổi nhưng giao diện chưa vẽ lại. Truyền cho `setState` một closure `async` bị debug chặn với "setState() callback argument returned a Future" (ta đã thử): việc bất đồng bộ làm trước, rồi mới `setState` đồng bộ với kết quả.

## Timer, dispose và test bằng pump

`Timer.periodic(Duration(seconds: 1), ...)` gọi callback sau mỗi giây, lần đầu **sau một giây** chứ không ngay lập tức. Timer sống ngoài cây widget: không ai huỷ thì nó chạy mãi. Vì vậy mọi `Timer`, `StreamSubscription` hay controller tạo ở `initState` phải được đóng ở `dispose`.

Test không chờ giây thật: `testWidgets` chạy trong đồng hồ giả, và `tester.pump(Duration)` đẩy đồng hồ tiến lên rồi dựng một khung hình.

### searching_countdown_test.dart · 10-stateful-lifecycle/test/searching_countdown_test.dart (trích)

```dart
    testWidgets('chưa đủ một giây thì chưa trừ', (tester) async {
      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 30)));

      await tester.pump(const Duration(milliseconds: 999));
      expect(_remaining(30), findsOneWidget);

      await tester.pump(const Duration(milliseconds: 1));
      expect(_remaining(29), findsOneWidget);
    });
```

999 mili giây chưa đủ để timer chạy, thêm 1 mili giây thì đủ. Các test khác kiểm: về 0 thì `onTimeout` gọi đúng một lần và đi tiếp một phút không có gì xảy ra thêm.

Với `didUpdateWidget`, test thay widget cùng vị trí bằng `seconds` khác:

```dart
    testWidgets('đổi seconds thì đếm lại từ giá trị mới, chỉ còn một timer', (
      tester,
    ) async {
      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 30)));
      await tester.pump(const Duration(seconds: 10));
      expect(_remaining(20), findsOneWidget);

      await tester.pumpWidget(_app(const SearchingCountdown(seconds: 60)));
      expect(_remaining(60), findsOneWidget);

      // Nếu timer cũ chưa bị huỷ, một giây trôi qua sẽ trừ hai lần (58).
      await tester.pump(const Duration(seconds: 1));
      expect(_remaining(59), findsOneWidget);
      await tester.pump(const Duration(seconds: 5));
      expect(_remaining(54), findsOneWidget);
    });
```

Dòng `expect(_remaining(59), ...)` là chỗ bắt lỗi quên huỷ timer cũ: nếu `_startTimer` bỏ dòng `_timer?.cancel()`, một giây trôi qua trừ hai lần và ra 58. Ta đã thử xoá dòng đó và hai test đỏ.

Còn `dispose`? Test dưới đây gỡ widget rồi cho đồng hồ đi một phút:

```dart
    testWidgets('gỡ widget khỏi cây thì Timer bị huỷ, không gọi gì nữa', (
      tester,
    ) async {
      var timeouts = 0;
      await tester.pumpWidget(
        _app(SearchingCountdown(seconds: 5, onTimeout: () => timeouts++)),
      );
      await tester.pump(const Duration(seconds: 2));

      await tester.pumpWidget(_app(const SizedBox()));
      await tester.pump(const Duration(minutes: 1));

      expect(timeouts, 0);
      expect(tester.takeException(), isNull);
      // Nếu dispose quên _timer?.cancel(), flutter_test sẽ làm test này đỏ
      // với "A Timer is still pending even after the widget tree was disposed".
    });
```

Không có `expect` nào kiểm timer: `flutter_test` làm việc đó, vì khi test kết thúc nó gỡ cây widget rồi kiểm còn Timer nào treo không. Ta đã thử xoá `_timer?.cancel()` trong `dispose`: 17 test đỏ với thông báo "A Timer is still pending even after the widget tree was disposed", kèm stack nơi timer được tạo. Lab còn có `_LeakyCountdown` (bản sai cố ý, chỉ nằm trong test): sau khi gỡ, `timer.isActive` vẫn `true` và `setState` ném `FlutterError`.

## mounted sau await

Timer huỷ được tại nguồn, còn `Future` đã `await` thì không rút lại được. Khách bấm "Huỷ tìm", API huỷ mất vài trăm mili giây, và trong lúc đó khách thoát màn hình: khi `await` trả về, State đã bị `dispose`. Hàm `_cancel` của lab xử lý như sau:

### searching_countdown.dart · 10-stateful-lifecycle/lib/searching_countdown.dart (trích)

```dart
  Future<void> _cancel() async {
    final onCancel = widget.onCancel;
    if (onCancel == null || _cancelling) return;
    setState(() {
      _cancelling = true;
      _cancelFailed = false;
    });
    try {
      await onCancel();
    } catch (_) {
      // Chi tiết lỗi và ghi log thuộc bài về xử lý lỗi; ở đây chỉ báo người
      // dùng và mở lại nút.
      if (!mounted) return;
      setState(() {
        _cancelling = false;
        _cancelFailed = true;
      });
      return;
    }
    // Trong lúc await, widget cha có thể đã gỡ widget này khỏi cây.
    if (!mounted) return;
    _timer?.cancel();
    setState(() {
      _cancelling = false;
      _cancelled = true;
    });
  }
```

`mounted` là `true` từ lúc State gắn vào cây tới lúc `dispose`. Quy tắc: **sau mỗi `await`, trước khi chạm `setState` hay `context`, kiểm `mounted`**. Trước `await` thì không cần vì code chạy đồng bộ ngay trong lúc State còn sống. Nhánh `catch` cũng phải kiểm `mounted`, vì lỗi cũng đến sau `await`.

Lint `use_build_context_synchronously` nằm trong `flutter_lints` 6.0.0 hỗ trợ việc này: ta đã thử đặt `Navigator.of(context).pop()` sau một `await` không kiểm thì lint báo, thêm `if (!mounted) return;` thì hết báo. Nhưng lint chỉ canh `context`; `setState` thiếu `mounted` thì lint im lặng, nên ta cần test:

```dart
    testWidgets('widget bị gỡ trong lúc await: mounted chặn setState', (
      tester,
    ) async {
      final completer = Completer<void>();
      await tester.pumpWidget(
        _app(SearchingCountdown(onCancel: () => completer.future)),
      );
      await tester.tap(find.text('Huỷ tìm'));
      await tester.pump();

      await tester.pumpWidget(_app(const SizedBox()));
      completer.complete();
      await tester.pump();

      expect(tester.takeException(), isNull);
    });

    testWidgets('bản thiếu mounted: setState sau await ném FlutterError', (
      tester,
    ) async {
      final completer = Completer<void>();
      await tester.pumpWidget(_app(_NoMountedCheck(completer.future)));
      final state = tester.state<_NoMountedCheckState>(
        find.byType(_NoMountedCheck),
      );

      final outcome = expectLater(
        state.run(),
        throwsA(
          isA<FlutterError>().having(
            (e) => e.toString(),
            'message',
            contains('setState() called after dispose()'),
          ),
        ),
      );
      await tester.pumpWidget(_app(const SizedBox()));
      completer.complete();
      await tester.pump();

      await outcome;
    });
```

Test thứ nhất gỡ widget khi `onCancel` còn treo (một `Completer` chưa hoàn thành), hoàn thành nó, và không có lỗi. Test thứ hai chạy cùng kịch bản trên `_NoMountedCheck`, bản thiếu `mounted`, và bắt được `FlutterError`. Ta gọi `state.run()` trực tiếp để lỗi trả về qua `Future`; nếu lỗi trôi ra ngoài `onPressed`, nó là lỗi bất đồng bộ không ai bắt và làm test đỏ.

`FindDriverScreen._cancelSearch` cũng kiểm `mounted` trước khi `setState`, vì chính màn hình có thể bị thoát trong lúc chờ. Test `thoát màn hình khi đang huỷ` chứng minh.

## Key và GlobalKey: khi nào cần

Với mỗi vị trí con, framework hỏi `Widget.canUpdate(cũ, mới)`: cùng `runtimeType` và cùng `key` thì **giữ Element và State cũ**, chỉ gọi `didUpdateWidget`; khác thì huỷ cái cũ và tạo mới. Phần lớn thời gian ta không cần key. Ba lựa chọn giải ba bài toán khác nhau:

| Cách | Khi nào dùng | Hệ quả |
|---|---|---|
| Không key | Mặc định | Khớp theo loại và vị trí; đổi loại cha thì State mới |
| `ValueKey(x)` | Danh sách có thứ tự đổi (bài 09); muốn **tạo lại** State bằng cách đổi key | Key đổi thì State cũ bị `dispose`, State mới `initState` |
| `GlobalKey` | Giữ State khi widget chuyển sang cha khác; hoặc truy cập `currentState`/`currentContext` từ ngoài (`Form` ở bài tiếp theo) | Phải duy nhất trong cả cây; hai widget cùng một GlobalKey là lỗi |

Test `keys_test.dart` đổi cha của `SearchingCountdown` giữa `Padding` và `Center`:

### keys_test.dart · 10-stateful-lifecycle/test/keys_test.dart (trích)

```dart
class _ReparentState extends State<_Reparent> {
  var _padded = true;

  @override
  Widget build(BuildContext context) {
    final countdown = SearchingCountdown(key: widget.countdownKey, seconds: 30);
    return Column(
      children: [
        TextButton(
          onPressed: () => setState(() => _padded = !_padded),
          child: const Text('đổi cha'),
        ),
        if (_padded)
          Padding(padding: const EdgeInsets.all(8), child: countdown)
        else
          Center(child: countdown),
      ],
    );
  }
}
```

```dart
    testWidgets('không key: đổi loại cha thì State mới, đếm lại từ đầu', (
      tester,
    ) async {
      await tester.pumpWidget(_app(const _Reparent(countdownKey: null)));
      await tester.pump(const Duration(seconds: 10));
      expect(_remaining(20), findsOneWidget);

      await tester.tap(find.text('đổi cha'));
      await tester.pump();

      expect(_remaining(30), findsOneWidget);
    });

    testWidgets('GlobalKey: chuyển sang cha khác vẫn giữ State đang đếm', (
      tester,
    ) async {
      final key = GlobalKey();
      await tester.pumpWidget(_app(_Reparent(countdownKey: key)));
      await tester.pump(const Duration(seconds: 10));
      expect(_remaining(20), findsOneWidget);

      await tester.tap(find.text('đổi cha'));
      await tester.pump();

      expect(_remaining(20), findsOneWidget);
      await tester.pump(const Duration(seconds: 1));
      expect(_remaining(19), findsOneWidget);
    });
```

Không key: đổi từ `Padding` sang `Center` là hai loại cha khác nhau, Element cũ không dùng lại được, và đồng hồ về 30. Có `GlobalKey`: framework gỡ Element ra rồi gắn vào chỗ mới, State đang đếm được giữ nguyên, đồng hồ tiếp tục ở 20 rồi 19.

Đừng với tới `GlobalKey` quá sớm. Đếm lại khi đổi tham số thì `didUpdateWidget` gọn hơn (như `FindDriverScreen` đổi dịch vụ). Đếm lại từ đầu bất kể tham số thì đổi `ValueKey`: test `đổi ValueKey` chứng minh State mới bắt đầu từ 30 dù `seconds` không đổi. Gọi `key.currentState` từ ngoài là ra lệnh lén cho widget con; tham số cộng `didUpdateWidget` dễ đọc và dễ thử hơn.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với StatefulWidget</span>
<ul>
<li><strong>Quên huỷ Timer ở <code>dispose</code>.</strong> Dấu hiệu: <code>flutter test</code> báo "A Timer is still pending even after the widget tree was disposed"; ngoài đời, đồng hồ chạy ngầm và <code>setState() called after dispose()</code>. Cách sửa: giữ <code>Timer?</code> trong field và <code>cancel()</code> ở <code>dispose</code>; trước khi tạo timer mới cũng huỷ timer cũ.</li>
<li><strong><code>setState</code> hay dùng <code>context</code> sau <code>await</code> mà không kiểm <code>mounted</code>.</strong> Dấu hiệu: "setState() called after dispose()" thỉnh thoảng mới ra, khi mạng chậm và người dùng thoát màn hình. Cách sửa: <code>if (!mounted) return;</code> ngay sau mỗi <code>await</code>, cả trong nhánh <code>catch</code>.</li>
<li><strong>Đọc <code>widget.x</code> trong <code>initState</code> rồi mong nó tự cập nhật.</strong> Dấu hiệu: cha đổi tham số nhưng giao diện như cũ. Cách sửa: ghi đè <code>didUpdateWidget</code> và so <code>oldWidget</code> với <code>widget</code>, hoặc đọc <code>widget.x</code> thẳng trong <code>build</code>.</li>
<li><strong>Sửa field không có <code>setState</code>, hoặc đưa việc <code>async</code> vào <code>setState</code>.</strong> Dấu hiệu: giá trị đổi mà màn hình đứng yên, hoặc lỗi "setState() callback argument returned a Future". Cách sửa: làm việc bất đồng bộ trước, rồi <code>setState(() =&gt; _x = kết quả)</code> đồng bộ.</li>
<li><strong>Đặt <code>setState</code> ở widget quá cao.</strong> Dấu hiệu: cả màn hình dựng lại mỗi giây, cuộn giật. Cách sửa: tách phần đổi liên tục thành widget con có State riêng và dùng <code>const</code> cho phần tĩnh.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/10-stateful-lifecycle
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` 0 issue, **37 test xanh**, không test nào chờ giây thật. Mức: **Kiểm chứng tự động**, không cần thiết bị.

| Test | Chứng minh điều gì |
|---|---|
| `service_picker_test.dart` (8) | Chọn dịch vụ, `initial`, `onChanged`; không tràn ở 320 px; phạm vi rebuild của `setState`; `const` không dựng lại |
| `searching_countdown_test.dart` (18) | Đếm đúng từng giây; `onTimeout` một lần; `didUpdateWidget` chỉ còn một timer; huỷ Timer ở `dispose`; `mounted` sau `await`; hai bản sai cố ý ném lỗi; thông báo lỗi huỷ tự mất khi đếm tiếp hoặc đổi `seconds` |
| `find_driver_screen_test.dart` (7) | Đổi dịch vụ khi đang tìm thì đếm lại 45 giây; State sống qua lúc cha dựng lại; thoát màn hình khi đang huỷ không lỗi |
| `keys_test.dart` (4) | Không key thì đổi cha reset đồng hồ; `GlobalKey` giữ State; đổi `ValueKey` tạo lại State |

Ta cũng thử bốn "đột biến" trên `SearchingCountdown`: xoá `cancel` ở `dispose`, bỏ `mounted` trong `_cancel`, bỏ `cancel` trong `_startTimer`, và vô hiệu `didUpdateWidget`. Mỗi lần đều có test đỏ.

**Chưa làm**: chạy trên iOS Simulator hay Android emulator (máy kiểm chưa có Android SDK và Xcode đầy đủ; README của lab ghi lệnh `flutter create` để sinh runner); đồng hồ đếm theo mốc thời điểm và việc app xuống nền rồi quay lại (cần `AppLifecycleListener`); phân loại lỗi `onCancel`; `AnimationController`, vốn cũng sống trong State (bài về animation).

## Bài tập

1. `ServicePicker` đang chọn "Giao đồ ăn". Cha dựng lại và đổi `initial` từ `ride` sang `parcel`. Picker hiện gì, và vì sao? Muốn nó theo `initial` mới thì sửa thế nào?
2. Cha muốn có nút "Tìm lại" đưa đồng hồ về `seconds` ban đầu mà không đổi `seconds`. `didUpdateWidget` hiện tại không bắt được. Nêu hai cách sửa và cái nào nên ưu tiên.
3. Một widget có `initState` gọi `Future.delayed(const Duration(seconds: 3), () => setState(...))`. Nêu hai cách sửa và viết ý chính của test chứng minh bằng `pump`.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Vẫn "Giao đồ ăn": <code>initial</code> chỉ được đọc ở <code>initState</code>, mà State được giữ qua lần dựng lại (Hình 1). Muốn theo cha: so <code>widget.initial</code> với <code>oldWidget.initial</code> trong <code>didUpdateWidget</code>, đổi <code>ValueKey</code> để tạo lại State, hoặc để cha giữ giá trị và bỏ State của picker.</p>
<p><strong>2.</strong> Cách một: cha đổi <code>ValueKey</code> (ví dụ số lần tìm) để tạo lại State; test <code>đổi ValueKey</code> chứng minh. Cách hai: thêm tham số <code>restartToken</code> và reset trong <code>didUpdateWidget</code> khi nó đổi. <code>GlobalKey</code> cộng <code>currentState?.restart()</code> cũng chạy nhưng khó đọc, khó thử, để sau cùng.</p>
<p><strong>3.</strong> Cách một (nên dùng): thay bằng <code>Timer</code> lưu trong field, huỷ ở <code>dispose</code>. Cách hai: giữ <code>Future.delayed</code> và kiểm <code>if (!mounted) return;</code> trước <code>setState</code>. Test: <code>pumpWidget</code> widget, <code>pumpWidget(const SizedBox())</code> để gỡ, <code>pump(const Duration(seconds: 5))</code>, rồi <code>expect(tester.takeException(), isNull)</code>.</p>
</details>

Bài trước: [ListView, Sliver và cuộn](/articles/flutter-listview-sliver-scroll-lich-su-chuyen). Bài tiếp: [Form, TextFormField và validation](/articles/flutter-form-validation-focus-nhap-diem-don-diem-den), nơi `GlobalKey<FormState>` xuất hiện đúng chỗ nó cần.

## Tài liệu gốc

- [State class](https://api.flutter.dev/flutter/widgets/State-class.html) và [StatefulWidget class](https://api.flutter.dev/flutter/widgets/StatefulWidget-class.html) (api.flutter.dev): vòng đời và hợp đồng của từng phương thức.
- [Adding interactivity to your Flutter app](https://docs.flutter.dev/ui/interactivity) (docs.flutter.dev).
- [Differentiate between ephemeral state and app state](https://docs.flutter.dev/data-and-backend/state-mgmt/ephemeral-vs-app) (docs.flutter.dev): khi nào state ở widget, khi nào đưa lên cha.
- [Key](https://api.flutter.dev/flutter/foundation/Key-class.html) và [GlobalKey](https://api.flutter.dev/flutter/widgets/GlobalKey-class.html) (api.flutter.dev).
- [SegmentedButton](https://api.flutter.dev/flutter/material/SegmentedButton-class.html) (api.flutter.dev).
- [Timer.periodic](https://api.dart.dev/stable/dart-async/Timer/Timer.periodic.html) (api.dart.dev).
- [WidgetTester.pump](https://api.flutter.dev/flutter/flutter_test/WidgetTester/pump.html) và [An introduction to widget testing](https://docs.flutter.dev/cookbook/testing/widget/introduction) (docs.flutter.dev).
- [use_build_context_synchronously](https://dart.dev/tools/linter-rules/use_build_context_synchronously) (dart.dev).
- [Performance best practices: control build cost](https://docs.flutter.dev/perf/best-practices) (docs.flutter.dev).
