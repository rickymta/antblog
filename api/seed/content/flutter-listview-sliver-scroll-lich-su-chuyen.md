---
slug: flutter-listview-sliver-scroll-lich-su-chuyen
title: "ListView, Sliver và cuộn: danh sách lịch sử chuyến"
summary: "Bài 09: dựng màn hình lịch sử chuyến của Tới Nơi với ListView.builder chỉ build phần tử trong vùng nhìn, CustomScrollView + SliverAppBar + SliverList, tải thêm cuối trang bằng ScrollController, kéo để làm mới, ValueKey cùng findItemIndexCallback giữ state khi đổi thứ tự."
category: programming
level: beginner
tags: [flutter, dart, mobile, toi-noi, listview, sliver, scroll, ios, android]
language: vi
---

Một khách quen của Tới Nơi dùng app gần như mỗi ngày: sáng đặt xe đi làm, trưa đặt cơm, chiều gửi bưu kiện. Sau một năm, tab "Lịch sử" có hàng trăm chuyến. Nếu app tải và vẽ hết trong một lượt, khách nhìn vòng quay rất lâu rồi cuộn trong một danh sách nặng nề trên máy tầm trung. Màn hình này cần làm điều ngược lại: chỉ vẽ vài dòng đang nhìn thấy, tải thêm khi gần hết, và cho kéo xuống để làm mới.

Bài này dựng `TripHistoryScreen` theo đúng yêu cầu đó, trên dữ liệu giả có phân trang, qua sáu cơ chế: `ListView.builder`, sliver, nguồn phân trang, tải thêm cuối trang, kéo để làm mới và key. Đây là bài 09 của lộ trình **Flutter qua Tới Nơi**. Bạn cần biết widget và constraints ở [bài 08](/articles/flutter-layout-constraints-row-column-the-chuyen), và `Future` ở [bài 05](/articles/dart-future-async-await-stream-bao-gia-va-vi-tri). Chuyến, `Money` và `calculateFare` đến từ package [toi_noi_core](/articles/dart-package-pubspec-lint-test-toi-noi-core). Bảng giá là số minh hoạ trong hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi), không phải bảng giá thực tế.

Một lưu ý về thứ tự học: ta chưa học `setState`. Màn hình phải nhớ danh sách đã tải và cờ đang chờ nguồn, mỗi dòng phải nhớ mình đang mở hay gập, nên có hai chỗ dùng `StatefulWidget`: `TripHistoryScreen` và `TripHistoryTile`. Bài này chỉ cần bạn biết `setState` báo cho framework rằng state vừa đổi và cần build lại. `State`, vòng đời và phạm vi rebuild được giải thích kỹ ở [bài 10](/articles/flutter-statefulwidget-lifecycle-setstate-chon-dich-vu). Mỗi chỗ dùng ở dưới đều được đánh dấu.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Danh sách có 500 chuyến, viết bằng <code>ListView.builder(itemCount: 500, ...)</code>. Màn hình cao 600 dp, mỗi dòng cao 72 dp. Ngay khi màn hình hiện lên, <code>itemBuilder</code> được gọi khoảng bao nhiêu lần: 9, 12 hay 500?</p>
<details><summary>Xem đáp án</summary><p>12 lần, với chỉ số từ 0 đến 11 (test ở cuối bài đo đúng con số này). Vùng nhìn chứa 8,3 dòng, và framework dựng thêm một <strong>vùng đệm</strong> phía dưới (và phía trên khi đã cuộn) để dòng mới đã sẵn sàng trước khi trượt vào màn hình. Vùng đệm mặc định là 250 dp, tức khoảng ba dòng nữa. 488 dòng còn lại chưa có widget, chưa có Element, chưa có RenderObject.</p></details>
</div>

## ListView.builder: chỉ tạo thứ đang cần

