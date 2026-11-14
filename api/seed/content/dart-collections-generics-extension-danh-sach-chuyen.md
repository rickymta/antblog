---
slug: dart-collections-generics-extension-danh-sach-chuyen
title: "Collections, generics và extension: quản lý danh sách chuyến và tài xế"
summary: "Bài 04: chọn List, Set, Map theo thao tác nghiệp vụ; hiểu Iterable lười, where/map/fold, sắp xếp ổn định; viết generic Paged<T> và extension trên Iterable<Trip> để tổng cước, nhóm theo trạng thái. Có test chứng minh query lười chỉ chạy khi duyệt."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, collections, generics]
language: vi
---

Màn hình "Lịch sử chuyến" của app khách Tới Nơi cần ba việc: cộng tổng cước các chuyến, gom chuyến theo trạng thái và hiện ba chuyến mỗi trang. Phía app tài xế lại hỏi chuyện khác: tài xế mang id `D-23` là ai, và thành phố nào đang có người online. Cả hai chỉ là câu hỏi về một tập dữ liệu, nhưng mỗi câu hợp với một cấu trúc khác nhau. Chọn sai, ta vẫn ra kết quả đúng, chỉ là chậm hơn, rối hơn hoặc dễ sai khi dữ liệu đổi.

Đây là bài 04 của lộ trình **Flutter qua Tới Nơi**, dùng dự án mẫu hư cấu [Tới Nơi](/articles/du-an-mau-toi-noi). Bài này cần `Trip`, `TripStatus` và `enum` từ [bài 03](/articles/dart-ham-class-enum-sealed-pattern-matching-trang-thai-chuyen). Ta dùng bản rút gọn của chúng trong lab. Nếu đã đọc [Generics và collection trong C#](/articles/csharp-generics-collections), bạn sẽ thấy nhiều ý quen: `List<T>`, `HashSet<T>`, `Dictionary<K,V>`. Điểm khác chính của Dart là `Iterable` lười và extension method gắn lên kiểu có sẵn.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Sau dòng <code>final ids = trips.where((t) => !t.status.isTerminal).map((t) =&gt; t.id);</code>, hàm trong <code>where</code> đã chạy chưa? Nếu sau đó gọi <code>ids.length</code> rồi <code>ids.toList()</code>, ta duyệt danh sách mấy lần?</p>
<details><summary>Xem đáp án</summary><p>Chưa chạy lần nào: <code>where</code> và <code>map</code> chỉ dựng một pipeline. Hai lệnh sau mỗi lệnh duyệt lại từ đầu, nên hàm lọc chạy hai lượt trên toàn bộ danh sách. Mục "Iterable lười" bên dưới kiểm bằng bộ đếm.</p></details>
</div>

## Ba cấu trúc, ba câu hỏi

Đừng chọn collection theo thói quen. Hãy bắt đầu từ câu hỏi nghiệp vụ.

- **List** trả lời "phần tử thứ mấy" và "theo thứ tự nào": lịch sử chuyến, trang kết quả, các tab của màn hình chính.
- **Set** trả lời "có hay chưa": thành phố có tài xế online, tập id đã gửi thông báo. Set tự loại trùng.
- **Map** trả lời "ứng với khoá này là gì": tài xế theo id, số chuyến theo trạng thái. Tra theo khoá, không phải tìm lần lượt.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Chọn cấu trúc theo câu hỏi nghiệp vụ. List: thứ tự và chỉ số, ví dụ lịch sử chuyến. Set: có hay chưa, tự loại trùng, ví dụ thành phố có tài xế online. Map: tra theo khoá, ví dụ tài xế theo id."><rect x="20" y="20" width="260" height="200" rx="10" class="f-phys-soft s-phys"/><text x="36" y="52" class="t-b">List&lt;Trip&gt;</text><text x="36" y="80" class="t-small">Câu hỏi: thứ tự? phần tử thứ mấy?</text><text x="36" y="112" class="t-mono">trips[0], trips.length</text><text x="36" y="140" class="t-small">Cho phép trùng, giữ thứ tự chèn</text><text x="36" y="172" class="t-small">Ví dụ: lịch sử chuyến, một trang</text><text x="36" y="196" class="t-small">Tìm theo giá trị: duyệt lần lượt</text><rect x="310" y="20" width="260" height="200" rx="10" class="f-pk-soft s-pk"/><text x="326" y="52" class="t-b">Set&lt;String&gt;</text><text x="326" y="80" class="t-small">Câu hỏi: có hay chưa? duy nhất?</text><text x="326" y="112" class="t-mono">cities.contains('Huế')</text><text x="326" y="140" class="t-small">Thêm trùng: bị bỏ qua, add trả false</text><text x="326" y="172" class="t-small">Ví dụ: thành phố có tài xế online</text><text x="326" y="196" class="t-small">Không có chỉ số: không có cities[0]</text><rect x="600" y="20" width="260" height="200" rx="10" class="f-bg s-line"/><text x="616" y="52" class="t-b">Map&lt;String, Driver&gt;</text><text x="616" y="80" class="t-small">Câu hỏi: khoá này ứng với gì?</text><text x="616" y="112" class="t-mono">byId['D-23'] → Driver?</text><text x="616" y="140" class="t-small">Khoá duy nhất, thiếu khoá thì null</text><text x="616" y="172" class="t-small">Ví dụ: tài xế theo id</text><text x="616" y="196" class="t-small">Tra theo khoá, không duyệt hết</text></svg>
</div>
<figcaption>Hình 1. Ba câu hỏi, ba cấu trúc. Cùng một danh sách tài xế có thể dựng thành List, Set thành phố hoặc Map theo id tuỳ việc cần làm.</figcaption>
</figure>

Dart cho viết collection thẳng bằng literal, kèm ba tiện ích: **collection-if** chọn phần tử theo điều kiện, **collection-for** sinh phần tử từ vòng lặp, **spread** `...` chèn cả một collection vào chỗ đang viết.

### literals.dart · 04-collections-generics/lib/literals.dart

```dart
import 'model.dart';

/// Hai trạng thái kết thúc, là const Set: bất biến và dùng chung một bản.
const terminalStatuses = <TripStatus>{
  TripStatus.completed,
  TripStatus.cancelled,
};

/// collection-if chọn phần tử theo điều kiện; spread `...` chèn cả list.
List<String> homeTabs({
  required bool isDriverApp,
  List<String> extra = const [],
}) => ['Chuyến', if (isDriverApp) 'Thu nhập' else 'Ưu đãi', ...extra];

/// collection-for dựng Map: mọi trạng thái bắt đầu từ 0 chuyến.
Map<TripStatus, int> emptyStatusCounts() => {
  for (final status in TripStatus.values) status: 0,
};
```

Vài điểm cần nhớ. Khi không có kiểu ngữ cảnh, `{}` trơn là **Map** rỗng; muốn Set rỗng viết `<String>{}` hoặc khai báo `final Set<String> s = {};`. Literal mặc định giữ thứ tự chèn, cả Set lẫn Map. Từ khoá `const` tạo collection bất biến, nên `terminalStatuses.add(...)` ném `UnsupportedError`; giá trị mặc định `extra = const []` cũng nhờ vậy mà an toàn dùng chung giữa mọi lần gọi. Trong collection-for của Map, ta viết `status: 0` như một mục `key: value`.

Nếu thao tác "đã có chưa" chạy nhiều lần trên danh sách lớn, hãy đổi sang Set: `contains` của Set không phải duyệt từng phần tử.

## Iterable lười: where và map chưa chạy cho tới khi duyệt

`List`, `Set` và kết quả của `where`, `map`, `take`, `skip` đều là `Iterable`. Điểm khác nhau là List và Set giữ dữ liệu thật, còn kết quả của `where` hay `map` chỉ là **công thức** "lấy từ nguồn, lọc, rồi biến đổi". Công thức chỉ chạy khi có ai duyệt: `for`, `toList()`, `length`, `first`, `join`...

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 2. Pipeline lười where, map, toList. Ba bước đầu chỉ dựng công thức nên bộ đếm bằng 0. Khi toList duyệt, where chạy 5 lần và map chạy 2 lần với 5 chuyến mẫu. Gọi toList lần nữa sẽ chạy lại từ đầu."><defs><marker id="fl04-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">trips</text><text x="34" y="88" class="t-small">List, 5 chuyến (dữ liệu thật)</text><path d="M190 70 L220 70" class="f-none s-phys" stroke-width="2" marker-end="url(#fl04-a)"/><rect x="220" y="30" width="170" height="80" rx="10" class="f-pk-soft s-pk"/><text x="234" y="62" class="t-b">where(...)</text><text x="234" y="88" class="t-small">lọc, chưa chạy</text><path d="M390 70 L420 70" class="f-none s-phys" stroke-width="2" marker-end="url(#fl04-a)"/><rect x="420" y="30" width="170" height="80" rx="10" class="f-pk-soft s-pk"/><text x="434" y="62" class="t-b">map(...)</text><text x="434" y="88" class="t-small">biến đổi, chưa chạy</text><path d="M590 70 L620 70" class="f-none s-phys" stroke-width="2" marker-end="url(#fl04-a)"/><rect x="620" y="30" width="240" height="80" rx="10" class="f-warn-soft s-warn"/><text x="634" y="62" class="t-b">toList()</text><text x="634" y="88" class="t-small">duyệt thật, ra List mới (snapshot)</text><path d="M20 140 L590 140" class="f-none s-pk" stroke-width="2"/><text x="20" y="166" class="t-b">Chưa chạy: predicateCalls = 0, projectionCalls = 0</text><path d="M620 140 L860 140" class="f-none s-warn" stroke-width="2"/><text x="620" y="166" class="t-b">Chạy khi duyệt: where ×5, map ×2</text><text x="20" y="210" class="t-small">Gọi toList() lần hai: chạy lại từ đầu, bộ đếm lên where ×10, map ×4. Pipeline không nhớ kết quả.</text></svg>
</div>
<figcaption>Hình 2. Pipeline lười. Ba ô đầu chỉ mô tả việc cần làm; công việc thật xảy ra ở ô duyệt cuối cùng, và mỗi lần duyệt chạy lại từ đầu.</figcaption>
</figure>

Để thấy tận mắt, lab có `QueryProbe` đếm số lần `where` và `map` được gọi.

### lazy_demo.dart · 04-collections-generics/lib/lazy_demo.dart

```dart
import 'model.dart';

/// Đếm side effect để thấy Iterable lười chạy khi nào.
class QueryProbe {
  int predicateCalls = 0;
  int projectionCalls = 0;

  /// where + map: chỉ dựng pipeline, chưa gọi hàm nào của hai bước.
  Iterable<String> activeIds(Iterable<Trip> trips) => trips
      .where((trip) {
        predicateCalls++;
        return !trip.status.isTerminal;
      })
      .map((trip) {
        projectionCalls++;
        return trip.id;
      });
}

/// Nhận Iterable: truyền List, Set hay một pipeline lười đều được.
int countCancellable(Iterable<Trip> trips) =>
    trips.where((trip) => trip.status.canCancel).length;
```

Test dựng pipeline trên năm chuyến mẫu (hai completed, một inTransit, một searching, một cancelled) rồi quan sát bộ đếm (trích `test/lazy_demo_test.dart`).

```dart
    test('dựng where + map chưa chạy hàm nào: bộ đếm vẫn bằng 0', () {
      final probe = QueryProbe();
      final trips = fiveTrips();
      final ids = probe.activeIds(trips);
      expect(probe.predicateCalls, 0);
      expect(probe.projectionCalls, 0);
      expect(ids, isA<Iterable<String>>());
      expect(ids, isNot(isA<List<String>>()));
    });

    test(
      'toList duyệt hết: where chạy n lần, map chỉ chạy cho chuyến qua lọc',
      () {
        final probe = QueryProbe();
        final trips = fiveTrips();
        final snapshot = probe.activeIds(trips).toList();
        expect(snapshot, ['T2', 'T3']);
        expect(probe.predicateCalls, trips.length); // n = 5
        expect(probe.projectionCalls, 2);
      },
    );

    test('duyệt hai lần thì đếm hai lần', () {
      final probe = QueryProbe();
      final ids = probe.activeIds(fiveTrips());
      ids.toList();
      ids.toList();
      expect(probe.predicateCalls, 10);
      expect(probe.projectionCalls, 4);
    });
```

Bốn điều rút ra từ các test này:

1. **Dựng chưa chạy.** Ngay sau `activeIds(...)`, cả hai bộ đếm bằng 0, và kết quả là `Iterable`, không phải `List`.
2. **`toList()` là snapshot.** `where` chạy năm lần, `map` chỉ chạy hai lần vì chỉ hai chuyến qua được bộ lọc. Sau đó thêm chuyến vào danh sách nguồn thì snapshot không đổi, còn pipeline lười nhìn thấy chuyến mới ở lần duyệt kế tiếp.
3. **Duyệt hai lần là chạy hai lần.** Bẫy kinh điển: lưu `Iterable` vào biến rồi vừa gọi `length` vừa gọi `toList()`. Nếu hàm lọc tốn kém, hoặc có side effect như ghi log, hàm đó chạy gấp đôi. Cần dùng nhiều lần thì gọi `toList()` một lần rồi dùng List.
4. **Duyệt dừng sớm được.** `first` chỉ kiểm tra tới chuyến đầu tiên thoả điều kiện, ở ví dụ là hai lần gọi `where` chứ không phải năm.

Điều này ảnh hưởng tới cách thiết kế chữ ký hàm. **Nhận** `Iterable<Trip>` khi hàm chỉ cần duyệt: List, Set hay pipeline lười đều truyền được, như `countCancellable` ở trên. **Trả** `List<Trip>` khi người gọi cần chỉ số, cần giữ kết quả, hoặc khi nguồn có thể đổi sau đó. Trả `Iterable` lười từ một getter chỉ nên làm khi bạn muốn người gọi tiếp tục nối thêm `where` hay `take` và hiểu rằng pipeline đó chưa chạy. Cơ chế này tương tự LINQ deferred execution của C#; bài [Deferred execution trong LINQ](/articles/csharp-linq-deferred-execution) giải thích phiên bản .NET, bẫy duyệt hai lần cũng giống hệt.

## fold, reduce, any, every, firstWhere và nhóm theo khoá

Sau khi có pipeline, ta cần gom nó thành một kết quả.

- **`fold(initial, combine)`** cộng dồn từ giá trị khởi đầu; kiểu kết quả có thể khác kiểu phần tử. Danh sách rỗng thì trả `initial`: tổng cước của danh sách rỗng là 0.
- **`reduce(combine)`** không có giá trị khởi đầu, nên danh sách rỗng ném `StateError`. Chỉ dùng khi chắc không rỗng, hoặc chặn trước như `priciest` bên dưới.
- **`any`** và **`every`** trả `bool`. `every` trên danh sách rỗng là `true`.
- **`firstWhere`** ném `StateError` khi không có phần tử nào, trừ khi có `orElse`.

### driver_lookup.dart · 04-collections-generics/lib/driver_lookup.dart

```dart
import 'package:collection/collection.dart' show IterableExtension;

import 'model.dart';

/// Map tra tài xế theo id. Trùng id thì bản sau ghi đè bản trước.
Map<String, Driver> indexById(Iterable<Driver> drivers) => {
  for (final driver in drivers) driver.id: driver,
};

/// Set thành phố có ít nhất một tài xế online. Trùng thành phố tự bị loại.
Set<String> onlineCities(Iterable<Driver> drivers) => {
  for (final driver in drivers)
    if (driver.isOnline) driver.city,
};

/// Tra tài xế theo id; id null hoặc không có trong map đều trả null.
Driver? findDriver(Map<String, Driver> byId, String? id) =>
    id == null ? null : byId[id];

/// Tài xế của một chuyến (Trip.driverId có thể null).
Driver? driverOf(Trip trip, Map<String, Driver> byId) =>
    findDriver(byId, trip.driverId);

/// any: có ít nhất một tài xế online trong thành phố?
bool hasOnlineDriverIn(Iterable<Driver> drivers, String city) =>
    drivers.any((driver) => driver.isOnline && driver.city == city);

/// firstWhereOrNull của package:collection: không có thì trả null.
Driver? firstOnlineIn(Iterable<Driver> drivers, String city) => drivers
    .firstWhereOrNull((driver) => driver.isOnline && driver.city == city);

/// firstWhere + orElse: không có thì dùng tài xế dự phòng thay vì StateError.
Driver firstOnlineOrFallback(
  Iterable<Driver> drivers,
  String city,
  Driver fallback,
) => drivers.firstWhere(
  (driver) => driver.isOnline && driver.city == city,
  orElse: () => fallback,
);
```

Hai hàm ở đầu file dựng Map và Set bằng collection-for, đúng như mục "Ba cấu trúc, ba câu hỏi". `findDriver` cho thấy hợp đồng của Map: khoá không có thì `byId[id]` trả `null`, không ném lỗi, nên kiểu trả về là `Driver?` và người gọi buộc phải xử lý null (null safety ở bài 02). Chú ý `Trip.driverId` cũng nullable vì chuyến đang tìm tài xế chưa có ai nhận.

`firstOnlineOrFallback` dùng `orElse` trả tài xế dự phòng thay vì ném `StateError`. `firstWhereOrNull` của `package:collection` là lựa chọn gọn hơn khi "không có" là kết quả bình thường.

**Nhóm theo khoá** là thao tác hay gặp nhất với danh sách chuyến. Viết tay chỉ vài dòng, và nên làm một lần để hiểu:

```dart
/// Cùng kết quả với `byStatus` nhưng viết tay, để thấy groupBy làm gì.
Map<TripStatus, List<Trip>> groupByStatusManually(Iterable<Trip> trips) {
  final groups = <TripStatus, List<Trip>>{};
  for (final trip in trips) {
    groups.putIfAbsent(trip.status, () => []).add(trip);
  }
  return groups;
}
```

`putIfAbsent` tạo List rỗng cho khoá mới, rồi `add` chuyến vào. `groupBy` của `package:collection` làm đúng việc đó; test `groupBy của package:collection khớp bản viết tay` kiểm hai bản cho cùng kết quả.

**Sắp xếp** cần nói riêng vì có bẫy về tính ổn định. Sắp xếp **ổn định** nghĩa là hai phần tử có khoá bằng nhau giữ nguyên thứ tự ban đầu. Với cước, hai chuyến cùng 12.000 VND nên giữ thứ tự theo lúc đặt. `sortedByCompare` (và `sortedBy`) của `package:collection` được cài đặt bằng `mergeSortBy` (merge sort, ổn định); lab có test 200 chuyến cước trùng để giữ hành vi này. `sorted` của cùng package chỉ sao chép rồi gọi `List.sort`, mà `List.sort` của Dart không cam kết ổn định. Vì vậy `sortedByFareDesc` dùng `sortedByCompare` với bộ so sánh ngược `b.compareTo(a)`. Đảo bằng `.reversed` sẽ đảo luôn thứ tự các phần tử bằng nhau.

## Generics: viết một lần, đúng kiểu cho mọi phần tử

Cần phân trang cho chuyến, cho tài xế, rồi cho ưu đãi. Viết ba lớp riêng thì lặp code; viết một lớp chứa `List<Object>` hoặc `dynamic` thì mất kiểu, và `page.items[0].fareVnd` trở thành lỗi chỉ bắt được lúc chạy. **Generics** cho tham số kiểu `T`: mỗi nơi dùng chọn `T` và trình biên dịch kiểm tra đúng kiểu.

### paged.dart · 04-collections-generics/lib/paged.dart

```dart
/// Một trang kết quả của kiểu bất kỳ. `page` đếm từ 1.
class Paged<T> {
  const Paged({
    required this.items,
    required this.page,
    required this.pageSize,
    required this.total,
  });

  final List<T> items;
  final int page;
  final int pageSize;

  /// Tổng số phần tử của toàn bộ danh sách, không phải của trang này.
  final int total;

  /// Còn trang sau khi số phần tử đã lấy (tới hết trang này) chưa đủ total.
  bool get hasNext => page * pageSize < total;

  /// Đổi kiểu phần tử, giữ nguyên thông tin phân trang.
  Paged<R> map<R>(R Function(T item) transform) => Paged<R>(
    items: List.unmodifiable(items.map(transform)),
    page: page,
    pageSize: pageSize,
    total: total,
  );

  @override
  String toString() => 'Paged(page $page, ${items.length}/$total items)';
}

/// Cắt một trang từ danh sách đầy đủ. Trang vượt cuối cho items rỗng.
Paged<T> paginate<T>(List<T> all, {required int page, required int pageSize}) {
  if (page < 1) throw ArgumentError.value(page, 'page', 'must be >= 1');
  if (pageSize < 1) {
    throw ArgumentError.value(pageSize, 'pageSize', 'must be >= 1');
  }
  final items = all.skip((page - 1) * pageSize).take(pageSize);
  return Paged<T>(
    items: List.unmodifiable(items),
    page: page,
    pageSize: pageSize,
    total: all.length,
  );
}

/// Phần tử có khoá lớn nhất. `K extends Comparable<K>` ràng buộc kiểu khoá:
/// compareTo có sẵn, không cần ép kiểu hay dynamic. String, DateTime, num dùng
/// được. Lưu ý `int` là `Comparable<num>`, nên viết tường minh `K = int` bị
/// từ chối; để Dart suy luận (K = num) hoặc dùng khoá String/DateTime.
T? highestBy<T, K extends Comparable<K>>(
  Iterable<T> items,
  K Function(T item) keyOf,
) {
  T? best;
  K? bestKey;
  for (final item in items) {
    final key = keyOf(item);
    if (bestKey == null || key.compareTo(bestKey) > 0) {
      best = item;
      bestKey = key;
    }
  }
  return best;
}
```

- `Paged<T>` mang `List<T>` và thông tin trang. `page` đếm từ 1; `hasNext` là `page * pageSize < total`, nên trang cuối vừa khít (8 phần tử, cỡ 4, trang 2) cho `false`, không sinh trang rỗng.
- `map<R>` có **tham số kiểu riêng** của phương thức: đổi `Paged<Trip>` thành `Paged<String>` mà giữ `page`, `pageSize`, `total`. Dart suy luận `R` từ hàm truyền vào, không cần viết `map<String>(...)`.
- `paginate<T>` là hàm generic: `paginate(trips, ...)` suy ra `T = Trip` từ đối số. Trang vượt cuối cho `items` rỗng thay vì ném lỗi, còn `page < 1` hoặc `pageSize < 1` là lỗi lập trình nên ném `ArgumentError`.
- `highestBy<T, K extends Comparable<K>>` dùng **ràng buộc** `extends`: khoá `K` phải so sánh được với chính nó, nên gọi `key.compareTo(bestKey)` an toàn. Truyền khoá là `Driver` (không so sánh được) sẽ báo lỗi ngay khi biên dịch. Lưu ý `int` là `Comparable<num>`, không phải `Comparable<int>`: viết tường minh `K = int` bị từ chối; để Dart suy luận (`K = num`) hoặc dùng khoá `String`, `DateTime` thì chạy.

Hai kiểu "rộng" này hỏng theo hai cách khác nhau. Với `dynamic`, trình biên dịch ngừng kiểm tra: gõ nhầm `fareVND` vẫn biên dịch, chỉ hỏng khi người dùng mở màn hình. Với `Object`, trình biên dịch vẫn kiểm tra, nhưng phải ép kiểu (`as Trip`) mới đọc được `fareVnd`, và ép sai mới hỏng lúc chạy. List của Dart còn **covariant**: `List<Trip>` gán được cho `List<Object>`, nhưng thêm một `int` vào đó thì ném lỗi lúc chạy. Kiểu cụ thể hoặc tham số kiểu an toàn hơn cả hai.

## Extension method: thêm hành vi cho Iterable<Trip>

`Trip` thuộc về ta, nhưng `Iterable<Trip>` thì không: ta không sửa được `Iterable`, và không nên tạo `class TripList extends ListBase<Trip>` chỉ để có thêm vài phương thức. **Extension** gắn thêm getter và phương thức lên một kiểu có sẵn, mà không kế thừa hay bọc.

### trip_queries.dart · 04-collections-generics/lib/trip_queries.dart

```dart
import 'package:collection/collection.dart' show IterableExtension, groupBy;

import 'model.dart';

/// Thêm truy vấn nghiệp vụ cho mọi `Iterable<Trip>` mà không kế thừa gì.
extension TripIterableX on Iterable<Trip> {
  /// Tổng cước bằng fold: bắt đầu từ 0, cộng dồn từng chuyến.
  int get totalFareVnd => fold(0, (sum, trip) => sum + trip.fareVnd);

  /// Nhóm theo trạng thái. Dùng groupBy của package:collection.
  Map<TripStatus, List<Trip>> get byStatus =>
      groupBy(this, (trip) => trip.status);

  /// Chuyến chưa kết thúc. Trả Iterable lười: chưa lọc gì cho tới khi duyệt.
  Iterable<Trip> get active => where((trip) => !trip.status.isTerminal);

  /// Tất cả đã kết thúc (every). Iterable rỗng cho true.
  bool get allSettled => every((trip) => trip.status.isTerminal);

  /// Chuyến cước cao nhất (reduce). Rỗng thì reduce ném StateError, nên chặn trước.
  Trip? get priciest => isEmpty
      ? null
      : reduce((best, trip) => trip.fareVnd > best.fareVnd ? trip : best);

  /// Cước giảm dần, giữ thứ tự ban đầu khi cước bằng nhau (sắp xếp ổn định).
  ///
  /// sortedByCompare được cài đặt bằng mergeSortBy (merge sort, ổn định); test
  /// 200 chuyến cước trùng nhau giữ hành vi này. Còn `sorted` của
  /// package:collection gọi List.sort, không cam kết ổn định.
  List<Trip> get sortedByFareDesc =>
      sortedByCompare((trip) => trip.fareVnd, (a, b) => b.compareTo(a));
}
```

Hàm `groupByStatusManually` ở cuối file đã in ở mục trước nên không lặp lại ở đây.

Vì extension nhắm vào `Iterable<Trip>`, nó dùng được cho `List<Trip>`, `Set<Trip>` và cả pipeline lười (test `extension chạy trên Set và trên pipeline lười` kiểm điều này). Có hai điều phải nhớ. Thứ nhất, extension được chọn theo **kiểu tĩnh** lúc biên dịch: gọi trên giá trị kiểu `dynamic` thì không tìm thấy. Thứ hai, extension không ghi đè được member có sẵn của kiểu: nếu `Iterable` có `length`, ta không thay nó được.

`active` trả `Iterable<Trip>` lười, nên `trips.active.totalFareVnd` nối được hai extension với nhau mà không tạo List trung gian. Còn `sortedByFareDesc` trả `List` vì sắp xếp phải duyệt hết và người gọi thường cần chỉ số.

Dòng import đầu file chỉ lấy hai tên bằng `show`. `show` và `hide` cũng là công cụ để xử lý khi hai extension trùng tên member. Lab có hai extension, `TripSummaryShort` và `TripSummaryLong`, đều có getter `summary` trên `Iterable<Trip>`:

### summary_short.dart · 04-collections-generics/lib/summary_short.dart

```dart
import 'model.dart';

/// Số chuyến kèm đơn vị. Có thêm hàm này để `hide TripSummaryShort` vẫn còn
/// một tên khác cần import từ file.
String pluralTrips(int count) => '$count chuyến';

/// Hai extension khác tên nhưng cùng member `summary`: ví dụ xung đột.
extension TripSummaryShort on Iterable<Trip> {
  String get summary => pluralTrips(length);
}
```

### summary_long.dart · 04-collections-generics/lib/summary_long.dart

```dart
import 'model.dart';
import 'trip_queries.dart';

extension TripSummaryLong on Iterable<Trip> {
  String get summary => '$length chuyến, tổng $totalFareVnd VND';
}
```

Nếu một file import cả hai rồi viết `trips.summary`, analyzer báo `ambiguous_extension_member_access`: không có extension nào cụ thể hơn. Có hai cách gỡ, và cả hai đều có test:

### extension_conflict_test.dart · 04-collections-generics/test/extension_conflict_test.dart

```dart
import 'package:test/test.dart';
import 'package:toi_noi_04/model.dart';
// Import cả hai: gọi `trips.summary` sẽ là lỗi biên dịch vì không biết chọn
// TripSummaryShort hay TripSummaryLong. Áp extension tường minh để chọn.
import 'package:toi_noi_04/summary_long.dart';
import 'package:toi_noi_04/summary_short.dart';

import 'fixtures.dart';

void main() {
  test('áp extension tường minh để giải xung đột tên member', () {
    final trips = <Trip>[
      makeTrip('T1', TripStatus.completed, 25500),
      makeTrip('T2', TripStatus.searching, 12000),
    ];
    expect(TripSummaryShort(trips).summary, '2 chuyến');
    expect(TripSummaryLong(trips).summary, '2 chuyến, tổng 37500 VND');
  });
}
```

### extension_hide_test.dart · 04-collections-generics/test/extension_hide_test.dart

```dart
import 'package:test/test.dart';
import 'package:toi_noi_04/model.dart';
import 'package:toi_noi_04/summary_long.dart';
// hide loại extension không dùng: `summary` chỉ còn một nghĩa, mà vẫn dùng được
// pluralTrips từ cùng file.
import 'package:toi_noi_04/summary_short.dart' hide TripSummaryShort;

import 'fixtures.dart';

void main() {
  test('hide một extension thì trips.summary dùng extension còn lại', () {
    final trips = <Trip>[makeTrip('T1', TripStatus.completed, 25500)];
    expect(trips.summary, '1 chuyến, tổng 25500 VND');
    expect(pluralTrips(trips.length), '1 chuyến');
  });
}
```

Cách đầu **áp extension tường minh** `TripSummaryShort(trips).summary`, khi cần cả hai. Cách sau **`hide`** extension không dùng (hoặc `show` đúng tên cần) ở dòng import, khi chỉ cần một. `hide TripSummaryShort` chỉ che extension, hàm `pluralTrips` cùng file vẫn dùng được; `hide` hữu ích khi file còn export thứ khác bạn cần. Đặt tên member của extension đủ cụ thể (`totalFareVnd`, không phải `total`) để ít va chạm.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp với collection</span>
<ul>
<li><b>Sửa list khi đang duyệt.</b> Dấu hiệu: <code>ConcurrentModificationError</code> ở lần lặp kế tiếp của <code>for-in</code>. Nguyên nhân: <code>removeCompletedWrong</code> gọi <code>remove</code> trong khi <code>for</code> đang đi trên cùng List; test <code>sửa list đang duyệt ném ConcurrentModificationError</code> tái hiện. Cách sửa: dùng <code>removeWhere</code> như <code>removeCompleted</code>, hoặc duyệt trên bản sao <code>trips.toList()</code>, hoặc dựng List mới bằng <code>where</code>.</li>
<li><b>Nhầm <code>List.from</code> với <code>toList</code>.</b> Dấu hiệu: không có kiểu ngữ cảnh, <code>List.from(trips)</code> cho ra <code>List&lt;dynamic&gt;</code>, mất kiểu <code>Trip</code>; gán sang <code>List&lt;Trip&gt;</code> là lỗi biên dịch, còn ép bằng <code>as</code> thì thành lỗi lúc chạy. Cách sửa: dùng <code>trips.toList()</code> hoặc <code>List.of(trips)</code> để giữ kiểu <code>Trip</code>.</li>
<li><b><code>==</code> của List so sánh tham chiếu.</b> Dấu hiệu: <code>[1, 2, 3] == [1, 2, 3]</code> là <code>false</code>, nên so danh sách chuyến bằng <code>==</code> trong code chính hoặc làm key của Map đều sai. Cách sửa: dùng <code>ListEquality</code> hoặc <code>DeepCollectionEquality</code> của <code>package:collection</code>; trong test dùng <code>expect(a, equals(b))</code>, matcher so sánh sâu.</li>
<li><b><code>sort</code> sửa tại chỗ và trả <code>void</code>.</b> Dấu hiệu: danh sách gốc bị đổi thứ tự bất ngờ, hoặc viết <code>final sorted = list.sort()</code> rồi thấy kiểu <code>void</code>. Cách sửa: muốn bản mới thì <code>[...list]..sort()</code> hoặc <code>sortedBy</code>; test <code>sort sửa tại chỗ, sorted trả list mới</code> minh hoạ.</li>
</ul>
</div>

```dart
/// Sai: sửa list đang duyệt, lần lặp kế tiếp ném ConcurrentModificationError.
void removeCompletedWrong(List<Trip> trips) {
  for (final trip in trips) {
    if (trip.status == TripStatus.completed) trips.remove(trip);
  }
}

/// Đúng: removeWhere sửa tại chỗ an toàn, không cần vòng for.
void removeCompleted(List<Trip> trips) =>
    trips.removeWhere((trip) => trip.status == TripStatus.completed);
```

Lưu pipeline lười trên một List còn đổi để dùng sau cũng nguy hiểm: kết quả đổi theo nguồn mà không ai nhận ra. Cần giá trị tại một thời điểm thì chốt bằng `toList()`.

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/04-collections-generics
dart pub get
dart run bin/main.dart
dart analyze
dart test
```

`bin/main.dart` dựng tám chuyến hư cấu (số liệu cước theo bảng giá minh hoạ, không phải giá thật) và năm tài xế hư cấu. Đầu ra rút gọn:

```text
Tổng cước 229650 VND (số liệu minh hoạ)
Chưa kết thúc: 5 chuyến, 180150 VND

Nhóm theo trạng thái:
  completed: TN-1, TN-6
  inTransit: TN-2
  ...
Trang 1 (hasNext: true):
  TN-1 completed
  TN-2 inTransit
  TN-3 accepted

Thành phố có tài xế online: Hà Nội, Đà Nẵng, TP.HCM
```

Mức kiểm chứng: **Kiểm chứng tự động**. `dart analyze` báo 0 issue và `dart test` chạy 46 test xanh; không cần thiết bị hay simulator. Bảng dưới cho biết test nào chứng minh điều gì.

| Test | Chứng minh |
|---|---|
| `trip_queries_test.dart` | Tổng cước (kể cả rỗng), nhóm theo trạng thái, `active`, sắp xếp ổn định trên 200 chuyến có cước trùng nhau |
| `paged_test.dart` | `hasNext` ở trang đầu, trang cuối lẻ (8 phần tử, cỡ 3), trang vừa khít, trang vượt cuối, `map` giữ thông tin trang |
| `lazy_demo_test.dart` | Bộ đếm = 0 trước khi duyệt, = n sau `toList()`, duyệt hai lần đếm hai lần, `first` dừng sớm, snapshot và pipeline lười; bốn lỗi thường gặp |
| `lookup_literals_test.dart` | Set loại trùng, Map thiếu khoá trả `null`, `firstWhere` có và không có `orElse`, collection-if/for và `const` |
| `extension_conflict_test.dart`, `extension_hide_test.dart` | Xung đột tên member giữa hai extension và hai cách giải |

Chưa làm trong lab: phân trang theo cursor và gọi API thật (để dành cho các bài về dữ liệu và mạng), `TripEvent` và `Money` (chưa cần ở bài này), đo hiệu năng collection với dữ liệu lớn. Các con số cước và thời gian trong bài đều là minh hoạ.

## Bài tập

1. Viết getter `cancellable` trên `Iterable<Trip>` trả các chuyến có `status.canCancel`. Nên trả `Iterable<Trip>` hay `List<Trip>`? Vì sao?
2. `Paged<T>` đang đếm trang từ 1. Với `page: 2, pageSize: 5, total: 10`, `hasNext` là gì? Nếu đổi sang đếm từ 0, công thức `hasNext` phải sửa thế nào?
3. Với `final ids = QueryProbe().activeIds(fiveTrips())` (năm chuyến mẫu), gọi `ids.first` rồi `ids.toList()`. Sau hai lệnh, `predicateCalls` và `projectionCalls` bằng bao nhiêu?

<details><summary>Xem lời giải</summary>
<p>1. <code>Iterable&lt;Trip&gt; get cancellable =&gt; where((trip) =&gt; trip.status.canCancel);</code>. Trả <code>Iterable</code> lười là hợp lý vì người gọi thường nối tiếp (<code>.length</code>, <code>.take(3)</code>, <code>.totalFareVnd</code>) và không cần List trung gian. Trả <code>List</code> (thêm <code>.toList()</code>) khi kết quả cần giữ lâu hoặc nguồn có thể đổi.</p>
<p>2. <code>2 * 5 &lt; 10</code> là <code>false</code>, nên không còn trang sau. Nếu <code>page</code> đếm từ 0, trang đã lấy xong là <code>(page + 1) * pageSize</code>, nên <code>hasNext</code> thành <code>(page + 1) * pageSize &lt; total</code>, và <code>paginate</code> bỏ qua <code>page * pageSize</code> phần tử thay vì <code>(page - 1) * pageSize</code>.</p>
<p>3. <code>first</code> dừng ở chuyến thứ hai: <code>predicateCalls = 2</code>, <code>projectionCalls = 1</code>. <code>toList()</code> duyệt lại từ đầu thêm 5 lần lọc và 2 lần biến đổi. Tổng: <code>predicateCalls = 7</code>, <code>projectionCalls = 3</code>. Test <code>first rồi toList</code> kiểm đúng các con số này.</p>
</details>

Bài trước: [Hàm, class, enum, sealed class và pattern matching](/articles/dart-ham-class-enum-sealed-pattern-matching-trang-thai-chuyen). Bài tiếp: [Future, async/await và Stream](/articles/dart-future-async-await-stream-bao-gia-va-vi-tri).

## Tài liệu gốc

- [Collections trong Dart](https://dart.dev/language/collections): List, Set, Map, collection-if/for và spread.
- [Iterable collections](https://dart.dev/libraries/collections/iterables): `where`, `map`, `fold`, `toList` và tính lười.
- [Generics](https://dart.dev/language/generics): tham số kiểu, ràng buộc `extends`.
- [Extension methods](https://dart.dev/language/extension-methods): xung đột và extension override.
- [package:collection](https://pub.dev/packages/collection): `groupBy`, `sortedBy`, `firstWhereOrNull`, `DeepCollectionEquality`.
- [Iterable (API)](https://api.flutter.dev/flutter/dart-core/Iterable-class.html) và [List.sort](https://api.flutter.dev/flutter/dart-core/List/sort.html): hành vi và đảm bảo ổn định.
