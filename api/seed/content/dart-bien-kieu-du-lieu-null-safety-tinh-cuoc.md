---
slug: dart-bien-kieu-du-lieu-null-safety-tinh-cuoc
title: "Dart cơ bản: biến, kiểu dữ liệu và null safety khi tính cước Tới Nơi"
summary: "Bài 02: chọn kiểu cho tiền VND, khoảng cách và trạng thái; hiểu var, final, const, late, phép chia và làm tròn; dùng null safety để phân biệt chưa có tài xế với dữ liệu lỗi. Có bộ test ca biên chạy bằng dart test."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, null-safety]
language: vi
---

Một khách vừa đặt xe trên [Tới Nơi](/articles/du-an-mau-toi-noi). Màn hình báo giá đã có quãng đường và số tiền, nhưng ô "Tài xế" còn trống vì chưa ai nhận chuyến. Ô trống đó có hai cách hiểu: chưa tìm được tài xế, hoặc dữ liệu từ server bị hỏng. Nếu code không phân biệt được, app sẽ hiện chữ "null" cho khách, hoặc crash ngay trên màn hình.

Đây là bài 02 của lộ trình **Flutter qua Tới Nơi**. Bài này tiếp nối [bài 01](/articles/flutter-va-dart-tu-source-code-den-ung-dung-toi-noi), nơi ta chạy app in cước 5 km. Hôm nay ta chưa đụng tới widget, chỉ dùng Dart thuần để trả lời: tiền nên là kiểu gì, `var`/`final`/`const`/`late` khác nhau ở đâu, và null safety giúp viết hợp đồng cho dữ liệu "có thể chưa có" thế nào.

Mọi con số giá trong bài là **minh hoạ**: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt, miền hợp lệ 0 đến 100 km. Đây không phải bảng giá thực tế của sản phẩm nào.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta chặn khoảng cách sai bằng <code>if (km &lt; 0 || km &gt; 100) throw ...</code>. Nếu <code>km</code> là <code>double.nan</code>, câu lệnh này có ném lỗi không?</p>
<details><summary>Xem đáp án</summary><p><b>Không.</b> NaN so sánh với số nào cũng cho <code>false</code>, kể cả so với chính nó. Cả hai vế <code>km &lt; 0</code> và <code>km &gt; 100</code> đều sai, nên NaN lọt qua guard. Cách viết đúng là nêu điều kiện <i>hợp lệ</i> rồi phủ định: <code>!(km &gt;= 0 &amp;&amp; km &lt;= 100)</code>. Test ở các phần sau chứng minh điều này.</p></details>
</div>

## Chọn kiểu cho một chuyến

Dart có kiểu tĩnh. Một chuyến Tới Nơi cần vài loại dữ liệu cơ bản:

| Dữ liệu | Kiểu | Vì sao |
| --- | --- | --- |
| Số tiền `fareVnd` | `int` | VND không có phần lẻ, lưu số đồng nguyên |
| Quãng đường nhập liệu `distanceKm` | `double` | km có phần thập phân |
| Quãng đường lưu và truyền `distanceMeters` | `int` | mét nguyên, không dính sai số nhị phân |
| Mã chuyến, tên tài xế | `String` | văn bản |
| Đã có tài xế chưa | `bool` | chỉ có `true` hoặc `false` |
| Trạng thái chuyến | `enum` (bài 03) | tập giá trị cố định, không dùng chuỗi |

`int` và `double` cùng là con của `num`. Trộn hai loại, như `12000 + 4.5`, cho ra `double`. Trong code nghiệp vụ hãy khai báo `int` hoặc `double` cụ thể, chỉ dùng `num` khi hàm thật sự nhận cả hai.

Dart không có kiểu thập phân chính xác trong thư viện lõi. Bài C# tương ứng ([biến, kiểu dữ liệu và decimal](/articles/csharp-bien-kieu-du-lieu-decimal)) dùng `decimal` cho tiền. Ở Dart ta chọn đường khác: lưu tiền bằng `int` đồng, và làm tròn đúng một lần theo quy tắc rõ ràng. Cách này hợp với VND vì đồng đã là đơn vị nhỏ nhất. Kiểu `Money` bọc số đồng sẽ có ở bài 06.

Trên iOS và Android (Dart native), `int` là số nguyên có dấu 64 bit, khoảng ±9,2 × 10¹⁸, đủ rộng cho tiền của một chuyến. Tràn số không ném lỗi mà quay vòng:

