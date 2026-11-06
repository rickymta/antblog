---
slug: flutter-form-validation-focus-nhap-diem-don-diem-den
title: "Form, TextFormField và validation: nhập điểm đón, điểm đến"
summary: "Bài 11: dựng TripRequestForm của Tới Nơi với Form, TextFormField và validator thuần Dart có unit test; AutovalidateMode, FocusNode và TextInputAction.next chuyển ô, dispose controller, chuẩn hoá dữ liệu khi submit và cách Scaffold xử lý bàn phím."
category: programming
level: beginner
tags: [flutter, dart, mobile, toi-noi, form, validation]
language: vi
---

Khách của Tới Nơi thường đặt xe khi đang đứng bên lề đường, một tay xách đồ, tay kia gõ. Bàn phím chiếm nửa màn hình, ngón cái dễ gõ nhầm, và người Việt viết "5,5 km" chứ không viết "5.5". Nếu form không giúp, ta nhận về điểm đón rỗng, điểm đến trùng điểm đón hoặc khoảng cách "abc"; server từ chối và khách phải gõ lại.

Bài này dựng `TripRequestForm`: ba ô điểm đón, điểm đến và khoảng cách (nhập tay, tạm thay cho bản đồ ở Phần 5). Form báo lỗi đúng lúc và đúng ô, phím hành động đưa focus sang ô kế tiếp, và khi hợp lệ thì gửi đi dữ liệu đã chuẩn hoá. Quy tắc kiểm nằm trong hàm thuần Dart có unit test; widget test chứng minh phần giao diện.

Đây là bài 11 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 10 về StatefulWidget, lifecycle và setState](/articles/flutter-statefulwidget-lifecycle-setstate-chon-dich-vu). Ta cần biết `State`, `initState` và `dispose` từ bài đó, cùng `FarePolicy` và `calculateFare` của `toi_noi_core` từ [bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core). Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); bảng giá và số km chỉ là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ô khoảng cách phải nhận <code>5,5</code>. Ta định kiểm bằng <code>double.tryParse(text)</code>: trả <code>null</code> thì báo lỗi, ngược lại thì tin kết quả. Cách này có đủ không?</p>
<details><summary>Xem đáp án</summary><p>Không đủ, và sai theo hai hướng. <code>double.tryParse('5,5')</code> trả <code>null</code>, nên chính định dạng tiếng Việt bị coi là sai. Ngược lại, <code>'NaN'</code>, <code>'Infinity'</code> và <code>'1e3'</code> đều parse thành công (NaN, vô cực, 1000.0). Test của lab chứng minh cả hai điều. Vì vậy ta kiểm định dạng bằng một biểu thức chính quy chặt rồi mới đọc số và kiểm miền.</p></details>
</div>

## Form, FormField và validator: ai gọi ai

`TextField` chỉ là ô nhập. `TextFormField` bọc nó trong một `FormField<String>` để thêm `validator` và tự đưa câu lỗi vào `InputDecoration.errorText`. `Form` là widget tổ tiên: mỗi `FormField` tự đăng ký với `FormState` gần nhất khi nó được build. Khi ta gọi `FormState.validate()`, `FormState` gọi validator của từng field đã đăng ký và trả `true` nếu tất cả đều hợp lệ.

Validator nhận giá trị hiện tại và trả `String?`: `null` là hợp lệ, một chuỗi là câu lỗi hiện dưới ô (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 1. Nút Tìm tài xế gọi _submit, _submit gọi FormState.validate. FormState gọi validator của ba TextFormField đã đăng ký, mỗi validator trả null hoặc một câu lỗi. Cả ba trả null thì validate trả true và onSubmit được gọi; có lỗi thì validate trả false, các ô hiện lỗi và _submit đưa focus tới ô lỗi đầu tiên."><defs><marker id="fl11-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="115" width="190" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="146" class="t-b">Nút Tìm tài xế</text><text x="36" y="172" class="t-mono">_submit()</text><text x="36" y="194" class="t-small">gọi validate()</text>
<path d="M212 160 L268 160" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-1-a)"/>
<rect x="270" y="115" width="200" height="90" rx="10" class="f-pk-soft s-pk"/><text x="286" y="146" class="t-b">FormState</text><text x="286" y="172" class="t-mono">validate()</text><text x="286" y="194" class="t-small">duyệt các ô đăng ký</text>
<path d="M472 140 L538 62" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-1-a)"/>
<path d="M472 160 L538 180" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-1-a)"/>
<path d="M472 180 L538 298" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-1-a)"/>
<rect x="540" y="20" width="320" height="84" rx="10" class="f-phys-soft s-phys"/><text x="556" y="48" class="t-b">Điểm đón</text><text x="556" y="72" class="t-mono">validatePickup(value)</text><text x="556" y="92" class="t-small">trả null hoặc câu lỗi</text>
<rect x="540" y="138" width="320" height="84" rx="10" class="f-phys-soft s-phys"/><text x="556" y="166" class="t-b">Điểm đến</text><text x="556" y="190" class="t-mono">validateDestination(...)</text><text x="556" y="210" class="t-small">trả null hoặc câu lỗi</text>
<rect x="540" y="256" width="320" height="84" rx="10" class="f-phys-soft s-phys"/><text x="556" y="284" class="t-b">Khoảng cách (km)</text><text x="556" y="308" class="t-mono">validateDistance(value)</text><text x="556" y="328" class="t-small">trả null hoặc câu lỗi</text>
<path d="M370 207 L370 243" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-1-a)"/>
<rect x="20" y="245" width="450" height="100" rx="10" class="f-bg s-line"/><text x="36" y="274" class="t-small">Cả ba trả null: validate() = true,</text><text x="36" y="298" class="t-small">gọi onSubmit(data) với dữ liệu chuẩn hoá.</text><text x="36" y="322" class="t-small">Có câu lỗi: false; _submit đưa focus về ô lỗi.</text>
</svg>
</div>
<figcaption>Hình 1. Luồng kiểm khi bấm gửi. Mũi tên chỉ chiều gọi: nút gọi validate(), FormState gọi validator của từng ô, kết quả quyết định có gọi onSubmit hay không.</figcaption>
</figure>