`ListView.builder` nhận `itemCount` và một hàm `itemBuilder(context, index)`. Framework gọi hàm đó khi cần một dòng, không gọi cho mọi dòng cùng lúc. "Cần" nghĩa là dòng nằm trong **viewport** (khung nhìn) hoặc trong **cache extent** quanh nó. Giá trị mặc định của cache extent là 250 dp (dp ở đây là logical pixel của Flutter; hằng `RenderAbstractViewport.defaultCacheExtent` trong source). Dòng lọt vào vùng này mới có widget, Element và RenderObject; dòng rời khỏi vùng bị bỏ đi (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 1. Danh sách 500 dòng trong một viewport. Chín dòng đầu nằm trong vùng nhìn, ba dòng kế nằm trong vùng đệm 250 dp, các dòng còn lại chưa được tạo.">
<text x="40" y="24" class="t-b">Danh sách 500 dòng, mỗi dòng cao 72 dp</text>
<rect x="40" y="40" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="54" class="t-mono">Dòng 0</text>
<rect x="40" y="62" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="76" class="t-mono">Dòng 1</text>
<rect x="40" y="84" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="98" class="t-mono">Dòng 2</text>
<rect x="40" y="106" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="120" class="t-mono">Dòng 3</text>
<rect x="40" y="128" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="142" class="t-mono">Dòng 4</text>
<rect x="40" y="150" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="164" class="t-mono">Dòng 5</text>
<rect x="40" y="172" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="186" class="t-mono">Dòng 6</text>
<rect x="40" y="194" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="208" class="t-mono">Dòng 7</text>
<rect x="40" y="216" width="260" height="20" rx="4" class="f-phys-soft s-phys"/><text x="52" y="230" class="t-mono">Dòng 8</text>
<rect x="40" y="238" width="260" height="20" rx="4" class="f-pk-soft s-pk"/><text x="52" y="252" class="t-mono">Dòng 9</text>
<rect x="40" y="260" width="260" height="20" rx="4" class="f-pk-soft s-pk"/><text x="52" y="274" class="t-mono">Dòng 10</text>
<rect x="40" y="282" width="260" height="20" rx="4" class="f-pk-soft s-pk"/><text x="52" y="296" class="t-mono">Dòng 11</text>
<path d="M312 40 L312 238" class="f-none s-phys" stroke-width="2"/><text x="324" y="132" class="t-b t-phys">Vùng nhìn</text><text x="324" y="150" class="t-small">600 dp, tức 8,3 dòng</text>
<path d="M312 240 L312 304" class="f-none s-pk" stroke-width="2"/><text x="324" y="268" class="t-b t-pk">Vùng đệm</text><text x="324" y="286" class="t-small">250 dp, tới dòng 11</text>
<rect x="40" y="310" width="260" height="52" rx="6" class="f-none s-muted" stroke-width="1.5" stroke-dasharray="6 5"/>
<text x="52" y="332" class="t-b">Dòng 12 đến 499</text><text x="52" y="350" class="t-small">chưa gọi itemBuilder</text>
<rect x="500" y="40" width="360" height="214" rx="10" class="f-surface s-line"/>
<text x="520" y="68" class="t-b">Đo bằng lazy_build_test.dart</text>
<text x="520" y="88" class="t-small">Màn hình 800 x 600 dp, dòng cao 72 dp</text>
<text x="520" y="128" class="t-b">Lần dựng đầu: build 12 lần</text>
<text x="520" y="148" class="t-small">(chỉ số 0 đến 11, không phải 500)</text>
<text x="520" y="188" class="t-b">Cuộn tay 5 lần, mỗi lần 500 dp:</text>
<text x="520" y="208" class="t-small">dưới 60 dòng từng được build, dòng 0</text>
<text x="520" y="228" class="t-small">bị bỏ khi ra khỏi vùng đệm</text>
</svg>
</div>
<figcaption>Hình 1. Viewport, vùng đệm và những dòng chưa được tạo trong ListView.builder 500 dòng. Con số đo ở test trên màn hình 800 x 600 dp.</figcaption>
</figure>

Đây là danh sách gọn nhất của lab, dùng `ListView.separated` để có đường kẻ giữa các dòng:

### recent_trips_list.dart · 09-listview-sliver/lib/recent_trips_list.dart

```dart
class RecentTripsList extends StatelessWidget {
  const RecentTripsList({super.key, required this.trips});

  final List<Trip> trips;

  @override
  Widget build(BuildContext context) {
    return ListView.separated(
      itemCount: trips.length,
      itemBuilder: (context, index) {
        final trip = trips[index];
        final completed = trip.status == TripStatus.completed;
        return ListTile(
          title: Text('${trip.serviceKind.label} · ${trip.id}'),
          // Chuyến huỷ không tính cước nên hiện trạng thái thay cho số tiền.
          trailing: Text(completed ? trip.fare.toString() : trip.status.label),
        );
      },
      separatorBuilder: (context, index) => const Divider(height: 1),
    );
  }
}
```

Ta không tin lời hứa "lười" mà đo nó. Test đặt vào danh sách một widget `_Probe` ghi lại chỉ số mỗi lần `build` được gọi:

### lazy_build_test.dart · 09-listview-sliver/test/lazy_build_test.dart (trích)

```dart
/// Phần tử đếm số lần `build` của chính nó, ghi lại chỉ số vào [log].
class _Probe extends StatelessWidget {
  const _Probe({required this.index, required this.log});

  final int index;
  final List<int> log;

  @override
  Widget build(BuildContext context) {
    log.add(index);
    return SizedBox(height: 72, child: Text('Dòng $index'));
  }
}
```

```dart
testWidgets('500 phần tử nhưng chỉ vài chục lần build', (tester) async {
  final builds = <int>[];
  await tester.pumpWidget(
    app(
      ListView.builder(
        itemCount: 500,
        itemBuilder: (context, index) => _Probe(index: index, log: builds),
      ),
    ),
  );

  // Màn hình test cao 600 dp, mỗi dòng 72 dp, vùng đệm 250 dp.
  expect(builds, [for (var i = 0; i < 12; i++) i]);
  expect(find.text('Dòng 0'), findsOneWidget);
  expect(find.text('Dòng 400', skipOffstage: false), findsNothing);
});
```

Còn `ListView(children: [...])`? Danh sách dựng sẵn chạy constructor của cả 500 widget ngay từ đầu (test đếm đúng 500), dù chỉ vùng nhìn được build. `builder` không tạo các widget thừa đó, và với nguồn dữ liệu lớn hay phân trang thì đây là khác biệt giữa chạy được và không.

Có một ngoại lệ đáng biết: nhảy xa một phát. Khi dòng có chiều cao chưa biết trước, `jumpTo` tới dòng 400 làm framework dựng lần lượt các dòng ở giữa để biết dòng 400 nằm ở pixel nào. Test đo được 412 lần build tổng cộng (12 lúc đầu cộng 400 trong cú nhảy), thay vì 12. Khi mọi dòng cùng cao và ta khai báo `itemExtent: 72`, `ListView` dùng `SliverFixedExtentList`, biết ngay vị trí mà không cần đo, và tổng số lần build chỉ còn 28 (16 sau cú nhảy). Cuộn tay không gặp tình huống này, nhưng `jumpTo` hay nút "về đầu danh sách" thì có.

## Từ ListView sang CustomScrollView và sliver

`ListView` và `CustomScrollView` đều là `ScrollView`. `ListView` tự xếp sẵn đúng một **sliver** danh sách: `SliverList`, hoặc `SliverFixedExtentList` khi có `itemExtent` (đọc ở `buildChildLayout`). `CustomScrollView` để ta tự xếp các sliver. Sliver là một mảnh có thể cuộn của viewport. Màn hình lịch sử cần nhiều loại mảnh cùng cuộn như một khối: app bar co lại khi kéo, dòng tóm tắt, danh sách, chân trang. Nếu xếp `ListView` trong `Column`, mỗi mảnh sẽ tự cuộn kiểu riêng, hoặc không cuộn được (xem mục lỗi thường gặp).

Cơ chế của sliver giống "constraints đi xuống, kích thước đi lên" ở bài 08, nhưng tính theo trục cuộn (Hình 2). Viewport gửi xuống mỗi sliver một `SliverConstraints`, trong đó có `scrollOffset` (mảnh này đã bị cuộn qua bao nhiêu) và `remainingPaintExtent` (còn bao nhiêu chỗ để vẽ). Sliver trả lên một `SliverGeometry` với `scrollExtent` (mảnh này dài bao nhiêu trong tổng chiều dài cuộn) và `paintExtent` (hiện được bao nhiêu). Nhờ mỗi sliver tự quyết phần nào của nó cần dựng, `SliverList` mới lười được.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 420" role="img" aria-label="Hình 2. Màn hình lịch sử gồm bốn sliver xếp trong một viewport: SliverAppBar, SliverToBoxAdapter tóm tắt, SliverList.separated và SliverToBoxAdapter chân trang. Viewport gửi SliverConstraints xuống, sliver trả SliverGeometry lên."><defs><marker id="fl09-2-d" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl09-2-u" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="20" width="420" height="360" rx="10" class="f-surface s-line"/>
<text x="36" y="46" class="t-b">CustomScrollView (một Viewport)</text>
<rect x="40" y="64" width="380" height="58" rx="6" class="f-pk-soft s-pk"/><text x="54" y="88" class="t-b">SliverAppBar</text><text x="54" y="108" class="t-small">pinned: true, expandedHeight: 120</text>
<rect x="40" y="130" width="380" height="44" rx="6" class="f-phys-soft s-phys"/><text x="54" y="150" class="t-b">SliverToBoxAdapter</text><text x="54" y="167" class="t-small">dòng tóm tắt, một box widget thường</text>
<rect x="40" y="182" width="380" height="124" rx="6" class="f-pk-soft s-pk"/><text x="54" y="206" class="t-b">SliverList.separated</text><text x="54" y="226" class="t-small">itemBuilder chỉ chạy cho dòng đang cần</text>
<rect x="54" y="238" width="352" height="18" rx="3" class="f-surface s-pk"/><text x="64" y="251" class="t-mono">TripHistoryTile</text>
<rect x="54" y="260" width="352" height="18" rx="3" class="f-surface s-pk"/><text x="64" y="273" class="t-mono">TripHistoryTile</text>
<rect x="54" y="282" width="352" height="18" rx="3" class="f-surface s-pk"/><text x="64" y="295" class="t-mono">TripHistoryTile</text>
<rect x="40" y="314" width="380" height="44" rx="6" class="f-phys-soft s-phys"/><text x="54" y="334" class="t-b">SliverToBoxAdapter</text><text x="54" y="351" class="t-small">chân trang: vòng quay, Thử lại, Đã hết</text>
<rect x="520" y="40" width="340" height="60" rx="10" class="f-phys-soft s-phys"/><text x="536" y="66" class="t-b">Viewport</text><text x="536" y="86" class="t-small">RenderViewport, biết vùng nhìn</text>
<rect x="520" y="290" width="340" height="60" rx="10" class="f-pk-soft s-pk"/><text x="536" y="316" class="t-b">Mỗi sliver</text><text x="536" y="336" class="t-small">tự quyết phần nào của nó cần dựng</text>
<path d="M545 102 L545 288" class="f-none s-phys" stroke-width="2" marker-end="url(#fl09-2-d)"/>
<path d="M575 288 L575 102" class="f-none s-pk" stroke-width="2" marker-end="url(#fl09-2-u)"/>
<text x="600" y="150" class="t-b t-phys">Xuống: SliverConstraints</text><text x="600" y="170" class="t-mono">scrollOffset</text><text x="600" y="186" class="t-mono">remainingPaintExtent</text>
<text x="600" y="232" class="t-b t-pk">Lên: SliverGeometry</text><text x="600" y="252" class="t-mono">scrollExtent</text><text x="600" y="268" class="t-mono">paintExtent</text>
<text x="20" y="408" class="t-small">Khi chưa có chuyến nào, SliverFillRemaining thay cho SliverList.</text>
</svg>
</div>
<figcaption>Hình 2. Bốn sliver của màn hình lịch sử trong một viewport. Viewport gửi SliverConstraints xuống, mỗi sliver trả SliverGeometry lên.</figcaption>
</figure>

Lab ghép bốn sliver. Phần `build` của màn hình, rút gọn:

### trip_history_screen.dart · 09-listview-sliver/lib/trip_history_screen.dart (trích)

```dart
child: CustomScrollView(
  controller: _controller,
  // Danh sách ngắn hơn màn hình vẫn phải kéo được để làm mới.
  physics: const AlwaysScrollableScrollPhysics(),
  slivers: [
    const SliverAppBar(
      pinned: true,
      expandedHeight: 120,
      flexibleSpace: FlexibleSpaceBar(title: Text('Lịch sử chuyến')),
    ),
    if (_trips.isNotEmpty)
      SliverToBoxAdapter(child: _Summary(trips: _trips)),
    if (_trips.isEmpty)
      SliverFillRemaining(
        hasScrollBody: false,
        child: _EmptyState(
          loading: _loading,
          error: _error,
          onRetry: _retry,
        ),
      )
    else
      SliverList.separated(
        itemCount: _trips.length,
        findItemIndexCallback: _indexOfKey,
        itemBuilder: (context, index) {
          final trip = _trips[index];
          return TripHistoryTile(
            key: ValueKey(trip.id),
            trip: trip,
            onPin: index == 0 ? null : () => _pin(trip),
          );
        },
        separatorBuilder: (context, index) => const Divider(height: 1),
      ),
    if (_trips.isNotEmpty)
      SliverToBoxAdapter(
        child: _Footer(
          loading: _loading,
          hasMore: _hasMore,
          error: _error,
          onRetry: _retry,
        ),
      ),
```

`SliverAppBar` với `pinned: true` ở lại trên cùng khi cuộn, còn `expandedHeight` cho nó chiều cao ban đầu 120 dp rồi co dần. Test cuộn 1200 dp và kiểm tiêu đề vẫn nằm trong vùng thanh công cụ. `SliverToBoxAdapter` đặt một widget thường (dòng tóm tắt, chân trang) vào viewport. Khác `SliverList`, child của nó luôn nằm trong cây widget kể cả khi đã cuộn khỏi màn hình: test tìm thấy dòng tóm tắt với `skipOffstage: false` và thấy đáy của nó khuất sau thanh công cụ. `SliverFillRemaining(hasScrollBody: false)` chiếm phần còn lại cho các trạng thái chưa có danh sách: vòng quay, lỗi, rỗng.

`SliverList.separated` là bản sliver của `ListView.separated`, dùng cùng delegate bên dưới. Hai tham số `findItemIndexCallback` và `ValueKey` sẽ được giải thích ở mục cuối.

## Nguồn dữ liệu phân trang và StatefulWidget tối thiểu

Màn hình không nên biết dữ liệu đến từ đâu. Nó chỉ biết giao diện `TripHistorySource`: cho tôi tối đa `limit` chuyến, bắt đầu từ vị trí `offset`, và cho biết còn trang sau hay không.

### trip_history_source.dart · 09-listview-sliver/lib/trip_history_source.dart (trích)

```dart
final class TripPage {
  const TripPage({required this.trips, required this.hasMore});

  final List<Trip> trips;
  final bool hasMore;
}

/// Nguồn dữ liệu lịch sử chuyến, phân trang theo vị trí.
///
/// Màn hình chỉ biết giao diện này. Ở bài 19 ta thay bằng bản gọi HTTP; ở bài
/// này chỉ có bản giả bên dưới, không có mạng.
abstract interface class TripHistorySource {
  /// Lấy tối đa [limit] chuyến, bắt đầu từ vị trí [offset] (0 là chuyến mới
  /// nhất). Ném [ArgumentError] khi [offset] âm hoặc [limit] không dương.
  Future<TripPage> fetchPage({required int offset, required int limit});
}
```

`FakeTripHistorySource` là bản giả: 47 chuyến sinh sẵn, mỗi lần gọi trả về sau một độ trễ giả, không có mạng. Cước tính bằng `calculateFare` của `toi_noi_core`, nên chuyến 5 km hiện 25.500 VND, chuyến 2,001 km hiện 12.005 VND và chuyến 100 km hiện 453.000 VND; test kiểm đúng ba số này. Cứ năm chuyến có một chuyến bị khách huỷ, các chuyến huỷ không tính cước nên dòng hiện trạng thái thay cho số tiền.

Bây giờ đến phần màn hình phải nhớ. Widget trong Flutter là bất biến, nên danh sách đã tải, cờ đang chờ và lỗi không thể nằm trong `TripHistoryScreen`. Chúng nằm trong một đối tượng `State` sống lâu hơn các lần build. **Đây là chỗ dùng `StatefulWidget` thứ nhất**:

### trip_history_screen.dart · 09-listview-sliver/lib/trip_history_screen.dart (trích)

```dart
final _controller = ScrollController();
final _trips = <Trip>[];

// Đang chờ nguồn trả lời. Mang giá trị true ngay từ đầu vì initState bắt đầu
// tải trang đầu trước khi build lần đầu.
bool _loading = true;
bool _hasMore = true;
Object? _error;

// Tăng mỗi lần làm mới. Câu trả lời của lượt cũ (generation nhỏ hơn) bị bỏ.
int _generation = 0;

@override
void initState() {
  super.initState();
  _controller.addListener(_onScroll);
  // _loading đã là true từ khai báo, nên không cần setState trước build đầu.
  _fetch(reset: true);
}
// ...
/// Bật cờ đang tải rồi gọi [_fetch]. Hoàn thành khi nguồn trả lời.
Future<void> _load({required bool reset}) {
  setState(() {
    _loading = true;
    _error = null;
  });
  return _fetch(reset: reset);
}

/// Hỏi nguồn trang đầu ([reset] true) hoặc trang kế rồi cập nhật state.
Future<void> _fetch({required bool reset}) async {
  final generation = reset ? ++_generation : _generation;
  final offset = reset ? 0 : _trips.length;

  try {
    final page = await widget.source.fetchPage(
      offset: offset,
      limit: widget.pageSize,
    );
    if (!mounted || generation != _generation) return;
    setState(() {
      if (reset) _trips.clear();
      _trips.addAll(page.trips);
      _hasMore = page.hasMore;
      _loading = false;
    });
```

Hai việc cần chú ý. Một, `setState` nhận một hàm thay đổi state; nó đánh dấu widget cần build lại ở frame kế. Lần đầu `_loading` đã là `true` từ khai báo, nên `initState` gọi thẳng `_fetch` mà không cần `setState`: chưa có build nào để báo. Hai, sau `await`, màn hình có thể đã biến mất, nên ta kiểm `mounted` trước khi đụng tới state. Lý do và cách làm đầy đủ nằm ở bài 10.

## Tải thêm cuối trang bằng ScrollController

Muốn biết người dùng đã cuộn gần cuối chưa, ta cần một `ScrollController`. Nó cho ta `position`, và `position.extentAfter` là phần nội dung còn lại phía dưới vùng nhìn, tính bằng dp (trong source: `max(maxScrollExtent - pixels, 0.0)`). Listener của controller chạy mỗi khi vị trí cuộn đổi:

### trip_history_screen.dart · 09-listview-sliver/lib/trip_history_screen.dart (trích)

```dart
void _onScroll() {
  if (!_controller.hasClients) return;
  if (_controller.position.extentAfter < widget.loadMoreThreshold) {
    _loadMore();
  }
}

void _loadMore() {
  // Cả ba điều kiện đều cần: đang tải thì không gọi chồng, hết trang thì
  // không gọi nữa, đang lỗi thì chờ người dùng bấm "Thử lại".
  if (_loading || !_hasMore || _error != null) return;
  _load(reset: false);
}
```

Còn dưới 300 dp, tức khoảng bốn dòng, thì tải trang kế. Phần khó nằm ở ba điều kiện trong `_loadMore`. Khi ngón tay trượt, listener chạy hàng chục lần mỗi giây, và nguồn trả lời mất vài trăm mili giây. Nếu không chặn, mỗi lần chạy đều thấy "còn dưới 300 dp" và gọi nguồn thêm một lần với cùng `offset`, và trang đó sẽ xuất hiện nhiều lần trong danh sách. Cờ `_loading` được đặt đồng bộ, trước `await` đầu tiên, nên lần listener kế tiếp đã thấy nó (Hình 3). `_hasMore` ngăn gọi sau trang cuối. `_error` ngăn vòng lặp "lỗi, gọi lại, lỗi": sau một lần lỗi, chỉ nút "Thử lại" mới gọi tiếp, và nó gọi lại đúng trang đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Dòng thời gian của tải thêm. Sự kiện cuộn đầu tiên đặt cờ loading và gọi fetchPage offset 20. Năm sự kiện cuộn sau đó bị chặn. Khi nguồn trả lời, cờ tắt và lần kiểm tra sau khi build gọi tiếp offset 40."><defs><marker id="fl09-3-t" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<text x="20" y="84" class="t-b">Sự kiện cuộn</text><text x="20" y="102" class="t-small">listener của controller</text>
<text x="20" y="178" class="t-b">Cờ _loading</text>
<text x="20" y="252" class="t-b">fetchPage</text>
<text x="222" y="50" class="t-b t-pk">qua cổng</text>
<path d="M240 60 L240 96" class="f-none s-pk" stroke-width="3"/>
<path d="M270 60 L270 96" class="f-none s-warn" stroke-width="3"/>
<path d="M300 60 L300 96" class="f-none s-warn" stroke-width="3"/>
<path d="M330 60 L330 96" class="f-none s-warn" stroke-width="3"/>
<path d="M360 60 L360 96" class="f-none s-warn" stroke-width="3"/>
<path d="M390 60 L390 96" class="f-none s-warn" stroke-width="3"/>
<text x="262" y="118" class="t-b t-warn">5 lần bị chặn bởi _loading</text>
<path d="M500 60 L500 96" class="f-none s-phys" stroke-width="3"/><text x="440" y="50" class="t-b t-phys">sau khi build xong</text><text x="512" y="84" class="t-small">kiểm tra lại</text>
<rect x="240" y="150" width="240" height="40" rx="6" class="f-pk-soft s-pk"/><text x="254" y="175" class="t-b">true: chờ nguồn trả lời</text>
<rect x="500" y="150" width="200" height="40" rx="6" class="f-pk-soft s-pk"/><text x="514" y="175" class="t-b">true: trang kế</text>
<text x="486" y="206" class="t-small">tắt</text>
<rect x="240" y="230" width="100" height="40" rx="6" class="f-phys-soft s-phys"/><text x="254" y="255" class="t-mono">offset 20</text>
<rect x="500" y="230" width="100" height="40" rx="6" class="f-phys-soft s-phys"/><text x="514" y="255" class="t-mono">offset 40</text>
<path d="M200 300 L860 300" class="f-none s-muted" stroke-width="2" marker-end="url(#fl09-3-t)"/><text x="780" y="320" class="t-small">thời gian</text>
</svg>
</div>
<figcaption>Hình 3. Minh hoạ: sự kiện cuộn đầu tiên đặt cờ _loading và gọi nguồn; các sự kiện sau bị chặn tới khi nguồn trả lời. Khi cờ tắt, lần kiểm tra sau khi build gọi trang kế nếu vẫn còn gần cuối.</figcaption>
</figure>

Test của nhóm này không cuộn một lần mà cuộn rồi "lắc" lên xuống trong vùng ngưỡng, để sinh nhiều sự kiện như ngón tay thật, rồi kiểm số lần nguồn bị gọi:

### trip_history_screen_test.dart · 09-listview-sliver/test/trip_history_screen_test.dart (trích)

```dart
/// Cuộn xuống cuối rồi lắc nhẹ lên xuống trong vùng ngưỡng. Mỗi cú lắc đổi vị
/// trí cuộn nên gọi listener của ScrollController một lần nữa, giống ngón tay
/// người dùng còn đang trượt gần cuối danh sách.
Future<void> _scrollToEndRepeatedly(
  WidgetTester tester, {
  int times = 3,
}) async {
  await tester.drag(_scrollView, _bottom);
  await tester.pump();
  for (var i = 0; i < times; i++) {
    await tester.drag(_scrollView, const Offset(0, 60));
    await tester.pump();
    await tester.drag(_scrollView, const Offset(0, -60));
    await tester.pump();
  }
}
```

```dart
testWidgets('cuộn tới cuối gọi đúng một lần cho dù có rất nhiều sự kiện '
    'cuộn', (tester) async {
  final source = await pumpScreen(tester);
  source.respond(0);
  await tester.pump();
  expect(source.offsets, [0]);

  await _scrollToEndRepeatedly(tester);

  // Trang kế đang chờ nguồn trả lời: chỉ có đúng một lần gọi offset 20.
  expect(source.offsets, [0, 20]);
  expect(find.byType(CircularProgressIndicator), findsOneWidget);
});
```

Có hai ca biên khác đã có test. Thứ nhất, trang đầu có thể chưa lấp đầy màn hình (ví dụ `pageSize` nhỏ): khi ấy không có gì để cuộn, nên cũng không có sự kiện cuộn nào gọi `_onScroll`. Màn hình tự hỏi lại sau khi build xong bằng `addPostFrameCallback`, và test với `pageSize: 2` thấy các offset 0, 2, 4, ... nối tiếp nhau không trùng cho tới khi đủ dài. Thứ hai, ngưỡng bằng 0 không bao giờ kích hoạt, vì `extentAfter` không âm nên điều kiện `extentAfter < 0` không bao giờ đúng (bài tập 1).

## Kéo để làm mới với RefreshIndicator

`RefreshIndicator` bọc một widget cuộn. Khi người dùng kéo xuống quá đầu danh sách, nó hiện vòng quay và gọi `onRefresh`; `onRefresh` chỉ chạy khi người dùng kéo đủ ngưỡng rồi thả tay. Vòng quay biến mất khi `Future` mà `onRefresh` trả về hoàn thành; ta trả về `_load(reset: true)` nên nó quay đúng đến lúc nguồn trả lời.

### trip_history_screen.dart · 09-listview-sliver/lib/trip_history_screen.dart (trích)

```dart
body: RefreshIndicator(
  onRefresh: () => _load(reset: true),
  child: CustomScrollView(
    controller: _controller,
    // Danh sách ngắn hơn màn hình vẫn phải kéo được để làm mới.
    physics: const AlwaysScrollableScrollPhysics(),
```

Dòng `physics` có lý do dễ bỏ sót. Một `ScrollView` dọc không truyền `controller` được tự gán `AlwaysScrollableScrollPhysics` (constructor của `ScrollView`), nên danh sách ngắn vẫn kéo để làm mới được. Lab có truyền `controller` để tải thêm, nên mất phần tự gán đó và rơi về physics mặc định của nền tảng (`BouncingScrollPhysics` trên iOS, `ClampingScrollPhysics` trên Android), vốn từ chối kéo khi nội dung ngắn hơn màn hình. Ta phải đặt lại `AlwaysScrollableScrollPhysics`. `refresh_physics_test.dart` chạy cả Android lẫn iOS: không controller thì `onRefresh` chạy 1 lần, có controller mà thiếu physics thì 0 lần, thêm physics thì lại 1 lần.

Làm mới và tải thêm có thể chạy chồng nhau. Hình dung: người dùng đang chờ trang `offset: 20`, kéo làm mới, và hai câu trả lời về lệch thứ tự: nhận cả hai thì danh sách thành trang đầu mới nối với một trang cũ. Biến `_generation` giải quyết việc này: mỗi lần làm mới tăng nó lên một, và mọi câu trả lời mang số cũ bị bỏ ở dòng `generation != _generation` trong `_fetch`. Test dựng đúng kịch bản này bằng `ManualSource` (test tự quyết lúc nào nguồn trả lời) và kiểm câu trả lời quá hạn không được nối vào.

Lỗi khi làm mới khác lỗi khi tải trang đầu: nếu đã có danh sách, ta giữ nguyên nó và báo bằng một `SnackBar`, không xoá cả danh sách chỉ vì một lần mất mạng.

## Key trong danh sách: state phải đi theo chuyến

Mỗi dòng `TripHistoryTile` có một state riêng: đang mở hay gập. **Đây là chỗ dùng `StatefulWidget` thứ hai:**

### trip_history_tile.dart · 09-listview-sliver/lib/trip_history_tile.dart (trích)

```dart
class _TripHistoryTileState extends State<TripHistoryTile> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final trip = widget.trip;
    final completed = trip.status == TripStatus.completed;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        ListTile(
          leading: Icon(_iconOf(trip.serviceKind)),
          title: Text('${trip.serviceKind.label} · ${trip.id}'),
          subtitle: Text(
            '${formatTripTime(trip.requestedAt.toLocal())} · '
            '${trip.distanceKm} km',
          ),
          trailing: Text(completed ? trip.fare.toString() : trip.status.label),
          onTap: () => setState(() => _expanded = !_expanded),
```

State thuộc về Element, không thuộc về widget. Khi danh sách build lại, framework ghép mỗi widget mới với một Element cũ nếu cùng `runtimeType` và cùng `key` (hàm `Widget.canUpdate`). Widget không có key được ghép theo vị trí. Màn hình có nút "Ghim lên đầu" làm thứ tự đổi, nên điều này quan trọng. Test đảo cả danh sách, mở T-0046 ở vị trí 1 rồi đảo:

### keys_test.dart · 09-listview-sliver/test/keys_test.dart (trích)

```dart
testWidgets('không key: state ở lại vị trí, dòng mở sai chuyến', (
  tester,
) async {
  await pumpOrder(tester, trips, useKey: false, useFindIndex: false);
  await tester.tap(find.textContaining('T-0046'));
  await tester.pump();
  expect(isExpanded('T-0046'), isTrue);

  await pumpOrder(
    tester,
    trips.reversed.toList(),
    useKey: false,
    useFindIndex: false,
  );

  // Vị trí số 1 vẫn đang mở, nhưng giờ nó hiển thị chuyến T-0044.
  expect(isExpanded('T-0046'), isFalse);
  expect(isExpanded('T-0044'), isTrue);
});
```

Không key thì vị trí 1 vẫn đang mở, nhưng nó đã hiển thị T-0044 (Hình 4, cột trái). Khách bấm ghim một chuyến và thấy một chuyến khác bung ra.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 4. Danh sách năm chuyến, chuyến T-0046 đang mở. Sau khi đảo thứ tự, không key thì vị trí 1 vẫn mở và lúc này là T-0044. Có ValueKey và findChildIndexCallback thì T-0046 đổi sang vị trí 3 và vẫn mở."><defs><marker id="fl09-4-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<text x="20" y="30" class="t-b">Không key</text>
<text x="20" y="56" class="t-small">Trước: T-0046 đang mở</text>
<rect x="20" y="66" width="76" height="52" rx="6" class="f-surface s-line"/><text x="28" y="88" class="t-mono">T-0047</text>
<rect x="102" y="66" width="76" height="52" rx="6" class="f-pk-soft s-pk"/><text x="110" y="88" class="t-mono">T-0046</text>
<text x="110" y="106" class="t-small">mở</text>
<rect x="184" y="66" width="76" height="52" rx="6" class="f-surface s-line"/><text x="192" y="88" class="t-mono">T-0045</text>
<rect x="266" y="66" width="76" height="52" rx="6" class="f-surface s-line"/><text x="274" y="88" class="t-mono">T-0044</text>
<rect x="348" y="66" width="76" height="52" rx="6" class="f-surface s-line"/><text x="356" y="88" class="t-mono">T-0043</text>
<path d="M225 124 L225 168" class="f-none s-muted" stroke-width="2" marker-end="url(#fl09-4-a)"/><text x="237" y="150" class="t-small">đảo thứ tự</text>
<text x="20" y="192" class="t-small">Sau khi đảo</text>
<rect x="20" y="202" width="76" height="52" rx="6" class="f-surface s-line"/><text x="28" y="224" class="t-mono">T-0043</text>
<rect x="102" y="202" width="76" height="52" rx="6" class="f-warn-soft s-warn"/><text x="110" y="224" class="t-mono">T-0044</text>
<text x="110" y="242" class="t-small">mở</text>
<rect x="184" y="202" width="76" height="52" rx="6" class="f-surface s-line"/><text x="192" y="224" class="t-mono">T-0045</text>
<rect x="266" y="202" width="76" height="52" rx="6" class="f-surface s-line"/><text x="274" y="224" class="t-mono">T-0046</text>
<rect x="348" y="202" width="76" height="52" rx="6" class="f-surface s-line"/><text x="356" y="224" class="t-mono">T-0047</text>
<text x="20" y="286" class="t-b t-warn">Vị trí 1 vẫn mở, nhưng giờ là T-0044: sai chuyến</text>
<text x="450" y="30" class="t-b">ValueKey + findChildIndexCallback</text>
<text x="450" y="56" class="t-small">Trước: T-0046 đang mở</text>
<rect x="450" y="66" width="76" height="52" rx="6" class="f-surface s-line"/><text x="458" y="88" class="t-mono">T-0047</text>
<rect x="532" y="66" width="76" height="52" rx="6" class="f-pk-soft s-pk"/><text x="540" y="88" class="t-mono">T-0046</text>
<text x="540" y="106" class="t-small">mở</text>
<rect x="614" y="66" width="76" height="52" rx="6" class="f-surface s-line"/><text x="622" y="88" class="t-mono">T-0045</text>
<rect x="696" y="66" width="76" height="52" rx="6" class="f-surface s-line"/><text x="704" y="88" class="t-mono">T-0044</text>
<rect x="778" y="66" width="76" height="52" rx="6" class="f-surface s-line"/><text x="786" y="88" class="t-mono">T-0043</text>
<path d="M655 124 L655 168" class="f-none s-muted" stroke-width="2" marker-end="url(#fl09-4-a)"/><text x="667" y="150" class="t-small">đảo thứ tự</text>
<text x="450" y="192" class="t-small">Sau khi đảo</text>
<rect x="450" y="202" width="76" height="52" rx="6" class="f-surface s-line"/><text x="458" y="224" class="t-mono">T-0043</text>
<rect x="532" y="202" width="76" height="52" rx="6" class="f-surface s-line"/><text x="540" y="224" class="t-mono">T-0044</text>
<rect x="614" y="202" width="76" height="52" rx="6" class="f-surface s-line"/><text x="622" y="224" class="t-mono">T-0045</text>
<rect x="696" y="202" width="76" height="52" rx="6" class="f-pk-soft s-pk"/><text x="704" y="224" class="t-mono">T-0046</text>
<text x="704" y="242" class="t-small">mở</text>
<rect x="778" y="202" width="76" height="52" rx="6" class="f-surface s-line"/><text x="786" y="224" class="t-mono">T-0047</text>
<text x="450" y="286" class="t-b t-pk">T-0046 sang vị trí 3 và vẫn mở: đúng chuyến</text>
<text x="20" y="320" class="t-small">Thiếu findChildIndexCallback, dòng đổi vị trí vẫn bị dựng lại và gập (keys_test.dart).</text>
</svg>
</div>
<figcaption>Hình 4. Cùng một lần đảo thứ tự: không key thì state ở lại vị trí và hiện sai chuyến, ValueKey cùng findChildIndexCallback thì state đi theo chuyến.</figcaption>
</figure>

Thêm `ValueKey(trip.id)` là chưa đủ. `ListView.builder` và `SliverList` tạo dòng theo chỉ số, nên khi thứ tự đổi, framework cần biết dòng mang key này giờ nằm ở chỉ số nào. Việc đó do `findChildIndexCallback` làm (với `separated` là `findItemIndexCallback`, vì `findChildIndexCallback` của `separated` đã bị đánh dấu deprecated trong source). Tài liệu của `SliverChildBuilderDelegate` nói rõ thiếu nó thì state có thể mất, và test xác nhận: chỉ có `ValueKey`, T-0046 đổi vị trí nên bị dựng lại và gập. May là không "mở nhầm" T-0044 như trường hợp không key. Có cả hai thì state đi theo chuyến (Hình 4, cột phải). Lab truyền key và callback như sau:

### trip_history_screen.dart · 09-listview-sliver/lib/trip_history_screen.dart (trích)

```dart
int? _indexOfKey(Key key) {
  if (key is! ValueKey<String>) return null;
  final index = _trips.indexWhere((trip) => trip.id == key.value);
  return index < 0 ? null : index;
}
```

```dart
SliverList.separated(
  itemCount: _trips.length,
  findItemIndexCallback: _indexOfKey,
  itemBuilder: (context, index) {
    final trip = _trips[index];
    return TripHistoryTile(
      key: ValueKey(trip.id),
      trip: trip,
      onPin: index == 0 ? null : () => _pin(trip),
    );
  },
  separatorBuilder: (context, index) => const Divider(height: 1),
),
```

Key phải ổn định và duy nhất: dùng `trip.id`, đừng dùng chỉ số trong danh sách. `indexWhere` duyệt tuyến tính mỗi lần được hỏi, nên danh sách rất lớn nên giữ một `Map<String, int>`. Test ở mức màn hình thật mở hai dòng, ghim một dòng lên đầu và kiểm cả hai vẫn mở đúng chuyến.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với danh sách cuộn</span>
<ul>
<li><strong>Đặt <code>ListView</code> trong <code>Column</code> không có <code>Expanded</code>.</strong> Dấu hiệu: assertion "Vertical viewport was given unbounded height" (test bắt đúng thông báo này). Nguyên nhân: <code>Column</code> không giới hạn chiều cao của con, còn viewport cần chiều cao hữu hạn. Cách sửa: bọc <code>Expanded</code> như bài 08, hoặc thay cả <code>Column</code> bằng <code>CustomScrollView</code> với các sliver.</li>
<li><strong>Chữa lỗi trên bằng <code>shrinkWrap: true</code> bên trong một cuộn khác.</strong> Dấu hiệu: màn hình chạy nhưng giật khi danh sách dài. Nguyên nhân: <code>shrinkWrap</code> buộc danh sách đo toàn bộ con, test thấy đủ 500 lần build. Cách sửa: dùng một <code>CustomScrollView</code> với các sliver.</li>
<li><strong>Tải thêm gọi trùng.</strong> Dấu hiệu: một trang xuất hiện hai lần, hoặc nguồn bị gọi nhiều lần cùng <code>offset</code>. Nguyên nhân: thiếu cờ <code>_loading</code>, hoặc đặt nó sau <code>await</code>. Cách sửa: đặt cờ đồng bộ trước <code>await</code> đầu tiên.</li>
<li><strong>Quên key khi danh sách đổi thứ tự.</strong> Dấu hiệu: dòng mở, ô nhập hoặc ô chọn "dính" vào vị trí thay vì đi theo dữ liệu. Cách sửa: <code>ValueKey</code> theo id, kèm <code>findChildIndexCallback</code> hoặc <code>findItemIndexCallback</code> với danh sách builder.</li>
<li><strong>Danh sách ngắn không kéo để làm mới được.</strong> Dấu hiệu: <code>RefreshIndicator</code> không phản ứng khi chỉ có vài chuyến, thường sau khi thêm <code>ScrollController</code>. Nguyên nhân: truyền controller làm mất <code>AlwaysScrollableScrollPhysics</code> tự gán, danh sách ngắn rơi về physics mặc định và từ chối kéo. Cách sửa: <code>physics: AlwaysScrollableScrollPhysics()</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/09-listview-sliver
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 (Dart 3.13.3): `flutter analyze` báo 0 issue và `flutter test` có 51 test xanh. Mức kiểm chứng: **Kiểm chứng tự động**, không cần thiết bị. Các con số 12, 412 và 28 là giá trị đo ở Flutter 3.47.4 với màn hình test 800 x 600 dp và đều được test khoá cứng. Các test dùng `pump` với độ trễ giả hoặc nguồn do test điều khiển, không phụ thuộc mạng hay đồng hồ thật.

| Test | Chứng minh điều gì |
|---|---|
| `lazy_build_test.dart` | 500 phần tử chỉ build 12 lần ở lần dựng đầu; cuộn tay vài màn hình vẫn dưới 60 dòng; nhảy xa build 412 lần tổng cộng, có `itemExtent` chỉ 28; `ListView` trong `Column` ném lỗi unbounded height; `shrinkWrap` build đủ 500 |
| `refresh_physics_test.dart` | Danh sách ngắn kéo làm mới được hay không, trên cả Android và iOS: không controller có, có controller thiếu physics không, thêm `AlwaysScrollableScrollPhysics` thì có |
| `keys_test.dart` | Đảo danh sách: không key mở sai chuyến, `ValueKey` + `findChildIndexCallback` giữ đúng chuyến, chỉ `ValueKey` thì mất state |
| `trip_history_screen_test.dart` | Trang đầu, tóm tắt cộng cước, trạng thái rỗng và lỗi, SliverAppBar ghim, tải thêm không gọi trùng, lỗi rồi Thử lại, ngưỡng 0, lấp đầy màn hình, kéo để làm mới, bỏ câu trả lời quá hạn, ghim giữ state |
| `trip_history_source_test.dart` | Trang 20, 20, 7; cước 25.500, 12.005, 453.000 VND; ca biên offset và limit; phân trang theo offset bị lặp khi có chuyến chen vào đầu |
| `app_test.dart` | `ToiNoiApp` mở thẳng màn hình lịch sử và tải trang đầu |

**Chưa làm**: chạy trên iOS Simulator, Android emulator hay thiết bị thật (máy kiểm chưa có Android SDK và Xcode đầy đủ), nên cảm giác cuộn thật của iOS và Android chưa được kiểm; phân trang theo con trỏ và khử trùng theo `id`; định dạng ngày giờ và tiền theo locale (bài 14); điều hướng sang chi tiết chuyến (bài 12); nguồn gọi HTTP thật (bài 19). Để chạy trên máy, sinh runner một lần bằng `flutter create --platforms=ios,android --project-name toi_noi_09 --org vn.toinoi .` trong thư mục lab, như ghi trong README.

## Bài tập

1. Đặt `loadMoreThreshold: 0` rồi cuộn tới cuối. Chuyện gì xảy ra với việc tải thêm, và vì sao?
2. Trong `keys_test.dart`, đổi `trips.reversed.toList()` thành danh sách đưa T-0044 lên đầu (`[T-0044, T-0047, T-0046, T-0045, T-0043]`). Với phiên bản không key, sau khi mở T-0046 ở vị trí 1, chuyến nào đang mở sau khi đổi?
3. Phân trang theo `offset` có lỗ hổng khi dữ liệu đổi trong lúc người dùng đang cuộn. Nếu một chuyến mới chen vào đầu danh sách giữa lần tải trang 1 và trang 2, danh sách trên máy sẽ ra sao, và nêu hai cách sửa.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Việc tải thêm không bao giờ chạy. <code>extentAfter</code> không bao giờ âm (source dùng <code>max(..., 0.0)</code>), nên điều kiện <code>extentAfter &lt; 0</code> luôn sai. Test <code>ngưỡng 0 không bao giờ kích hoạt</code> kiểm đúng điều này: nguồn chỉ bị gọi với <code>offset</code> 0.</p>
<p><strong>2.</strong> T-0047. Không key thì Element ghép theo vị trí: Element ở vị trí 1 giữ <code>_expanded = true</code>, và sau khi đổi, vị trí 1 hiển thị T-0047. Chuyến T-0046 được mở lúc đầu lại nằm ở vị trí 2 và đóng. Có <code>ValueKey</code> cùng <code>findChildIndexCallback</code> thì T-0046 vẫn mở dù chuyển sang vị trí 2.</p>
<p><strong>3.</strong> Chuyến chen vào đẩy mọi chuyến cũ xuống một vị trí, nên trang 2 (<code>offset: 20</code>) bắt đầu bằng đúng chuyến cuối của trang 1: danh sách có một chuyến hiện hai lần (test <code>phân trang theo offset</code> trong <code>trip_history_source_test.dart</code> tái hiện điều này). Cách sửa: một là phân trang theo con trỏ (gửi id hoặc thời điểm của chuyến cuối đã có, nguồn trả những chuyến cũ hơn nó); hai là khử trùng theo <code>id</code> khi nối trang mới vào danh sách.</p>
</details>

Bài trước: [Layout và constraints: Row, Column, Expanded và thẻ chuyến](/articles/flutter-layout-constraints-row-column-the-chuyen). Bài tiếp: [StatefulWidget, lifecycle và setState: chọn dịch vụ và đếm ngược tìm tài xế](/articles/flutter-statefulwidget-lifecycle-setstate-chon-dich-vu).

## Tài liệu gốc

- [Using slivers to achieve fancy scrolling](https://docs.flutter.dev/ui/layout/scrolling/slivers) và [Work with long lists](https://docs.flutter.dev/cookbook/lists/long-lists) (docs.flutter.dev).
- [Place a floating app bar above a list](https://docs.flutter.dev/cookbook/lists/floating-app-bar) (docs.flutter.dev).
- [ListView](https://api.flutter.dev/flutter/widgets/ListView-class.html), [CustomScrollView](https://api.flutter.dev/flutter/widgets/CustomScrollView-class.html) và [SliverList](https://api.flutter.dev/flutter/widgets/SliverList-class.html) (api.flutter.dev).
- [SliverAppBar](https://api.flutter.dev/flutter/material/SliverAppBar-class.html) và [SliverToBoxAdapter](https://api.flutter.dev/flutter/widgets/SliverToBoxAdapter-class.html) (api.flutter.dev).
- [SliverChildBuilderDelegate](https://api.flutter.dev/flutter/widgets/SliverChildBuilderDelegate-class.html), gồm `findChildIndexCallback` (api.flutter.dev).
- [SliverConstraints](https://api.flutter.dev/flutter/rendering/SliverConstraints-class.html) và [SliverGeometry](https://api.flutter.dev/flutter/rendering/SliverGeometry-class.html) (api.flutter.dev).
- [ScrollController](https://api.flutter.dev/flutter/widgets/ScrollController-class.html) và [ScrollMetrics.extentAfter](https://api.flutter.dev/flutter/widgets/ScrollMetrics/extentAfter.html) (api.flutter.dev).
- [RefreshIndicator](https://api.flutter.dev/flutter/material/RefreshIndicator-class.html) và [AlwaysScrollableScrollPhysics](https://api.flutter.dev/flutter/widgets/AlwaysScrollableScrollPhysics-class.html) (api.flutter.dev).
- [Key](https://api.flutter.dev/flutter/foundation/Key-class.html) (api.flutter.dev).