```dart
test('tràn số quay vòng quanh, không ném lỗi', () {
  const maxInt = 9223372036854775807;

  expect(maxInt + 1, -9223372036854775808);
});
```

Khi biên dịch sang web, `int` có giới hạn khác; app Tới Nơi chạy trên mobile nên ta không xét nhánh đó.

Chuỗi nội suy dùng `$`: `'$name'` chèn biến, `'${a + b}'` chèn biểu thức.

## var, final, const và late

`var` không làm Dart thành ngôn ngữ kiểu động. Nó nhờ compiler suy ra kiểu từ giá trị khởi tạo, rồi kiểu đó cố định. Đoạn dưới là `main.dart` của lab:

### main.dart · 02-dart-bien-kieu-null-safety/bin/main.dart

```dart
import 'package:toi_noi_02/fare.dart';
import 'package:toi_noi_02/quote.dart';

void main() {
  const distanceKm = 5.0; // const: biết từ lúc biên dịch
  final accepted = Quote.fromDistance(distanceKm, driverName: 'Lê Minh Châu');
  print('Đã có tài xế : ${accepted.describe()}');

  var quote = Quote.fromDistance(2.3); // var: suy ra kiểu Quote
  print('Chưa có tài xế: ${quote.describe()}');

  // Gán lại biến sang bản sao mới; đối tượng cũ không đổi.
  quote = quote.withDriver('Trần Văn An');
  // quote = 5; // không biên dịch: var đã chốt kiểu Quote
  print('Sau khi nhận  : ${quote.describe()}');

  print('5 / 2 = ${5 / 2}, 5 ~/ 2 = ${5 ~/ 2}');
  print(formatVnd(calculateFareVnd(100)));
}
```

Bốn từ khoá trả lời bốn câu hỏi khác nhau:

- **`var`**: biến gán lại được, kiểu do compiler suy ra. Dùng khi giá trị đổi.
- **`final`**: gán đúng một lần, giá trị có thể chỉ biết lúc chạy (ví dụ kết quả `Quote.fromDistance`). Sau đó biến không trỏ sang giá trị khác. Lưu ý `final` không làm đối tượng bất biến: ta phải tự thiết kế lớp như vậy, như `Quote` có toàn trường `final` và `withDriver` trả bản sao.
- **`const`**: hằng biết từ lúc biên dịch, như `5.0` hay các hằng giá ở đầu `fare.dart`. `DateTime.now()` không thể là `const` vì chỉ có giá trị khi chạy. `const [1, 2]` là danh sách bất biến sâu; `final list = [1, 2]` chỉ khoá biến, ta vẫn `add` được.
- **`late`**: hứa với compiler rằng biến sẽ có giá trị trước khi đọc, dù lúc khai báo chưa gán.

Quy tắc chọn đơn giản: ưu tiên `final` cho biến cục bộ, `const` cho hằng thật sự, `var` khi buộc phải gán lại. Lint `prefer_final_locals` nhắc đổi `var` thành `final` khi biến không bị gán lại; nó không kiểm `const`.

`late` là từ khoá cần dè dặt nhất, vì nó chuyển một lỗi biên dịch thành lỗi lúc chạy. Có hai cách dùng. Cách nguy hiểm là `late String id;` không có giá trị khởi tạo: nếu đọc trước khi gán, chương trình ném lỗi `has not been initialized`. Cách an toàn hơn là `late final` kèm initializer, như `Quote.fareText` ở phần null safety: giá trị chỉ được tính khi đọc lần đầu, đúng một lần. Test trong lab kiểm chứng cả hai:

```dart
class _Draft {
  late String id; // không có giá trị khởi tạo: đọc trước khi gán sẽ ném lỗi
}

test('đọc late chưa gán ném lỗi "has not been initialized"', () {
  final draft = _Draft();

  expect(
    () => draft.id,
    throwsA(
      isA<Error>().having(
        (e) => e.toString(),
        'message',
        contains('has not been initialized'),
      ),
    ),
  );
});
```

Nếu truyền được giá trị qua constructor thì đừng dùng `late`.

## Phép tính: chia, làm tròn và những giá trị lạ của double