Trong `_TripRequestFormState`, ta giữ một `GlobalKey<FormState>` để gọi `validate()`. Ta cũng giữ ba `GlobalKey<FormFieldState<String>>`, một cho mỗi ô, để hỏi `hasError` và biết ô nào đang lỗi:

### trip_request_form.dart · 11-form-validation/lib/trip_request_form.dart (trích)

```dart
void _submit() {
  if (!_formKey.currentState!.validate()) {
    _focusFirstInvalid();
    return;
  }
  // validate() vừa qua nên parseDistanceKm không thể trả null.
  final data = TripRequestData(
    pickup: normalizeAddress(_pickupController.text),
    destination: normalizeAddress(_destinationController.text),
    distanceKm: parseDistanceKm(_distanceController.text)!,
  );
  FocusScope.of(context).unfocus();
  widget.onSubmit(data);
}
```

### trip_request_form.dart · 11-form-validation/lib/trip_request_form.dart (trích)

```dart
void _focusFirstInvalid() {
  final fields = [
    (_pickupKey, _pickupFocus),
    (_destinationKey, _destinationFocus),
    (_distanceKey, _distanceFocus),
  ];
  for (final (key, focus) in fields) {
    if (key.currentState?.hasError ?? false) {
      focus.requestFocus();
      return;
    }
  }
}
```

Sau `validate()` ta đưa focus tới ô lỗi đầu tiên theo thứ tự trên màn hình, nên khách không phải tự tìm chỗ sai. Khi hợp lệ, `unfocus()` hạ bàn phím trước khi gọi callback.

Lưu ý: `Form` chỉ biết các field **đang được dựng**; field ngoài vùng của `ListView` dựng lười bị `validate()` bỏ qua (xem lỗi thường gặp).

## Validator là hàm thuần: tách ra và kiểm bằng unit test

Validator không cần `BuildContext` hay widget, nên ta viết thành hàm thuần Dart trong `validators.dart`, không import Flutter. Test chạy tức thì và ca biên liệt kê bằng một danh sách thay vì dựng cây widget.

Điểm đến cần nhìn sang ô khác, nên `validateDestination` nhận thêm tham số `pickup`:

### validators.dart · 11-form-validation/lib/validators.dart (trích)

```dart
String? validateDestination(String? value, {String? pickup}) {
  final error = validateAddress(value, label: 'điểm đến');
  if (error != null) {
    return error;
  }
  if (pickup != null && _sameAddress(value!, pickup)) {
    return 'Điểm đến trùng điểm đón.';
  }
  return null;
}

bool _sameAddress(String a, String b) =>
    normalizeAddress(a).toLowerCase() == normalizeAddress(b).toLowerCase();
```

Validator của ô điểm đến là một closure đọc `_pickupController.text` **tại lúc kiểm**, không chụp giá trị cũ (code ở phần controller bên dưới). So sánh bỏ qua hoa thường và khoảng trắng thừa: " 12  LÊ lợi " trùng "12 Lê Lợi".

Ô khoảng cách khó hơn. Biểu thức chính quy chỉ nhận chữ số ASCII, dấu trừ tuỳ chọn ở đầu và tối đa một dấu phân cách thập phân (phẩy hoặc chấm). Miền hợp lệ lấy từ `FarePolicy.maxKm` của `toi_noi_core`:

### validators.dart · 11-form-validation/lib/validators.dart (trích)

```dart
// Chỉ chữ số ASCII, tuỳ chọn dấu trừ ở đầu, tối đa một dấu phân cách thập phân
// (`,` hoặc `.`). Cố ý không dùng `double.tryParse`: nó nhận cả "NaN", "Infinity"
// và "1e3", còn dấu phẩy thì không nhận.
final RegExp _distancePattern = RegExp(r'^-?\d+(?:[.,]\d+)?$');

/// Đọc khoảng cách km từ chuỗi người dùng gõ; trả `null` nếu sai định dạng.
///
/// Nhận cả dấu phẩy (`5,5`, kiểu tiếng Việt) lẫn dấu chấm (`5.5`). Không nhận
/// chuỗi có cả hai dấu hoặc nhiều hơn một dấu phân cách như `1.234,5`.
/// Hàm này chỉ đọc số, việc kiểm miền 0..100 km nằm ở [validateDistance].
double? parseDistanceKm(String? raw) {
  final text = (raw ?? '').trim();
  if (!_distancePattern.hasMatch(text)) {
    return null;
  }
  final km = double.parse(text.replaceFirst(',', '.'));
  // "-0" ra -0.0; đổi về 0.0 để dữ liệu chuẩn hoá không mang dấu âm.
  return km == 0 ? 0.0 : km;
}

/// Kiểm ô khoảng cách. Miền hợp lệ lấy từ [FarePolicy.maxKm] để form và bảng
/// giá không bao giờ lệch nhau.
String? validateDistance(String? raw) {
  if ((raw ?? '').trim().isEmpty) {
    return 'Nhập khoảng cách (km).';
  }
  final km = parseDistanceKm(raw);
  if (km == null) {
    return 'Nhập số, ví dụ 5 hoặc 5,5.';
  }
  // Viết dưới dạng "điều kiện hợp lệ" rồi phủ định, giống calculateFareVnd.
  final isInRange = km >= 0 && km <= FarePolicy.maxKm;
  if (!isInRange) {
    return 'Khoảng cách phải từ 0 đến ${FarePolicy.maxKm} km.';
  }
  return null;
}
```