Phép `/` trong Dart luôn trả `double`, kể cả khi hai vế là số nguyên. Muốn chia lấy phần nguyên, dùng `~/`. Phần dư là `%`. Những khẳng định này đều có test trong `test/basics_test.dart`:

```dart
test('/ luôn trả double, ~/ chia lấy phần nguyên', () {
  expect(5 / 2, 2.5);
  expect(4 / 2, 2.0);
  expect(5 ~/ 2, 2);
  expect((5 / 2).runtimeType, double);
  expect((5 ~/ 2).runtimeType, int);
});

test('~/ cắt về phía 0, % luôn không âm với số chia dương', () {
  expect(-5 ~/ 2, -2);
  expect(-5 % 3, 1);
  expect((-5).remainder(3), -2);
});
```

Khác với C#, `5 / 2` ở Dart không bị "chia nguyên ngầm": nó ra 2,5, và ta phải chủ động viết `~/` khi muốn số nguyên. Chia nguyên cho 0 bằng `~/` ném lỗi, còn `1 / 0` cho ra `Infinity`.

Làm tròn dùng `round()`: điểm giữa làm tròn **ra xa số 0**, nên 2,5 thành 3 và -2,5 thành -3. Còn `toStringAsFixed(1)` chỉ tạo chuỗi để hiển thị, giá trị gốc không đổi:

```dart
test('round() làm tròn điểm giữa ra xa số 0', () {
  expect(2.5.round(), 3);
  expect((-2.5).round(), -3);
  expect(2.4.round(), 2);
});

test('toStringAsFixed chỉ định dạng, không đổi giá trị', () {
  const km = 2.567;

  expect(km.toStringAsFixed(1), '2.6');
  expect(km, 2.567);
});
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 180" role="img" aria-label="Hình 1. Phân rã báo giá 5 km và kiểu dữ liệu ở từng bước: hai km đầu 12.000 là int, ba km vượt nhân 4.500 là double, cước thô 25.500,0 là double, sau round() là int 25.500."><rect x="20" y="45" width="186" height="100" rx="10" class="f-phys-soft s-phys"/><text x="32" y="82" class="t-b">2 km đầu</text><text x="32" y="113" class="t-small">12.000 (int)</text><text x="210" y="100" class="t-b">→</text><rect x="230" y="45" width="186" height="100" rx="10" class="f-phys-soft s-phys"/><text x="242" y="82" class="t-b">3 km vượt</text><text x="242" y="113" class="t-small">3,0 × 4.500 (double)</text><text x="420" y="100" class="t-b">→</text><rect x="440" y="45" width="186" height="100" rx="10" class="f-phys-soft s-phys"/><text x="452" y="82" class="t-b">Cước thô</text><text x="452" y="113" class="t-small">25.500,0 (double)</text><text x="630" y="100" class="t-b">→</text><rect x="650" y="45" width="186" height="100" rx="10" class="f-pk-soft s-pk"/><text x="662" y="82" class="t-b">round()</text><text x="662" y="113" class="t-small">25.500 (int)</text></svg>
</div>
<figcaption>Hình 1. Phân rã báo giá 5 km theo cách <code>double</code>, ghi kiểu ở từng bước: phép nhân với km lẻ buộc cước thô thành double, <code>round()</code> đưa về int ở cuối. Phần tính cước bên dưới sẽ thay bằng cách tính toàn int.</figcaption>
</figure>

`double` có hai cái bẫy.

**Sai số nhị phân.** `double` lưu số theo cơ số 2 nên `0.1 + 0.2` không đúng bằng `0.3`. Khoảng cách 2,001 km trừ 2 cũng không ra đúng 0,001. Vì vậy không so sánh `double` bằng `==` khi kết quả đã qua phép tính:

```dart
test('0.1 + 0.2 không bằng 0.3 vì nhị phân không biểu diễn chính xác', () {
  expect(0.1 + 0.2 == 0.3, isFalse);
  expect((0.1 + 0.2 - 0.3).abs() < 1e-9, isTrue);
});

test('2.001 - 2 không đúng bằng 0.001', () {
  expect(2.001 - 2 == 0.001, isFalse);
});
```

Điều này ảnh hưởng thật tới cước. Với 2,001 km, phần vượt lý thuyết là 4,5 đồng, nhưng `(2.001 - 2) * 4500` ra 4,499999999999504. Làm tròn riêng phần vượt bằng `double` cho 4, và cước thành 12.004 thay vì 12.005:

```dart
test('làm tròn riêng phần vượt bằng double cho 12004 thay vì 12005', () {
  final extraFare = (2.001 - 2) * 4500; // lý thuyết là 4,5 đồng

  expect(extraFare, lessThan(4.5));
  expect(12000 + extraFare.round(), 12004);
});
```

Ta sẽ tránh điều đó ở phần tính cước bằng cách đổi km sang mét nguyên.

**NaN và Infinity.** `double.nan` không bằng chính nó, và mọi so sánh thứ tự với nó đều `false`:

```dart
test('NaN không bằng chính nó và mọi so sánh thứ tự đều false', () {
  const nan = double.nan;

  expect(nan == nan, isFalse);
  expect(nan >= 0, isFalse);
  expect(nan < 0, isFalse);
  expect(nan.isNaN, isTrue);
});
```

Đó là lý do câu "Thử trước khi đọc" có đáp án là "không". NaN có thể đến từ `0 / 0` hoặc từ JSON sai, nên guard phải viết sao cho **NaN rơi vào nhánh từ chối**. Còn `double.nan.round()` ném `UnsupportedError`, nên guard cũng phải đứng trước `round()`.

## Null safety: null là một phần của hợp đồng

Từ Dart 3, null safety luôn bật. Quy tắc cốt lõi: kiểu bình thường **không chứa null**. `String` luôn là một chuỗi thật. Muốn cho phép "chưa có", ta thêm `?`: `String?` là chuỗi hoặc null. Compiler từ chối cho bạn gọi `.length` trên `String?` cho tới khi chắc chắn nó không null.

Vì sao `Quote` có `driverName` nullable còn `fareVnd` thì không? Vì đó là hợp đồng nghiệp vụ. Báo giá nào cũng có giá, nên `int fareVnd` không bao giờ null. Còn tài xế chỉ có sau khi ai đó nhận chuyến, nên `String? driverName` mang đúng ý "chưa có". 

### quote.dart · 02-dart-bien-kieu-null-safety/lib/quote.dart

```dart
import 'fare.dart';

/// Một báo giá: khoảng cách, cước và (có thể chưa có) tài xế.
class Quote {
  Quote({required this.distanceKm, required this.fareVnd, this.driverName}) {
    // Public final field không được promote, nên chép ra biến cục bộ trước.
    final name = driverName;
    if (name != null && name.trim().isEmpty) {
      throw ArgumentError.value(
        name,
        'driverName',
        'dùng null khi chưa có tài xế, không dùng chuỗi rỗng',
      );
    }
  }

  /// Tính cước từ khoảng cách rồi tạo báo giá.
  factory Quote.fromDistance(double distanceKm, {String? driverName}) {
    return Quote(
      distanceKm: distanceKm,
      fareVnd: calculateFareVnd(distanceKm),
      driverName: driverName,
    );
  }

  final double distanceKm;
  final int fareVnd; // không nullable: báo giá luôn có giá
  final String? driverName; // nullable: null nghĩa là chưa có tài xế

  /// Chuỗi tiền chỉ được tạo khi cần, đúng một lần; sau đó là hằng của đối tượng.
  late final String fareText = formatVnd(fareVnd);

  /// Trả bản sao đã có tài xế; đối tượng cũ giữ nguyên.
  Quote withDriver(String name) =>
      Quote(distanceKm: distanceKm, fareVnd: fareVnd, driverName: name);

  String describe() {
    // ?. : nếu driverName là null thì cả biểu thức là null, không ném lỗi.
    final givenName = driverName?.split(' ').last;
    // ?? : thay null bằng giá trị mặc định.
    final driver = givenName ?? 'chưa có tài xế';
    return '${distanceKm.toStringAsFixed(1)} km | $fareText | $driver';
  }
}
```

Bốn toán tử và từ khoá làm việc với giá trị có thể null:

- **`required`** buộc người gọi truyền tham số đặt tên. Với tham số không nullable, quên truyền `fareVnd` là lỗi biên dịch, không phải lỗi lúc chạy. (`required` cũng dùng được cho tham số nullable, khi muốn người gọi tự nói rõ `null`.)
- **`?.`** gọi tiếp nếu giá trị không null, nếu không thì cả chuỗi gọi cho ra null. Trong `driverName?.split(' ').last`, khi `driverName` là null thì `split` và `last` không chạy.
- **`??`** cho giá trị dự phòng khi vế trái là null.
- **`!`** nói "tôi chắc giá trị này không null". Nếu sai, chương trình ném `TypeError` ngay lúc chạy. Chỉ dùng khi bạn chỉ ra được vì sao null không xảy ra.

Hai test sau chứng minh `!` ném lỗi, còn `?.` và `??` thì không:

```dart
test('dấu ! ném lỗi lúc chạy khi giá trị là null', () {
  expect(_lengthOf('Tới Nơi'), 7);
  expect(() => _lengthOf(null), throwsA(isA<TypeError>()));
});