Vài lựa chọn đáng nói:

- `\d` trong Dart chỉ khớp chữ số ASCII, nên chữ số Ả Rập như "٣" bị từ chối.
- Chuỗi có nhiều hơn một dấu phân cách (`1,2,3`, `1.234,5`) bị từ chối vì không rõ dấu nào là phân cách. Ngược lại `1.500` được đọc là 1,5 km. Với miền 0 đến 100 km ta chấp nhận điều này và ghi vào phần chưa làm.
- `parseDistanceKm` chỉ đọc số, `validateDistance` mới kiểm miền. Một chuỗi 400 chữ số vẫn qua được biểu thức chính quy và `double.parse` ra vô cực; điều kiện miền viết dưới dạng "hợp lệ" rồi phủ định (giống `calculateFareVnd`) chặn nó, và test có ca này.

| Người dùng gõ | `parseDistanceKm` | `validateDistance` |
|---|---|---|
| `5,5` hoặc `5.5` | 5.5 | `null` (hợp lệ) |
| ` 2,001 ` | 2.001 | `null` |
| `100,5` | 100.5 | "Khoảng cách phải từ 0 đến 100 km." |
| `1,2,3` | `null` | "Nhập số, ví dụ 5 hoặc 5,5." |
| `NaN`, `1e3` | `null` | "Nhập số, ví dụ 5 hoặc 5,5." |
| rỗng | `null` | "Nhập khoảng cách (km)." |

Test thuần Dart liệt kê 18 chuỗi bị từ chối (từ `abc`, `1,2,3` tới `NaN`, `1e3`) và ghi lại vì sao ta không dùng `double.tryParse`:

### validators_test.dart · 11-form-validation/test/validators_test.dart (trích)

```dart
test('tryParse nhận NaN, Infinity, 1e3 nhưng không nhận dấu phẩy', () {
  expect(double.tryParse('NaN')!.isNaN, isTrue);
  expect(double.tryParse('Infinity'), double.infinity);
  expect(double.tryParse('1e3'), 1000);
  expect(double.tryParse('5,5'), isNull);
});
```

Một test nối form với domain: mọi chuỗi `validateDistance` cho qua đều đưa được vào `calculateFare` mà không ném `ArgumentError`. Nếu ai đó gõ cứng một miền riêng (ví dụ 120) cho form, ca biên `100,01` đỏ.

## AutovalidateMode: khi nào nên hiện lỗi

Hiện lỗi quá sớm thì khách thấy mình bị la ngay chữ đầu; quá muộn thì họ chỉ biết sai lúc bấm gửi. `AutovalidateMode` quyết định thời điểm; source Flutter 3.47.4 có năm chế độ:

| Chế độ | Khi nào tự kiểm |
|---|---|
| `disabled` (mặc định) | Không bao giờ; chỉ khi ta gọi `validate()` |
| `always` | Mỗi lần ô build, kể cả khi chưa ai gõ gì |
| `onUserInteraction` | Sau khi người dùng đã đổi nội dung ô, mỗi lần build |
| `onUnfocus` | Khi ô mất focus |
| `onUserInteractionIfError` | Sau khi người dùng đã đổi nội dung, nhưng chỉ kiểm lại ô **đang có lỗi** |

Lab chọn `onUserInteractionIfError` cho cả ba ô. Trước lần gửi đầu không ô nào đỏ. Sau khi gửi hỏng, ô có lỗi tự kiểm lại mỗi lần khách sửa và câu lỗi biến mất khi nội dung hợp lệ. Một test dựng ô trần để so sánh: gõ "a" vào ô điểm đón thì `onUserInteraction` báo "Nhập ít nhất 3 ký tự" ngay ở ký tự đầu, còn `onUserInteractionIfError` và `disabled` im lặng.

### trip_request_form_test.dart · 11-form-validation/test/trip_request_form_test.dart (trích)

```dart
testWidgets('sửa ô đang lỗi thì lỗi biến mất, không cần bấm gửi lại', (
  tester,
) async {
  await _pumpForm(tester, (_) {});
  await _tapSubmit(tester);
  expect(find.text('Nhập điểm đón.'), findsOneWidget);

  await tester.enterText(_field(_pickupLabel), '12 Lê Lợi');
  await tester.pump();

  expect(find.text('Nhập điểm đón.'), findsNothing);
  // Các ô chưa sửa vẫn giữ lỗi.
  expect(find.text('Nhập điểm đến.'), findsOneWidget);
});
```

Các ô còn tác động lẫn nhau theo hai chiều, và test ghi lại cả hai:

- Mỗi khi một ô đổi, `Form` dựng lại **mọi** ô. Nên khi điểm đến đang báo "trùng điểm đón" mà khách sửa điểm đón, lỗi ở điểm đến tự biến mất: ô đó có lỗi và đã được gõ vào, nên tự kiểm lại với điểm đón mới.
- Chiều ngược lại thì không: điểm đến đang hợp lệ, khách sửa điểm đón cho trùng, ô điểm đến không có lỗi nên không kiểm lại. Lỗi chỉ hiện ở lần gửi kế tiếp.

Cái giá của "chỉ kiểm lại khi đã có lỗi" rộng hơn thế: sửa hết lỗi rồi gõ sai lại (chỉ "a", hoặc xoá trắng) thì ô cũng im lặng cho tới lần gửi sau. Test `sửa hết lỗi rồi gõ sai lại` ghi lại điều này.

Đặt `autovalidateMode` ở `Form` khác ở từng field: với `Form(autovalidateMode: onUserInteraction)`, gõ một ô là `Form` kiểm mọi ô, kể cả ô chưa chạm (test `đặt ở cấp Form`). Vì vậy lab đặt chế độ ở từng `TextFormField`.

## Controller, FocusNode và TextInputAction

`TextEditingController` giữ nội dung ô; ta cần nó để đọc giá trị lúc gửi và để validator điểm đến đọc điểm đón. `FocusNode` cho ta điều khiển focus bằng code. Cả hai là `ChangeNotifier`: ai tạo thì người đó dispose. `TextFormField` chỉ **mượn** chúng (Hình 2), nên `State` phải trả lại ở `dispose()`, như bài 10.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. _TripRequestFormState tạo và sở hữu ba TextEditingController, ba FocusNode và ba GlobalKey, rồi truyền controller và focusNode vào ba TextFormField. TextFormField chỉ dùng, không dispose. Trong dispose của State phải giải phóng ba controller và ba FocusNode; quên thì các đối tượng không được trả đúng lúc."><defs><marker id="fl11-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl11-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="40" width="300" height="220" rx="10" class="f-phys-soft s-phys"/><text x="36" y="72" class="t-b">_TripRequestFormState</text><text x="36" y="104" class="t-mono">3 TextEditingController</text><text x="36" y="128" class="t-mono">3 FocusNode</text><text x="36" y="152" class="t-mono">3 GlobalKey</text><text x="36" y="196" class="t-small">dispose() giải phóng</text><text x="36" y="218" class="t-small">3 controller và 3 FocusNode</text>
<path d="M322 100 L518 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-2-a)"/><text x="345" y="88" class="t-small">controller, focusNode</text>
<rect x="520" y="40" width="340" height="110" rx="10" class="f-pk-soft s-pk"/><text x="536" y="72" class="t-b">TextFormField x3</text><text x="556" y="100" class="t-small">chỉ mượn, không sở hữu</text><text x="556" y="124" class="t-small">không tự dispose hộ ta</text>
<path d="M322 225 L518 225" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl11-2-w)"/><text x="345" y="213" class="t-small t-warn">nếu quên dispose</text>
<rect x="520" y="185" width="340" height="80" rx="10" class="f-warn-soft s-warn"/><text x="536" y="216" class="t-b">Đối tượng bị bỏ quên</text><text x="536" y="242" class="t-small">không được giải phóng đúng lúc</text>
</svg>
</div>
<figcaption>Hình 2. Quan hệ sở hữu. State tạo và sở hữu controller, FocusNode; TextFormField chỉ nhận chúng qua tham số. Quên dispose thì không ai trả lại.</figcaption>
</figure>

### trip_request_form.dart · 11-form-validation/lib/trip_request_form.dart (trích)

```dart
final _formKey = GlobalKey<FormState>();

final _pickupKey = GlobalKey<FormFieldState<String>>();
final _destinationKey = GlobalKey<FormFieldState<String>>();
final _distanceKey = GlobalKey<FormFieldState<String>>();

final _pickupController = TextEditingController();
final _destinationController = TextEditingController();
final _distanceController = TextEditingController();

final _pickupFocus = FocusNode();
final _destinationFocus = FocusNode();
final _distanceFocus = FocusNode();

@override
void dispose() {
  _pickupController.dispose();
  _destinationController.dispose();
  _distanceController.dispose();
  _pickupFocus.dispose();
  _destinationFocus.dispose();
  _distanceFocus.dispose();
  super.dispose();
}
```

Quên dispose không gây lỗi nào trên màn hình, nên lab có test riêng: lấy controller và FocusNode từ các `TextField` đang dựng, gỡ form khỏi cây, rồi thử `addListener`. `ChangeNotifier` đã dispose ném `FlutterError` khi bật assert (như trong `flutter test`):

### trip_request_form_test.dart · 11-form-validation/test/trip_request_form_test.dart (trích)

```dart
testWidgets('gỡ form khỏi cây thì controller và FocusNode bị dispose', (
  tester,
) async {
  await _pumpForm(tester, (_) {});
  final controllers = [
    for (final label in [_pickupLabel, _destinationLabel, _distanceLabel])
      _controllerOf(tester, label),
  ];
  final focusNodes = [
    for (final label in [_pickupLabel, _destinationLabel, _distanceLabel])
      _focusOf(tester, label),
  ];
  // Còn sống: thêm listener được.
  for (final c in [...controllers, ...focusNodes]) {
    c.addListener(() {});
  }

  await tester.pumpWidget(const SizedBox.shrink());

  // ChangeNotifier đã dispose thì addListener ném FlutterError (khi bật assert).
  for (final c in controllers) {
    expect(() => c.addListener(() {}), throwsFlutterError);
  }
  for (final f in focusNodes) {
    expect(() => f.addListener(() {}), throwsFlutterError);
  }
});
```

Test này đỏ khi ta xoá bất kỳ dòng `dispose()` nào (đã thử cả sáu dòng).