test('?. và ?? trên giá trị null không ném lỗi', () {
  const String? name = null;

  expect(name?.length, isNull);
  expect(name?.length ?? 0, 0);
});
```

Công cụ quan trọng nhất không có ký hiệu: **flow analysis**, cơ chế cho phép type promotion. Sau khi bạn kiểm `name != null`, compiler hiểu rằng trong nhánh đó `name` là `String` thường, không cần `!`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Promotion của null safety: biến String? đi qua phép kiểm name khác null. Nhánh có giá trị cho name là String và dùng được trim, length. Nhánh null vẫn là Null, gọi length trực tiếp bị compiler chặn."><defs><marker id="fl02-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl02-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl02-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="105" width="235" height="90" rx="10" class="f-phys-soft s-phys"/><text x="34" y="138" class="t-b">driverName</text><text x="34" y="167" class="t-small">String? (có thể null)</text><rect x="300" y="105" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="314" y="138" class="t-b">Kiểm null</text><text x="314" y="167" class="t-mono t-small">name != null ?</text><rect x="545" y="20" width="315" height="100" rx="10" class="f-pk-soft s-pk"/><text x="559" y="55" class="t-b">Nhánh có giá trị</text><text x="559" y="85" class="t-small">name là String: .trim(), .length</text><rect x="545" y="180" width="315" height="100" rx="10" class="f-warn-soft s-warn"/><text x="559" y="215" class="t-b">Nhánh null</text><text x="559" y="245" class="t-small">name.length: compiler từ chối</text><path d="M255 150 L300 150" class="f-none s-phys" stroke-width="2" marker-end="url(#fl02-a)"/><path d="M490 130 L545 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl02-p)"/><path d="M490 170 L545 222" class="f-none s-warn" stroke-width="2" marker-end="url(#fl02-w)"/></svg>
</div>
<figcaption>Hình 2. Promotion: sau phép kiểm null, kiểu của biến được thu hẹp từ String? thành String ở nhánh có giá trị. Nhánh null phải được xử lý (dùng giá trị mặc định, báo lỗi hoặc thoát), nếu không compiler không cho gọi member của String.</figcaption>
</figure>

Promotion chỉ hoạt động trên biến mà compiler chắc chắn không bị đổi giữa lúc kiểm và lúc dùng: biến cục bộ, tham số, và trường `final` **private**. Trường public như `driverName` thì không, vì một lớp con có thể ghi đè getter. Đó là lý do constructor của `Quote` chép `driverName` ra biến cục bộ `name` trước khi kiểm. Hàm nhỏ sau trong test minh hoạ promotion:

```dart
String _labelOf(String? text) {
  if (text == null) {
    return 'chưa có';
  }
  // Tới đây text đã được promote từ String? thành String.
  return '$text (${text.length} ký tự)';
}
```

Constructor của `Quote` từ chối chuỗi rỗng bằng `ArgumentError`, vì ta quy ước: `null` nghĩa là "chưa có tài xế", còn `""` là dữ liệu hỏng. Đây là câu trả lời cho tình huống đầu bài: hai trạng thái khác nhau có hai biểu diễn khác nhau, và biểu diễn lỗi bị chặn ngay ở biên.

## Hàm tính cước và các ca biên

### fare.dart · 02-dart-bien-kieu-null-safety/lib/fare.dart

```dart
import 'dart:math' show max;

// Bảng giá minh hoạ của Tới Nơi, không phải bảng giá thực tế.
const int baseFareVnd = 12000; // đã gồm 2 km đầu
const int includedMeters = 2000;
const int pricePerExtraKmVnd = 4500;
const int maxDistanceKm = 100;
const int metersPerKm = 1000;

/// Tính cước (đồng VND) cho quãng đường [distanceKm], miền hợp lệ 0..100 km.
///
/// Ném [ArgumentError] khi ngoài miền, kể cả NaN. `round()` chỉ dùng một lần
/// ở biên km -> mét (độ phân giải 1 m); sau đó mọi phép tính là `int`.
int calculateFareVnd(double distanceKm) {
  // NaN so sánh với số nào cũng cho false, nên guard viết dưới dạng
  // "điều kiện hợp lệ" rồi phủ định. Guard phải đứng trước `round()` vì
  // NaN.round() và Infinity.round() ném UnsupportedError.
  final isValid = distanceKm >= 0 && distanceKm <= maxDistanceKm;
  if (!isValid) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0..$maxDistanceKm km',
    );
  }

  final meters = (distanceKm * metersPerKm).round();
  final extraMeters = max(0, meters - includedMeters);
  // (+ 500) ~/ 1000 là làm tròn nửa lên cho số không âm: 4,5 đồng -> 5 đồng.
  return baseFareVnd + (extraMeters * pricePerExtraKmVnd + 500) ~/ metersPerKm;
}

/// Định dạng số đồng với dấu chấm nghìn: 25500 -> "25.500 VND".
///
/// Chèn tay để chưa phải dùng package `intl` (bài về localization sẽ thay thế).
String formatVnd(int vnd) {
  final digits = vnd.abs().toString();
  final buffer = StringBuffer();
  for (var i = 0; i < digits.length; i++) {
    final remaining = digits.length - i;
    if (i > 0 && remaining % 3 == 0) {
      buffer.write('.');
    }
    buffer.write(digits[i]);
  }
  final sign = vnd < 0 ? '-' : '';
  return '$sign$buffer VND';
}
```

### Từ km sang mét nguyên

Cách `double` ở phần trước chỉ để học kiểu. Dạng cuối trong `fare.dart` đổi km sang mét ở biên rồi chỉ dùng `int`:

```dart
final meters = (distanceKm * metersPerKm).round();
final extraMeters = max(0, meters - includedMeters);
return baseFareVnd + (extraMeters * pricePerExtraKmVnd + 500) ~/ metersPerKm;
```

- `round()` chỉ xuất hiện một lần, ở biên km sang mét (nửa làm tròn ra xa số 0). Đầu vào có độ phân giải 1 m: 2,0004 km là 2.000 m, 2,0006 km là 2.001 m.
- Sau đó toàn `int`, không còn sai số nhị phân. Mỗi mét vượt giá 4,5 đồng; cộng 500 rồi `~/ 1000` là làm tròn nửa lên cho số không âm: 1 m vượt cho (4.500 + 500) `~/` 1000 = 5 đồng.
- Guard đứng trước `round()`, vì `NaN.round()` và `Infinity.round()` ném `UnsupportedError`.

Các hằng giá là `const` ở mức file, và `max(0, meters - includedMeters)` bảo đảm đi 1 km thì phần vượt bằng 0 thay vì âm.

`formatVnd` đếm từ **cuối** chuỗi: dấu chấm được viết trước chữ số nào còn lại số chữ số chia hết cho 3. Đếm từ đầu sẽ đặt sai chỗ với số 4, 5 chữ số. Lab chưa dùng `intl`; định dạng theo locale để dành cho bài 14.

Các ca biên trong `test/fare_test.dart`:

| Đầu vào (km) | Kết quả mong đợi | Chứng minh điều gì |
| --- | --- | --- |
| 0 | 12.000 | Cận dưới hợp lệ, vẫn thu giá cơ bản |
| 1, 2 | 12.000 | Trong 2 km đầu, phần vượt không âm |
| 2,001 | 12.005 | 1 m vượt là 4,5 đồng, làm tròn nửa lên |
| 2,0004 và 2,0006 | 12.000 và 12.005 | Độ phân giải 1 m ở biên km sang mét |
| 2,3 | 13.350 | 300 m vượt, khớp `main.dart` |
| 5 | 25.500 | Ca thông thường, khớp Hình 1 |
| 100 | 453.000 | Cận trên hợp lệ |
| -1, 100,5 | `ArgumentError` | Ngoài miền |
| `double.nan` | `ArgumentError` | Guard viết đúng chiều |
| `double.infinity` | `ArgumentError` | Lớn hơn 100 |

```dart
test('NaN và Infinity ném ArgumentError (NaN so sánh luôn false)', () {
  expect(() => calculateFareVnd(double.nan), throwsArgumentError);
  expect(() => calculateFareVnd(double.infinity), throwsArgumentError);
  expect(
    () => calculateFareVnd(double.negativeInfinity),
    throwsArgumentError,
  );
});