Hai ô còn lại cho thấy các tham số của `TextFormField`. Ô điểm đến đọc điểm đón qua closure; ô khoảng cách chọn bàn phím số có dấu thập phân và lọc ký tự:

### trip_request_form.dart · 11-form-validation/lib/trip_request_form.dart (trích)

```dart
TextFormField(
  key: _destinationKey,
  controller: _destinationController,
  focusNode: _destinationFocus,
  decoration: const InputDecoration(
    labelText: 'Điểm đến',
    prefixIcon: Icon(Icons.place),
  ),
  keyboardType: TextInputType.streetAddress,
  textInputAction: TextInputAction.next,
  textCapitalization: TextCapitalization.words,
  autovalidateMode: AutovalidateMode.onUserInteractionIfError,
  // Đọc điểm đón tại thời điểm kiểm, không chụp giá trị cũ.
  validator: (value) =>
      validateDestination(value, pickup: _pickupController.text),
),
```

### trip_request_form.dart · 11-form-validation/lib/trip_request_form.dart (trích)

```dart
TextFormField(
  key: _distanceKey,
  controller: _distanceController,
  focusNode: _distanceFocus,
  decoration: const InputDecoration(
    labelText: 'Khoảng cách (km)',
    helperText: 'Tạm nhập tay; bài bản đồ sẽ tính tự động',
    prefixIcon: Icon(Icons.straighten),
  ),
  keyboardType: const TextInputType.numberWithOptions(decimal: true),
  // Bộ lọc chỉ giúp gõ đỡ nhầm, không thay validator: dán vào vẫn
  // có thể ra "1,2,3".
  inputFormatters: [
    FilteringTextInputFormatter.allow(RegExp(r'[0-9.,]')),
  ],
  textInputAction: TextInputAction.done,
  autovalidateMode: AutovalidateMode.onUserInteractionIfError,
  validator: validateDistance,
  onFieldSubmitted: (_) => _submit(),
),
```

Tiếp theo là `textInputAction`, phím hành động trên bàn phím. Với `TextInputAction.next`, nếu không đặt `onEditingComplete`, ô nhập gọi `focusNode.nextFocus()` (source `EditableText`). Thứ tự focus theo `ReadingOrderTraversalPolicy`, chính sách `WidgetsApp` đặt mặc định: từ trên xuống dưới theo vị trí (Hình 3), nên ta chỉ cần xếp ba ô đúng thứ tự trong `Column`. Ô cuối dùng `TextInputAction.done`: bỏ focus rồi gọi `onFieldSubmitted`, ta nối nó vào `_submit`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 3. Chuỗi focus. Ô điểm đón dùng TextInputAction.next chuyển focus sang ô điểm đến, ô điểm đến dùng next chuyển sang ô khoảng cách, ô khoảng cách dùng done và gọi onFieldSubmitted, từ đó gọi _submit. Thứ tự next theo vị trí từ trên xuống dưới."><defs><marker id="fl11-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="240" height="96" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">Điểm đón</text><text x="36" y="88" class="t-mono">TextInputAction.next</text><text x="36" y="110" class="t-small">_pickupFocus</text>
<path d="M262 78 L318 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-3-a)"/><text x="276" y="68" class="t-small">next</text>
<rect x="320" y="30" width="240" height="96" rx="10" class="f-phys-soft s-phys"/><text x="336" y="62" class="t-b">Điểm đến</text><text x="336" y="88" class="t-mono">TextInputAction.next</text><text x="336" y="110" class="t-small">_destinationFocus</text>
<path d="M562 78 L618 78" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-3-a)"/><text x="576" y="68" class="t-small">next</text>
<rect x="620" y="30" width="240" height="96" rx="10" class="f-phys-soft s-phys"/><text x="636" y="62" class="t-b">Khoảng cách (km)</text><text x="636" y="88" class="t-mono">TextInputAction.done</text><text x="636" y="110" class="t-small">_distanceFocus</text>
<path d="M740 128 L740 168" class="f-none s-pk" stroke-width="2" marker-end="url(#fl11-3-a)"/>
<rect x="620" y="170" width="240" height="60" rx="10" class="f-pk-soft s-pk"/><text x="636" y="196" class="t-mono">_submit()</text><text x="636" y="218" class="t-small">done: onFieldSubmitted</text>
<text x="20" y="176" class="t-small">Thứ tự next: từ trên xuống dưới theo vị trí</text><text x="20" y="200" class="t-small">(ReadingOrderTraversalPolicy, chính sách mặc định)</text>
</svg>
</div>
<figcaption>Hình 3. Phím hành động đi theo chuỗi trên bàn phím có phím hành động: hai ô đầu chuyển focus bằng next, ô cuối gửi form bằng done.</figcaption>
</figure>

Hai lưu ý. Với `next`, Flutter vẫn gọi `onSubmitted` (tức `onFieldSubmitted`) sau khi chuyển focus, nên ta **không** đặt `onFieldSubmitted` ở hai ô đầu, kẻo form gửi sớm. Và ta không dựa vào phím Done của ô khoảng cách. Theo engine, `numberWithOptions(decimal: true)` ứng với bàn phím decimal pad của iOS; theo kiến thức về iOS (chưa thử trên thiết bị) bàn phím này không có phím Return, nên trên iPhone khách phải bấm nút "Tìm tài xế". Vì vậy nút luôn có trên màn hình, còn chuỗi next/done của Hình 3 áp dụng cho bàn phím có phím hành động, chủ yếu Android. Test giả lập phím bằng `tester.testTextInput.receiveAction`:

### trip_request_form_test.dart · 11-form-validation/test/trip_request_form_test.dart (trích)