test('2.001 km: 1 m vượt = 4,5 đồng, làm tròn nửa lên thành 12005', () {
  expect(calculateFareVnd(2.001), 12005);
});

test('độ phân giải 1 m: 2.0004 km tính là 2000 m, 2.0006 km là 2001 m', () {
  expect(calculateFareVnd(2.0004), 12000);
  expect(calculateFareVnd(2.0006), 12005);
});
```

Để chắc test NaN có giá trị, ta thử đổi guard thành `!(distanceKm < 0 || distanceKm > maxDistanceKm)`. NaN lọt qua, nổ ở `round()` bằng `UnsupportedError` thay vì `ArgumentError`, và test NaN đỏ. Cách sửa là viết guard dạng điều kiện hợp lệ, hoặc thêm `distanceKm.isNaN ||`.

## dynamic và Object: hai lối thoát nên đóng lại

Gặp kiểu khó, người mới hay thả `dynamic` cho hết lỗi. `dynamic` tắt kiểm tra kiểu: compiler cho gọi bất cứ thứ gì, lỗi chỉ lộ ra khi chạy. `Object` thì khác: compiler chỉ cho gọi những gì `Object` có.

```dart
test('dynamic dời lỗi sang lúc chạy', () {
  final dynamic anything = 'Tới Nơi';

  expect(anything.length, 7);
  expect(() => anything.fareVnd, throwsNoSuchMethodError);
});

test('Object giữ lại kiểm tra lúc biên dịch, is để thu hẹp kiểu', () {
  const Object anything = 'Tới Nơi';

  // anything.length; // không biên dịch: Object không có length
  expect(anything is String ? anything.length : -1, 7);
});
```

Một dòng sai tên như `anything.fareVnd` qua được compiler nhưng ném `NoSuchMethodError` khi khách bấm nút. Quy tắc của loạt bài: không dùng `dynamic` trong code nghiệp vụ. Dữ liệu từ ngoài (JSON ở bài 19) có thể tạm là `Object?`, rồi kiểm bằng `is` hoặc pattern matching (bài 03) để đưa về kiểu cụ thể ngay ở biên.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp</span>
<ul>
<li><b>Guard để lọt NaN.</b> Dấu hiệu: NaN không bị guard chặn mà nổ sâu trong phép tính với <code>UnsupportedError</code> từ <code>round()</code>, thông báo không nêu tham số; nơi gọi chỉ bắt <code>ArgumentError</code> sẽ bỏ sót. Cách sửa: nêu điều kiện hợp lệ rồi phủ định, hoặc kiểm <code>isNaN</code> riêng, và đặt guard trước <code>round()</code>.</li>
<li><b>Rải <code>!</code> để hết lỗi compiler.</b> Dấu hiệu: <code>Null check operator used on a null value</code> trên máy khách hàng. Cách sửa: kiểm <code>null</code> (promotion), dùng <code>??</code> hoặc <code>?.</code>, hoặc sửa hợp đồng để giá trị không còn nullable.</li>
<li><b>Dùng <code>""</code> thay cho null.</b> Dấu hiệu: UI hiện ô trống, hoặc <code>split(' ').last</code> trên chuỗi rỗng, không ai phân biệt được "chưa có" với "lỗi". Cách sửa: chọn một biểu diễn cho "chưa có" (null) và chặn chuỗi rỗng ở biên, như constructor của <code>Quote</code>.</li>
<li><b>Dùng <code>late</code> không khởi tạo.</b> Dấu hiệu: lỗi <code>has not been initialized</code> lúc chạy. Cách sửa: truyền giá trị qua constructor, hoặc dùng <code>late final</code> kèm initializer.</li>
<li><b>Nhầm <code>/</code> với <code>~/</code> và làm tròn nhiều lần.</b> Dấu hiệu: cước lệch 1 đồng so với kỳ vọng (như 12.004 thay vì 12.005), hoặc nhận <code>double</code> khi cần <code>int</code>. Cách sửa: chia nguyên bằng <code>~/</code>, chỉ làm tròn ở biên km sang mét, sau đó toàn <code>int</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/02-dart-bien-kieu-null-safety
dart pub get
dart run bin/main.dart
dart analyze
dart test
```