```dart
testWidgets('next chuyển focus điểm đón -> điểm đến -> khoảng cách', (
  tester,
) async {
  await _pumpForm(tester, (_) {});

  await tester.tap(_field(_pickupLabel));
  await tester.pump();
  expect(_focusOf(tester, _pickupLabel).hasPrimaryFocus, isTrue);

  await tester.testTextInput.receiveAction(TextInputAction.next);
  await tester.pump();
  expect(_focusOf(tester, _destinationLabel).hasPrimaryFocus, isTrue);

  await tester.testTextInput.receiveAction(TextInputAction.next);
  await tester.pump();
  expect(_focusOf(tester, _distanceLabel).hasPrimaryFocus, isTrue);
});
```

Bộ lọc `FilteringTextInputFormatter.allow(RegExp(r'[0-9.,]'))` giúp khách gõ đỡ nhầm nhưng không thay được validator: chuỗi `1,2,3` qua được bộ lọc và test chứng minh validator vẫn chặn.

## Chuẩn hoá dữ liệu khi gửi

Chuỗi khách gõ chưa phải dữ liệu để gửi. Ta gom vào value object `TripRequestData`: địa chỉ đã gọn khoảng trắng, khoảng cách `double` km đã đổi dấu phẩy thành dấu chấm:

### trip_request_data.dart · 11-form-validation/lib/trip_request_data.dart (trích)

```dart
final class TripRequestData {
  const TripRequestData({
    required this.pickup,
    required this.destination,
    required this.distanceKm,
  });

  final String pickup;
  final String destination;
  final double distanceKm;

  /// Cước ước tính theo bảng giá minh hoạ của `toi_noi_core`.
  Money get estimatedFare => calculateFare(distanceKm);

  // ... ==, hashCode, toString
}
```

`estimatedFare` không tự tính cước mà gọi `calculateFare` của `toi_noi_core`, nên luôn khớp bảng giá. Test gửi chuỗi lộn xộn và so sánh dữ liệu nhận được:

### trip_request_form_test.dart · 11-form-validation/test/trip_request_form_test.dart (trích)

```dart
testWidgets('gọn khoảng trắng và đổi dấu phẩy thành số thực', (
  tester,
) async {
  final received = <TripRequestData>[];
  await _pumpForm(tester, received.add);
  await _fill(
    tester,
    pickup: '  12   Lê  Lợi ',
    destination: ' Sân bay   Tân Sơn Nhất  ',
    distance: ' 2,001 ',
  );

  await _tapSubmit(tester);

  expect(received, [
    const TripRequestData(
      pickup: '12 Lê Lợi',
      destination: 'Sân bay Tân Sơn Nhất',
      distanceKm: 2.001,
    ),
  ]);
  expect(received.single.estimatedFare, calculateFare(2.001));
  expect(received.single.estimatedFare.vnd, 12005);
});
```

Bài 12 truyền một đối tượng báo giá, dựng từ dữ liệu tương tự, sang màn hình xác nhận chuyến. Đừng nhầm validation ở client với bảo vệ dữ liệu: server vẫn phải kiểm lại, như bài [validation và ProblemDetails của ASP.NET Core 10](/articles/dotnet10-validation-problem-details-openapi). Form chỉ giúp khách sửa nhanh hơn.

## Bàn phím, Scaffold và resizeToAvoidBottomInset

Khi bàn phím hiện, `viewInsets.bottom` lớn hơn 0. `Scaffold` mặc định có `resizeToAvoidBottomInset: true`: nó thu nhỏ `body` đúng bằng phần bàn phím chiếm, và body nhận `MediaQuery` đã bỏ phần inset đó. Form nằm trong `SingleChildScrollView` nên khi chỗ trống ít hơn nội dung, khách cuộn được tới nút gửi (Hình 4).

### request_screen.dart · 11-form-validation/lib/request_screen.dart (trích)

```dart
return Scaffold(
  appBar: AppBar(title: const Text('Đặt xe')),
  resizeToAvoidBottomInset: resizeToAvoidBottomInset,
  body: SafeArea(
    child: SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      keyboardDismissBehavior: ScrollViewKeyboardDismissBehavior.onDrag,
      child: TripRequestForm(onSubmit: (data) => _onSubmit(context, data)),
    ),
  ),
);
```

Ô nhận focus trong vùng cuộn được `EditableText` cuộn vào vùng nhìn thấy, chừa `scrollPadding` (mặc định 20 điểm ảnh); `keyboardDismissBehavior: onDrag` cho kéo để hạ bàn phím.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 4. Hai màn hình có bàn phím. Bên trái, resizeToAvoidBottomInset là true: body co lại bằng chiều cao bàn phím và form cuộn được tới nút gửi. Bên phải, resizeToAvoidBottomInset là false: body giữ nguyên chiều cao, nên với form ngắn hơn màn hình thì không cuộn và bàn phím che phần dưới của form gồm nút gửi.">
<text x="60" y="26" class="t-b">resizeToAvoidBottomInset: true</text>
<rect x="60" y="40" width="300" height="320" rx="20" class="f-none s-line"/>
<rect x="60" y="40" width="300" height="30" rx="0" class="f-surface s-line"/><text x="76" y="61" class="t-small">Đặt xe</text>
<rect x="60" y="70" width="300" height="150" class="f-pk-soft s-pk"/><text x="76" y="104" class="t-small">body co lại đúng bằng bàn phím</text><text x="76" y="128" class="t-small">form cuộn được tới nút gửi</text>
<rect x="60" y="220" width="300" height="140" class="f-phys-soft s-phys"/><text x="76" y="296" class="t-b">Bàn phím</text>
<text x="500" y="26" class="t-b">resizeToAvoidBottomInset: false</text>
<rect x="500" y="40" width="300" height="320" rx="20" class="f-none s-line"/>
<rect x="500" y="40" width="300" height="30" class="f-surface s-line"/><text x="516" y="61" class="t-small">Đặt xe</text>
<rect x="500" y="70" width="300" height="290" class="f-pk-soft s-pk"/><text x="516" y="104" class="t-small">body giữ nguyên chiều cao</text><text x="516" y="128" class="t-small">không cuộn nếu form ngắn hơn</text><text x="516" y="150" class="t-small">màn hình, nút gửi bị che</text>
<rect x="500" y="220" width="300" height="140" class="f-warn-soft s-warn"/><text x="516" y="296" class="t-b t-warn">Bàn phím che form</text>
</svg>
</div>
<figcaption>Hình 4. Hai cách Scaffold đối xử với bàn phím. Mặc định body co lại để khách cuộn tới nút gửi; đặt false thì body giữ nguyên và bàn phím che phần dưới.</figcaption>
</figure>

Test mô phỏng bàn phím bằng `tester.view.viewInsets` (300 điểm ảnh trên màn hình 600) và đo cả hai chế độ. Đây chỉ là mô phỏng chiều cao, không phải bàn phím thật:

### trip_request_form_test.dart · 11-form-validation/test/trip_request_form_test.dart (trích)

```dart
testWidgets(
  'mặc định: Scaffold lấy phần bàn phím ra khỏi body, form cuộn',
  (tester) async {
    await setUpScreen(tester, resize: true);

    await showKeyboard(tester);

    expect(bodyInset(tester), 0);
    expect(maxScroll(tester), greaterThan(0));

    // Cuộn xuống thì nút gửi lên trên mép bàn phím.
    await tester.ensureVisible(find.text('Tìm tài xế'));
    await tester.pump();
    expect(
      tester.getRect(find.text('Tìm tài xế')).bottom,
      lessThanOrEqualTo(screenHeight - keyboard),
    );
  },
);
```

Với form, giữ mặc định `true`.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với form</span>
<ul>
<li><strong>Quên dispose controller hoặc FocusNode.</strong> Dấu hiệu: không lỗi nào trên màn hình, chỉ test dispose của lab đỏ. Nguyên nhân: <code>State</code> tạo ra chúng nhưng không trả lại. Cách sửa: dispose đủ từng đối tượng trong <code>State.dispose()</code>, trước <code>super.dispose()</code>.</li>
<li><strong>Vừa đặt <code>initialValue</code> vừa truyền <code>controller</code>.</strong> Dấu hiệu: <code>AssertionError</code> ngay khi tạo <code>TextFormField</code> ở chế độ debug. Nguyên nhân: hai nguồn giá trị mâu thuẫn nên constructor có assert chặn. Cách sửa: chọn một, hoặc đặt giá trị đầu bằng <code>TextEditingController(text: ...)</code>.</li>
<li><strong>Đặt <code>Form</code> trong <code>ListView</code> dài.</strong> Dấu hiệu: <code>validate()</code> trả <code>true</code> dù một ô ở cuối đang sai. Nguyên nhân: ô chưa được dựng nên chưa đăng ký với <code>FormState</code>. Cách sửa: form ngắn thì dùng <code>SingleChildScrollView</code> + <code>Column</code> như lab; form dài thì kiểm dữ liệu trong model, không chỉ dựa vào <code>validate()</code>.</li>
<li><strong>Kiểm số bằng <code>double.tryParse</code> hoặc <code>double.parse</code>.</strong> Dấu hiệu: "5,5" bị báo sai, "NaN" hay "1e3" lọt qua, hoặc <code>FormatException</code> khi bấm gửi. Cách sửa: kiểm định dạng bằng biểu thức chính quy, đọc số, rồi mới kiểm miền như <code>parseDistanceKm</code>.</li>
<li><strong>Form không cuộn khi bàn phím hiện.</strong> Dấu hiệu: dải vàng đen "RenderFlex overflowed" dưới form. Nguyên nhân: <code>Scaffold</code> thu nhỏ body nhưng <code>Column</code> vẫn đòi chiều cao cũ. Cách sửa: bọc form trong <code>SingleChildScrollView</code>; test <code>form không cuộn gặp bàn phím</code> tái hiện đúng lỗi này.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/11-form-validation
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` báo 0 issue, 82 test xanh (48 test thuần Dart trong `validators_test.dart` và 34 widget test trong `trip_request_form_test.dart`). Mức kiểm chứng: **kiểm chứng tự động**, không cần simulator hay thiết bị.

| Test | Chứng minh điều gì |
|---|---|
| `validators_test.dart` | Chuẩn hoá khoảng trắng; địa chỉ rỗng, dưới 3 và trên 120 ký tự; điểm đến trùng điểm đón không phân biệt hoa thường; đọc `5,5`, `5.5`, `-0`; từ chối `1,2,3`, `NaN`, `1e3`, chữ số Ả Rập; biên 0 và 100 km; cước khớp `calculateFare` |
| `trip_request_form_test.dart`: lỗi | Form rỗng ra ba lỗi và không gọi callback; focus nhảy tới ô lỗi đầu; lỗi tự mất khi sửa; hai chiều của lỗi chéo ô; bộ lọc bàn phím và dán `1,2,3` |
| `trip_request_form_test.dart`: focus | Hai ô đầu `next`, ô cuối `done`; `next` chuyển focus mà không gửi form; `done` gửi form hoặc đưa focus về ô lỗi đầu |
| `trip_request_form_test.dart`: submit | Dữ liệu chuẩn hoá (2,001 km ra 12.005 VND, 100 km ra 453.000 VND, 5 km ra 25.500 VND); gửi xong thì hạ bàn phím |
| `trip_request_form_test.dart`: dispose, bàn phím, cạm bẫy | Sáu đối tượng bị dispose; `resizeToAvoidBottomInset` true và false; `initialValue` cùng `controller`; tràn RenderFlex; `Form` trong `ListView` bỏ sót ô |

**Chưa làm**: chạy trên iOS Simulator, Android emulator hay thiết bị thật (máy kiểm chưa có Android SDK và Xcode đầy đủ, nên bàn phím chỉ được mô phỏng); phím Return trên bàn phím số của từng thiết bị; hiểu `1.500` là một nghìn rưỡi (form đọc là 1,5 km); bản đồ và gợi ý địa chỉ; autofill; định dạng số theo locale bằng `intl` (bài 14); chuẩn hoá Unicode khi so sánh địa chỉ.

## Bài tập

1. Theo `toi_noi_core`, 0 km là hợp lệ (cước 12.000 VND). Nhưng một yêu cầu chuyến dài 0 km vô nghĩa. Ta nên sửa ở đâu để form từ chối 0, và có nên sửa `calculateFare` không?
2. Đổi `autovalidateMode` của ô điểm đón sang `AutovalidateMode.onUserInteraction`. Khách gõ ký tự "a" vào ô này thì thấy gì? Khác gì so với `onUserInteractionIfError`?
3. Trên một bàn phím Android có phím Done, khách điền khoảng cách "5", để trống hai ô địa chỉ rồi bấm Done. Callback có được gọi không, và focus đi đâu?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Sửa <code>validateDistance</code>: "dài hơn 0" là quy tắc của form đặt xe, không phải của bảng giá. <code>calculateFare</code> vẫn nên định nghĩa cho 0 km vì domain còn dùng ở chỗ khác. Thêm test biên cho "0" (lỗi) và "0,001" (hợp lệ).</p>
<p><strong>2.</strong> Lỗi "Nhập ít nhất 3 ký tự cho điểm đón." hiện ngay ở ký tự đầu, vì ô đã được gõ vào và chế độ này kiểm sau mỗi thay đổi. Với <code>onUserInteractionIfError</code>, ô chưa có lỗi nên im lặng cho tới lần gửi. Test <code>so sánh AutovalidateMode</code> ghi lại cả hai.</p>
<p><strong>3.</strong> Done gọi <code>_submit()</code>; <code>validate()</code> trả <code>false</code>, callback không được gọi, hai ô địa chỉ hiện lỗi, và <code>_focusFirstInvalid()</code> đưa focus về ô điểm đón (ô lỗi đầu tiên theo thứ tự trên màn hình), không ở lại ô khoảng cách.</p>
</details>

Bài trước: [StatefulWidget, lifecycle và setState](/articles/flutter-statefulwidget-lifecycle-setstate-chon-dich-vu). Bài tiếp: [Navigator, route và truyền dữ liệu](/articles/flutter-navigator-route-truyen-du-lieu-xac-nhan-chuyen).

## Tài liệu gốc

- [Build a form with validation](https://docs.flutter.dev/cookbook/forms/validation) và [Focus and text fields](https://docs.flutter.dev/cookbook/forms/focus) (docs.flutter.dev).
- [Retrieve the value of a text field](https://docs.flutter.dev/cookbook/forms/retrieve-input) và [Handle changes to a text field](https://docs.flutter.dev/cookbook/forms/text-field-changes) (docs.flutter.dev).
- [Form](https://api.flutter.dev/flutter/widgets/Form-class.html), [FormState.validate](https://api.flutter.dev/flutter/widgets/FormState/validate.html) và [FormFieldState](https://api.flutter.dev/flutter/widgets/FormFieldState-class.html) (api.flutter.dev).
- [TextFormField](https://api.flutter.dev/flutter/material/TextFormField-class.html) và [AutovalidateMode](https://api.flutter.dev/flutter/widgets/AutovalidateMode.html) (api.flutter.dev).
- [TextEditingController](https://api.flutter.dev/flutter/widgets/TextEditingController-class.html), [FocusNode](https://api.flutter.dev/flutter/widgets/FocusNode-class.html) và [FocusNode.nextFocus](https://api.flutter.dev/flutter/widgets/FocusNode/nextFocus.html) (api.flutter.dev).
- [ReadingOrderTraversalPolicy](https://api.flutter.dev/flutter/widgets/ReadingOrderTraversalPolicy-class.html) và [FocusTraversalGroup](https://api.flutter.dev/flutter/widgets/FocusTraversalGroup-class.html) (api.flutter.dev).
- [TextInputType](https://api.flutter.dev/flutter/services/TextInputType-class.html) và [FilteringTextInputFormatter](https://api.flutter.dev/flutter/services/FilteringTextInputFormatter-class.html) (api.flutter.dev).
- [Scaffold.resizeToAvoidBottomInset](https://api.flutter.dev/flutter/material/Scaffold/resizeToAvoidBottomInset.html) và [ScrollViewKeyboardDismissBehavior](https://api.flutter.dev/flutter/widgets/ScrollViewKeyboardDismissBehavior.html) (api.flutter.dev).
- [Testing Flutter apps](https://docs.flutter.dev/testing/overview) (docs.flutter.dev).