`dart run bin/main.dart` in:

```text
Đã có tài xế : 5.0 km | 25.500 VND | Châu
Chưa có tài xế: 2.3 km | 13.350 VND | chưa có tài xế
Sau khi nhận  : 2.3 km | 13.350 VND | An
5 / 2 = 2.5, 5 ~/ 2 = 2
453.000 VND
```

Ba file test chứng minh ba nhóm điều khác nhau:

| File | Chứng minh |
| --- | --- |
| `test/fare_test.dart` | Ca biên của cước (0, 1, 2, 2,001, 2,0004, 2,0006, 5, 100, ngoài miền, NaN, Infinity) và `formatVnd` |
| `test/quote_test.dart` | null và chuỗi rỗng khác nhau, `withDriver` không sửa bản cũ, `late` |
| `test/basics_test.dart` | Các khẳng định về `/`, `~/`, `%`, `round()` (kể cả 12.004 khi làm tròn phần vượt bằng double), NaN, `!`, `dynamic`, `Object` |

Kết quả đã kiểm với Flutter 3.47.4 / Dart 3.13.3: `dart analyze` không có issue, `dart test` chạy 40 test xanh. Mức kiểm chứng: **kiểm chứng tự động**, không cần simulator hay thiết bị.

Phần chưa làm: `Money` và `Trip` (bài 03, 06), định dạng tiền theo locale bằng `intl` (bài 14), phụ phí hay giảm giá, và kiểu `Distance` lưu khoảng cách bằng `int` mét (hàm vẫn nhận `double` km để dễ học).

## Bài tập

1. Dự đoán `calculateFareVnd(2.0004)` và `calculateFareVnd(2.0006)`, rồi giải thích vì sao hai kết quả khác nhau.
2. Nếu đổi `final String? driverName` thành `final String driverName` bắt buộc, code của ta thay đổi ra sao và ta mất điều gì?
3. Không chạy máy, cho biết giá trị của `(7 / 2).round()`, `7 ~/ 2`, `-7 ~/ 2` và `-7 % 2`.

<details><summary>Xem lời giải</summary>
<p><b>1.</b> 12.000 và 12.005. 2,0004 km làm tròn thành 2.000 m, không có mét vượt. 2,0006 km thành 2.001 m, vượt 1 m là 4,5 đồng, làm tròn nửa lên thành 5. Bước làm tròn duy nhất nằm ở biên km sang mét.</p>
<p><b>2.</b> Mọi nơi tạo báo giá chưa có tài xế phải bịa một giá trị như <code>""</code> hay <code>"N/A"</code>. Ta mất khả năng phân biệt "chưa có" với "có tên lạ", và compiler không còn nhắc xử lý trường hợp này: hợp đồng trở nên mơ hồ.</p>
<p><b>3.</b> <code>(7 / 2).round()</code> là 4 (3,5 làm tròn ra xa số 0), <code>7 ~/ 2</code> là 3, <code>-7 ~/ 2</code> là -3 (cắt về phía 0), <code>-7 % 2</code> là 1 (phần dư của <code>%</code> không âm khi số chia dương).</p>
</details>

**Bài trước:** [Flutter và Dart: từ source code đến ứng dụng Tới Nơi](/articles/flutter-va-dart-tu-source-code-den-ung-dung-toi-noi). **Bài tiếp:** [Hàm, class, enum và sealed class: mô hình trạng thái chuyến bằng Dart](/articles/dart-ham-class-enum-sealed-pattern-matching-trang-thai-chuyen).

## Tài liệu gốc

- [Built-in types: numbers, strings, records](https://dart.dev/language/built-in-types)
- [Variables: var, final, const, late](https://dart.dev/language/variables)
- [Operators: /, ~/, %, ?., ??](https://dart.dev/language/operators)
- [Sound null safety](https://dart.dev/null-safety)
- [Understanding null safety](https://dart.dev/null-safety/understanding-null-safety)
- [Type promotion on null checks](https://dart.dev/tools/non-promotion-reasons)
- [Numbers in Dart (int và double)](https://dart.dev/resources/language/number-representation)
- [dart:core, num.round](https://api.dart.dev/stable/dart-core/num/round.html)
