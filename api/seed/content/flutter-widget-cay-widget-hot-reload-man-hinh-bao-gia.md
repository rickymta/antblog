---
slug: flutter-widget-cay-widget-hot-reload-man-hinh-bao-gia
title: "Widget, cây widget và hot reload: màn hình báo giá đầu tiên"
summary: "Bài 07: dựng màn hình báo giá đầu tiên của Tới Nơi bằng StatelessWidget; phân biệt Widget, Element và RenderObject bằng source framework, hiểu vì sao build chạy nhiều mà app vẫn nhanh, hot reload khác hot restart ở đâu và viết widget test kiểm cước 5 km."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, widget, testing]
language: vi
---

Thứ Hai, PM của Tới Nơi ngồi cạnh một lập trình viên đang chạy bản debug trên máy của anh ấy và nhắn: "Chữ cước to hơn một chút, thêm dòng nhắc đây là giá minh hoạ." Lập trình viên sửa hai dòng, hot reload, và màn hình đang mở đổi ngay, không phải khởi động lại app. Cảm giác đó chính là điểm bán hàng của Flutter, và nó dựa trên một mô hình khá cụ thể: giao diện là một cây widget được dựng lại liên tục, còn những gì thật sự nằm trên màn hình thì sống lâu hơn nhiều.

Đây là bài 07 của lộ trình **Flutter qua Tới Nơi**, bài đầu tiên của Phần 2 về giao diện. Bạn nên đọc trước [bài 06 về package toi_noi_core](/articles/dart-package-pubspec-lint-test-toi-noi-core), vì màn hình báo giá gọi `calculateFare` của package đó thay vì tự tính cước. Ta dựng `ToiNoiApp` và `QuoteScreen`, nhìn vào ba cây Widget, Element, RenderObject, xem hot reload tận dụng chúng ra sao, rồi viết widget test đầu tiên. Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi), bảng giá chỉ là số minh hoạ. Nếu cần nhắc lại cách chạy app và phím `r`, `R`, xem [bài 01](/articles/flutter-va-dart-tu-source-code-den-ung-dung-toi-noi).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Mỗi lần <code>build</code> của <code>QuoteScreen</code> chạy, nó trả về cả một cây widget mới. Vậy Flutter vứt hết những gì đang vẽ trên màn hình và tạo lại từ đầu, đúng hay sai?</p>
<details><summary>Xem đáp án</summary><p><b>Sai.</b> Widget được tạo lại, nhưng widget chỉ là bản mô tả nhẹ. Nếu widget mới cùng <code>runtimeType</code> và cùng <code>key</code> ở cùng vị trí, Flutter giữ nguyên Element và RenderObject cũ rồi chỉ cập nhật cấu hình của chúng. Bài có test dùng <code>identical</code> để chứng minh điều này.</p></details>
</div>

## Ba cây: Widget, Element và RenderObject

Mỗi thành phần trên màn hình Flutter do ba loại đối tượng phối hợp tạo nên, mỗi loại một việc (Hình 1).

**Widget** là bản mô tả cấu hình: "ô này có padding 16, bên trong là cái kia". Widget bất biến (lớp `Widget` được đánh dấu `@immutable`) và rẻ để tạo. `QuoteScreen(distanceKm: 5)` không vẽ gì cả, nó chỉ ghi lại rằng ta muốn báo giá 5 km.

**Element** là widget đã được đặt vào cây tại một vị trí cụ thể. Source của framework nói rõ: mỗi lần một widget được đưa vào cây, nó được inflate thành một Element, nên cùng một widget đặt hai chỗ thì có hai Element. Element giữ widget hiện hành của mình và sống qua nhiều lần build.

**RenderObject** đo kích thước, đặt vị trí và vẽ. Chỉ `RenderObjectWidget` mới tạo ra nó, qua `createRenderObject`. `StatelessWidget` và `StatefulWidget` thì không có RenderObject riêng: việc duy nhất của chúng là trả về widget khác từ `build`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Widget tạo Element bằng createElement; Element của RenderObjectWidget tạo RenderObject bằng createRenderObject. Ví dụ Padding(16) tạo SingleChildRenderObjectElement và RenderPadding."><defs><marker id="fl07-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="200" height="140" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">Widget</text><text x="36" y="88" class="t-small">bản mô tả cấu hình</text><text x="36" y="110" class="t-small">bất biến, rẻ để tạo</text><text x="36" y="132" class="t-small">tạo lại khi build</text>
<rect x="340" y="30" width="200" height="140" rx="10" class="f-pk-soft s-pk"/><text x="356" y="62" class="t-b">Element</text><text x="356" y="88" class="t-small">chỗ đứng trong cây</text><text x="356" y="110" class="t-small">sống qua các build</text><text x="356" y="132" class="t-small">giữ widget hiện hành</text>
<rect x="660" y="30" width="200" height="140" rx="10" class="f-surface s-muted"/><text x="676" y="62" class="t-b">RenderObject</text><text x="676" y="88" class="t-small">đo, đặt chỗ và vẽ</text><text x="676" y="110" class="t-small">chỉ có ở</text><text x="676" y="132" class="t-small">RenderObjectWidget</text>
<path d="M222 110 L338 110" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-1-a)"/><text x="280" y="96" text-anchor="middle" class="t-small">createElement</text>
<path d="M542 110 L658 110" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-1-a)"/><text x="600" y="78" text-anchor="middle" class="t-small">create</text><text x="600" y="94" text-anchor="middle" class="t-small">RenderObject</text>
<text x="20" y="212" class="t-small">Ví dụ với một widget có RenderObject</text>
<rect x="20" y="224" width="200" height="76" rx="10" class="f-phys-soft s-phys"/><text x="36" y="252" class="t-b">Padding(16)</text><text x="36" y="276" class="t-small">widget cấu hình</text>
<rect x="340" y="224" width="200" height="76" rx="10" class="f-pk-soft s-pk"/><text x="356" y="246" class="t-mono">SingleChild</text><text x="356" y="264" class="t-mono">RenderObjectElement</text><text x="356" y="286" class="t-small">element</text>
<rect x="660" y="224" width="200" height="76" rx="10" class="f-surface s-muted"/><text x="676" y="252" class="t-mono">RenderPadding</text><text x="676" y="276" class="t-small">padding 16, có size</text>
<path d="M222 262 L338 262" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-1-a)"/>
<path d="M542 262 L658 262" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-1-a)"/>
<text x="20" y="326" class="t-small">Text, Icon, Container là StatelessWidget: có Element, nhưng không có RenderObject riêng.</text>
</svg>
</div>
<figcaption>Hình 1. Ba loại đối tượng. Mũi tên chỉ đối tượng nào tạo ra đối tượng nào; hàng dưới là Padding làm ví dụ.</figcaption>
</figure>

Test dưới đây dựng app rồi hỏi từng cây về `Padding` của màn hình báo giá. Trước khi đọc, nhớ rằng `tester.element` trả về Element và `tester.renderObject` trả về RenderObject tại cùng widget:

### widget_tree_test.dart · 07-widget-man-hinh-bao-gia/test/widget_tree_test.dart (trích)

```dart
    testWidgets('Padding: widget cấu hình, RenderPadding làm việc layout', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp());
      final finder = find.byKey(const ValueKey('quote-body'));

      final element = tester.element(finder);
      expect(element, isA<SingleChildRenderObjectElement>());

      final renderObject = tester.renderObject<RenderPadding>(finder);
      expect(renderObject.padding, const EdgeInsets.all(16));
      // Render object giữ kích thước đã layout; widget thì không có size.
      expect(renderObject.size.width, greaterThan(0));
      expect(identical(element.renderObject, renderObject), isTrue);
    });
```

`Text` thì khác. Đọc source: `Text` là `StatelessWidget`, `build` của nó dựng ra một `RichText`, và `RichText` mới là `RenderObjectWidget` tạo `RenderParagraph`. Test kiểm đúng chuỗi đó:

### widget_tree_test.dart · 07-widget-man-hinh-bao-gia/test/widget_tree_test.dart (trích)

```dart
      final text = find.text('Quãng đường: 5 km');

      expect(tester.element(text), isA<StatelessElement>());
      // Text dựng ra RichText, và RichText mới có render object.
      expect(
        find.descendant(of: text, matching: find.byType(RichText)),
        findsOneWidget,
      );
      expect(tester.renderObject(text), isA<RenderParagraph>());
```

`Icon` và `Container` cũng là `StatelessWidget`. `Container` không vẽ gì riêng: tuỳ tham số, `build` của nó ghép `Padding`, `ColoredBox`, `DecoratedBox`, `ConstrainedBox`, `Align` lại với nhau. Nhờ vậy phần lớn thứ ta gọi là "widget" chỉ là cách gọi tên khác cho một tổ hợp các widget nhỏ hơn.

## StatelessWidget và build: màn hình báo giá

Màn hình đầu tiên có ba file. `main()` chỉ đưa widget gốc cho `runApp`:

### main.dart · 07-widget-man-hinh-bao-gia/lib/main.dart

```dart
import 'package:flutter/material.dart';

import 'toi_noi_app.dart';

void main() {
  runApp(const ToiNoiApp());
}
```

`ToiNoiApp` dựng `MaterialApp`, nơi đặt theme Material 3 cho cả app:

### toi_noi_app.dart · 07-widget-man-hinh-bao-gia/lib/toi_noi_app.dart (trích)

```dart
class ToiNoiApp extends StatelessWidget {
  const ToiNoiApp({super.key, this.distanceKm = 5});

  /// Quãng đường báo giá, đơn vị km. Bài sau sẽ lấy từ form nhập điểm đi/đến.
  final double distanceKm;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
        useMaterial3: true,
      ),
      home: QuoteScreen(distanceKm: distanceKm),
    );
  }
}
```

Còn `QuoteScreen` là màn hình thật:

### quote_screen.dart · 07-widget-man-hinh-bao-gia/lib/quote_screen.dart (trích)

```dart
class QuoteScreen extends StatelessWidget {
  const QuoteScreen({super.key, required this.distanceKm});

  /// Quãng đường cần báo giá, đơn vị km.
  final double distanceKm;

  /// Quãng đường nằm trong miền bảng giá chấp nhận (0..100 km). Viết dạng
  /// "điều kiện hợp lệ" để NaN cũng bị loại.
  bool get _isInRange => distanceKm >= 0 && distanceKm <= FarePolicy.maxKm;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    final colorScheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(title: const Text('Tới Nơi')),
      body: Padding(
        key: const ValueKey('quote-body'),
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.directions_car, size: 48),
            const SizedBox(height: 8),
            Text(
              'Quãng đường: ${formatKm(distanceKm)} km',
              style: textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            if (_isInRange)
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: colorScheme.primaryContainer,
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Text(
                  calculateFare(distanceKm).toString(),
                  style: textTheme.headlineMedium,
                ),
              )
            else
              Text(
                'Quãng đường phải từ 0 đến ${FarePolicy.maxKm} km',
                style: textTheme.bodyMedium?.copyWith(color: colorScheme.error),
              ),
            const SizedBox(height: 8),
            const Text('Cước minh hoạ, không phải bảng giá thực tế'),
          ],
        ),
      ),
    );
  }
}
```

Lab còn một hàm nhỏ `formatKm` đổi `5.0` thành "5" và `2.001` thành "2,001", và import `toi_noi_core` ở đầu file. Cước là `calculateFare(distanceKm).toString()`, tức `Money` của bài 06 tự in "25.500 VND". Không có công thức nào được viết lại trong lab.

`build` là phương thức duy nhất bắt buộc của `StatelessWidget`. Tài liệu trong source nói `build` có thể bị gọi ở mọi frame và không nên có tác dụng phụ nào ngoài việc dựng widget, và kết quả chỉ được phụ thuộc vào hai thứ: các field của widget (đều `final`) và dữ liệu lấy từ `context`, như `Theme.of(context)` ở trên. Cần thứ gì khác, ví dụ một bộ đếm thay đổi theo thời gian, ta phải dùng `StatefulWidget` ở bài 10.

Hình 2 vẽ cây widget của màn hình. `MaterialApp` và `Scaffold` là `StatefulWidget`, ta chỉ dùng chúng, chưa cần hiểu bên trong. `ToiNoiApp` và `QuoteScreen` do ta viết nên là `StatelessWidget`; `Padding` và `Column` là hai widget có RenderObject trực tiếp.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 2. Cây widget của màn hình báo giá. ToiNoiApp dựng MaterialApp, MaterialApp dựng QuoteScreen, QuoteScreen dựng Scaffold, Scaffold chứa Padding, Padding chứa Column, Column có bốn con: Icon, Text quãng đường, Container chứa Text cước và Text ghi chú."><defs><marker id="fl07-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="130" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">ToiNoiApp</text><text x="36" y="84" class="t-small">stateless</text>
<rect x="185" y="30" width="130" height="70" rx="10" class="f-surface s-muted"/><text x="201" y="62" class="t-b">MaterialApp</text><text x="201" y="84" class="t-small">stateful</text>
<rect x="350" y="30" width="130" height="70" rx="10" class="f-phys-soft s-phys"/><text x="366" y="62" class="t-b">QuoteScreen</text><text x="366" y="84" class="t-small">stateless</text>
<rect x="515" y="30" width="130" height="70" rx="10" class="f-surface s-muted"/><text x="531" y="62" class="t-b">Scaffold</text><text x="531" y="84" class="t-small">stateful</text>
<rect x="680" y="30" width="130" height="70" rx="10" class="f-pk-soft s-pk"/><text x="696" y="62" class="t-b">Padding</text><text x="696" y="84" class="t-small">có render</text>
<path d="M152 65 L183 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<path d="M317 65 L348 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<path d="M482 65 L513 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<path d="M647 65 L678 65" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<rect x="680" y="150" width="130" height="70" rx="10" class="f-pk-soft s-pk"/><text x="696" y="182" class="t-b">Column</text><text x="696" y="204" class="t-small">có render</text>
<path d="M745 102 L745 148" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<path d="M745 222 L745 245 M115 245 L775 245" class="f-none s-pk" stroke-width="2"/>
<path d="M115 245 L115 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<path d="M335 245 L335 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<path d="M555 245 L555 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<path d="M775 245 L775 268" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-2-a)"/>
<rect x="20" y="270" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="36" y="296" class="t-b">Icon</text><text x="36" y="318" class="t-small">stateless</text><text x="36" y="338" class="t-mono">directions_car</text>
<rect x="240" y="270" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="256" y="296" class="t-b">Text</text><text x="256" y="318" class="t-small">stateless</text><text x="256" y="338" class="t-small">Quãng đường: 5 km</text>
<rect x="460" y="270" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="476" y="296" class="t-b">Container</text><text x="476" y="318" class="t-small">stateless</text><text x="476" y="338" class="t-small">chứa Text (cước)</text>
<rect x="680" y="270" width="190" height="80" rx="10" class="f-phys-soft s-phys"/><text x="696" y="296" class="t-b">Text</text><text x="696" y="318" class="t-small">stateless</text><text x="696" y="338" class="t-small">ghi chú minh hoạ</text>
<rect x="20" y="368" width="16" height="16" rx="3" class="f-phys-soft s-phys"/><text x="44" y="381" class="t-small">StatelessWidget</text>
<rect x="200" y="368" width="16" height="16" rx="3" class="f-surface s-muted"/><text x="224" y="381" class="t-small">StatefulWidget (bài 10)</text>
<rect x="450" y="368" width="16" height="16" rx="3" class="f-pk-soft s-pk"/><text x="474" y="381" class="t-small">có RenderObject</text>
</svg>
</div>
<figcaption>Hình 2. Cây widget của QuoteScreen, rút gọn: bỏ AppBar, SizedBox và các widget Scaffold tự thêm bên trong. Mũi tên đi từ widget cha tới widget con mà nó trả về hoặc chứa.</figcaption>
</figure>

Hai chi tiết của code. Một, các widget con không đổi viết với `const` (`const Icon`, `const SizedBox`, `const Text`): Dart tạo đúng một instance dùng chung, và ta sẽ thấy ở mục sau vì sao instance giống hệt đáng giá. Hai, nhánh `else` kiểm miền 0 đến 100 km trước khi gọi `calculateFare`. `calculateFare` ném `ArgumentError` khi ngoài miền, mà một lỗi ném ra trong `build` không phải chuyện nhỏ: mục lỗi thường gặp ở cuối bài cho thấy nó trông ra sao.

## Dựng lại: widget mới, Element và RenderObject sống tiếp

Mỗi khi `build` chạy, ta có một cây widget mới. Framework không thay cả cây mà so từng cặp widget cũ và mới ở cùng vị trí. Quy tắc nằm trong `Element.updateChild`. Khi cả Element con cũ và widget mới đều khác null, hàm này có ba nhánh (Hình 3).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 3. Quy tắc khi build trả widget mới. Nếu cùng instance thì giữ nguyên element và không build lại. Nếu không cùng instance nhưng Widget.canUpdate đúng thì gọi element.update. Nếu canUpdate sai thì bỏ element cũ và dựng element mới."><defs><marker id="fl07-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl07-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="125" width="150" height="70" rx="10" class="f-phys-soft s-phys"/><text x="36" y="152" class="t-b">build chạy</text><text x="36" y="176" class="t-small">trả widget mới</text>
<rect x="300" y="120" width="260" height="80" rx="10" class="f-surface s-muted"/><text x="316" y="150" class="t-b">Cùng instance?</text><text x="316" y="174" class="t-mono">child.widget == newWidget</text>
<rect x="600" y="30" width="260" height="70" rx="10" class="f-pk-soft s-pk"/><text x="616" y="58" class="t-b">Giữ nguyên element</text><text x="616" y="82" class="t-small">không build lại widget này</text>
<rect x="300" y="240" width="260" height="80" rx="10" class="f-surface s-muted"/><text x="316" y="270" class="t-b">Widget.canUpdate?</text><text x="316" y="294" class="t-small">cùng runtimeType và key</text>
<rect x="600" y="225" width="260" height="110" rx="10" class="f-pk-soft s-pk"/><text x="616" y="253" class="t-b">element.update(newWidget)</text><text x="616" y="277" class="t-small">Stateless: build lại</text><text x="616" y="299" class="t-small">RenderObjectElement:</text><text x="616" y="319" class="t-small">updateRenderObject</text>
<rect x="10" y="240" width="210" height="80" rx="10" class="f-warn-soft s-warn"/><text x="26" y="266" class="t-b">Bỏ element cũ</text><text x="26" y="288" class="t-small">dựng element mới</text><text x="26" y="308" class="t-small">và render object mới</text>
<path d="M172 160 L298 160" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-3-a)"/>
<path d="M430 202 L430 238" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-3-a)"/><text x="442" y="226" class="t-small">không</text>
<path d="M562 160 L730 160 L730 102" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-3-a)"/><text x="580" y="150" class="t-small">có</text>
<path d="M562 280 L598 280" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-3-a)"/><text x="568" y="270" class="t-small">có</text>
<path d="M298 280 L222 280" class="f-none s-warn" stroke-width="2" marker-end="url(#fl07-3-w)"/><text x="234" y="270" class="t-small t-warn">không</text>
</svg>
</div>
<figcaption>Hình 3. Ba nhánh của updateChild. Hot reload dùng lại đúng quy tắc này sau khi ép mọi Element build lại.</figcaption>
</figure>

`Widget.canUpdate` là hàm một dòng: hai widget cập nhật được cho nhau khi `runtimeType` bằng nhau và `key` bằng nhau. Không có key thì chỉ cần cùng kiểu. Test sau kiểm hành vi đó:

### widget_tree_test.dart · 07-widget-man-hinh-bao-gia/test/widget_tree_test.dart (trích)

```dart
    test('Widget.canUpdate so runtimeType và key', () {
      const a = Text('a');
      const b = Text('b');
      expect(Widget.canUpdate(a, b), isTrue); // cùng kiểu, cùng key null
      expect(
        Widget.canUpdate(const Text('a', key: ValueKey(1)), const Text('a')),
        isFalse,
      );
      expect(Widget.canUpdate(a, const Icon(Icons.add)), isFalse);
    });
```

Giờ thử trên màn hình thật. Ta dựng `ToiNoiApp` với 5 km, giữ lại Element và RenderObject của dòng cước, rồi dựng lại với 10 km:

### widget_tree_test.dart · 07-widget-man-hinh-bao-gia/test/widget_tree_test.dart (trích)

```dart
      await tester.pumpWidget(const ToiNoiApp(distanceKm: 5));
      final text = find.text('25.500 VND');
      final elementBefore = tester.element(text);
      final renderBefore = tester.renderObject(text);

      await tester.pumpWidget(const ToiNoiApp(distanceKm: 10));

      expect(find.text('25.500 VND'), findsNothing);
      final after = find.text('48.000 VND');
      expect(after, findsOneWidget);
      // Widget cũ bị bỏ, nhưng element và render object vẫn là đối tượng cũ.
      expect(identical(tester.element(after), elementBefore), isTrue);
      expect(identical(tester.renderObject(after), renderBefore), isTrue);
```

Widget `Text('25.500 VND')` đã biến mất, nhưng Element và `RenderParagraph` phía dưới vẫn là hai đối tượng cũ, chỉ nhận chữ mới. Đó là lý do một `build` chạy hàng trăm lần mà app vẫn mượt: thứ đắt (layout, vẽ) chỉ làm lại khi cấu hình thật sự đổi.

Còn nhánh đầu tiên của Hình 3? Test dưới đây dùng một widget đếm số lần `build`. Dựng lại với cùng một instance `const` thì `build` không chạy thêm:

### widget_tree_test.dart · 07-widget-man-hinh-bao-gia/test/widget_tree_test.dart (trích)

```dart
      const probe = _Probe();
      await tester.pumpWidget(probe);
      expect(_probeBuilds, 1);

      await tester.pumpWidget(probe);
      expect(_probeBuilds, 1); // cùng instance: framework bỏ qua
```

Đây là lý do widget con không đổi nên viết `const`: cùng instance thì framework bỏ qua cả nhánh. Source của `Widget` cũng khuyên cache những widget biết chắc không đổi. Cùng file test còn một test case ngược lại: đổi con của `Directionality` từ `Text` sang `Padding` thì `canUpdate` sai và Element của dòng chữ là đối tượng mới. Quy tắc này cũng là nền cho `key` ở bài về danh sách.

## Hot reload và hot restart

Với nền đó, hot reload không còn bí ẩn. Hot reload chạy khi ta bấm `r` trong terminal của `flutter run`, hoặc khi lưu file trong IDE đã bật hot reload khi lưu. Theo [tài liệu hot reload](https://docs.flutter.dev/tools/hot-reload), nó có ba bước (Hình 4). Máy phát triển biên dịch lại các thư viện có thay đổi. Mã đã biên dịch được gửi tới Dart VM trên thiết bị, VM nạp lại các thư viện nhưng chưa chạy dòng nào. Cuối cùng framework rebuild, relayout và repaint các widget và render object đang có; chỉ mã tham gia việc dựng lại widget chạy lại.

Bước cuối ứng với `Element.reassemble` trong source: nó gọi `markNeedsBuild` rồi đi xuống từng Element con. Mọi Element bị đánh dấu "bẩn", nên `build` chạy lại cả với widget `const`, và vì Element không bị thay, state hiện có được giữ. Test dưới đây mô phỏng phần này bằng `reassembleApplication`, hàm mà tooling gọi khi hot reload:

### widget_tree_test.dart · 07-widget-man-hinh-bao-gia/test/widget_tree_test.dart (trích)

```dart
      await tester.pumpWidget(const _Probe());
      final elementBefore = tester.element(find.byType(_Probe));
      expect(_probeBuilds, 1);

      // Giống bước 3 của hot reload: dựng lại mọi widget đang có. Future chỉ
      // hoàn tất sau một frame nên phải pump trước khi await.
      final reassemble = tester.binding.reassembleApplication();
      await tester.pump();
      await reassemble;

      expect(_probeBuilds, 2);
      // Hot reload dựng lại nhưng không tạo element mới.
      expect(
        identical(tester.element(find.byType(_Probe)), elementBefore),
        isTrue,
      );
```

`build` chạy lại dù widget là `const`, và Element vẫn là đối tượng cũ. Đây chỉ là mô phỏng trong test; lab chưa chạy `flutter run` trên thiết bị thật.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 4. Hot reload: lưu trong IDE hoặc bấm r, biên dịch lại thư viện đã đổi, VM nạp mã mới mà chưa chạy gì, rồi rebuild toàn cây hiện có; state được giữ. Hot restart: nạp mã đã sửa, khởi động lại ứng dụng Dart, main chạy lại và state mất."><defs><marker id="fl07-4-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl07-4-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<text x="20" y="26" class="t-b">Hot reload (r)</text>
<rect x="15" y="40" width="190" height="84" rx="10" class="f-pk-soft s-pk"/><text x="31" y="72" class="t-b">Lưu trong IDE</text><text x="31" y="96" class="t-small">hoặc bấm r</text>
<rect x="235" y="40" width="190" height="84" rx="10" class="f-pk-soft s-pk"/><text x="251" y="72" class="t-b">Biên dịch lại</text><text x="251" y="96" class="t-small">thư viện đã đổi</text>
<rect x="455" y="40" width="190" height="84" rx="10" class="f-pk-soft s-pk"/><text x="471" y="72" class="t-b">VM nạp mã mới</text><text x="471" y="96" class="t-small">chưa chạy dòng nào</text>
<rect x="675" y="40" width="190" height="84" rx="10" class="f-phys-soft s-phys"/><text x="691" y="72" class="t-b">Rebuild toàn cây</text><text x="691" y="96" class="t-small">relayout, repaint</text>
<path d="M207 82 L233 82" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-4-a)"/>
<path d="M427 82 L453 82" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-4-a)"/>
<path d="M647 82 L673 82" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-4-a)"/>
<text x="20" y="152" class="t-small">Kết quả: cây widget và state hiện có được giữ; main() và initState không chạy lại.</text>
<text x="20" y="196" class="t-b">Hot restart (R)</text>
<rect x="15" y="210" width="190" height="84" rx="10" class="f-pk-soft s-pk"/><text x="31" y="242" class="t-b">Bấm R</text><text x="31" y="266" class="t-small">nạp mã đã sửa</text>
<rect x="235" y="210" width="190" height="84" rx="10" class="f-pk-soft s-pk"/><text x="251" y="242" class="t-b">Khởi động lại</text><text x="251" y="266" class="t-small">ứng dụng Dart</text>
<rect x="455" y="210" width="190" height="84" rx="10" class="f-warn-soft s-warn"/><text x="471" y="242" class="t-b">main() chạy lại</text><text x="471" y="266" class="t-small">state mất hết</text>
<path d="M207 252 L233 252" class="f-none s-pk" stroke-width="2" marker-end="url(#fl07-4-a)"/>
<path d="M427 252 L453 252" class="f-none s-warn" stroke-width="2" marker-end="url(#fl07-4-w)"/>
<text x="20" y="324" class="t-small">Sửa code native (Kotlin, Java, Swift, Objective-C) cần full restart: build lại cả app.</text>
</svg>
</div>
<figcaption>Hình 4. Hot reload nạp mã vào VM đang chạy và dựng lại cây hiện có; hot restart khởi động lại ứng dụng Dart nên mất state.</figcaption>
</figure>

Vì hot reload không chạy lại `main()` hay `initState`, có những thay đổi hot reload không thấy. Bảng dưới tóm tắt theo tài liệu chính thức; tài liệu cũng nói hot reload chỉ dùng được ở chế độ debug.

| Ta sửa | Cần gì | Lý do |
|---|---|---|
| Chữ, màu, padding hay thêm widget trong `build` | hot reload | `build` được chạy lại |
| Giá trị khởi tạo của biến toàn cục hoặc `static` | hot restart | khởi tạo lười và không chạy lại; `const` thì luôn được nạp lại |
| `runApp(...)` trong `main()` với widget gốc khác | hot restart | `main()` không chạy lại, widget gốc cũ vẫn đứng đó |
| Đổi `enum` thành class hoặc ngược lại; đổi tham số kiểu của class generic | hot restart | hot reload không áp dụng được |
| Mã Kotlin, Java, Swift, Objective-C | full restart | native phải biên dịch lại |
| Lỗi biên dịch | sửa rồi reload tiếp | hot reload bị từ chối và báo dòng lỗi |

Tài liệu còn nhắc một điểm tinh tế: state cũ ghép với code mới có thể cho hành vi khác với một lần khởi động sạch. Khi thấy màn hình lạ sau nhiều lần reload, hãy hot restart trước khi tìm lỗi.

## Widget test đầu tiên

Test là cách nhanh nhất để biết màn hình đúng mà không cần thiết bị. `flutter_test` cung cấp `testWidgets`, truyền vào một `WidgetTester`. `pumpWidget` đặt widget ta đưa làm gốc cây rồi chạy một frame (tài liệu ghi là gọi `runApp`, còn code của `flutter_test` gọi `attachRootWidget`; tài liệu này cũng viết rằng nó ép dựng lại cả cây dù widget không đổi, nhưng bài theo kết quả đo ở mục trên: `build` của widget `const` cùng instance không chạy lại); sau đó `find` tìm widget trên cây và `expect` kiểm bằng matcher như `findsOneWidget`. Test case chính:

### quote_screen_test.dart · 07-widget-man-hinh-bao-gia/test/quote_screen_test.dart (trích)

```dart
  testWidgets('5 km hiện quãng đường và cước 25.500 VND', (tester) async {
    await tester.pumpWidget(const ToiNoiApp());

    expect(find.text('Tới Nơi'), findsOneWidget); // tiêu đề AppBar
    expect(find.text('Quãng đường: 5 km'), findsOneWidget);
    expect(find.text('25.500 VND'), findsOneWidget);
    expect(find.byIcon(Icons.directions_car), findsOneWidget);
    expect(find.textContaining('không phải bảng giá thực tế'), findsOneWidget);
  });
```

`find.text` so khớp cả chuỗi, nên `find.text('25.500')` không tìm thấy gì còn `find.textContaining('25.500')` thì có. Test tiếp theo dựng `QuoteScreen` trong `MaterialApp` rồi chạy qua một bảng test case biên của chính công thức ở bài 06:

### quote_screen_test.dart · 07-widget-man-hinh-bao-gia/test/quote_screen_test.dart (trích)

```dart
    const cases = <(double, String)>[
      (0, '12.000 VND'),
      (2, '12.000 VND'),
      (2.001, '12.005 VND'),
      (5, '25.500 VND'),
      (100, '453.000 VND'),
    ];

    for (final (km, expected) in cases) {
      testWidgets('$km km hiện $expected', (tester) async {
        await tester.pumpWidget(_host(km));

        expect(find.text(expected), findsOneWidget);
        expect(tester.takeException(), isNull);
      });
    }
```

Còn `_host` chỉ bọc `QuoteScreen` bằng `MaterialApp`. Thiếu lớp bọc đó, `Scaffold` không tìm được `Directionality` và test ném `FlutterError: No Directionality widget found`; thêm `Directionality` thì lỗi kế tiếp là thiếu `MaterialLocalizations`. `Theme.of` thì có giá trị dự phòng nên không gây lỗi. Một test case nữa chứng minh nhánh `else` của màn hình: với -1, 100,5, NaN và hai vô cực, màn hình hiện thông báo thay cho cước, không có dòng "VND" nào, và `tester.takeException()` là `null`.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Bốn lỗi hay gặp ở màn hình đầu tiên</span>
<ul>
<li><strong>Gọi hàm có thể ném lỗi ngay trong <code>build</code>.</strong> Dấu hiệu: ở debug, màn hình đỏ với thông báo lỗi; ở release, một ô xám trơn. Nguyên nhân: lỗi ném trong <code>build</code> bị framework bắt và thay widget hỏng bằng <code>ErrorWidget</code> (tài liệu của <code>ErrorWidget.builder</code> mô tả hai chế độ). Test <code>tính cước 101 km trong build ném ArgumentError</code> dựng đúng tình huống này: <code>tester.takeException()</code> trả <code>ArgumentError</code> và trên cây có một <code>ErrorWidget</code>. Cách sửa: kiểm điều kiện trước khi gọi, như <code>_isInRange</code>, rồi hiện thông báo.</li>
<li><strong>Sửa <code>main()</code> hoặc giá trị biến toàn cục rồi chỉ hot reload.</strong> Dấu hiệu: bấm <code>r</code> báo thành công nhưng màn hình không đổi. Nguyên nhân: hot reload không chạy lại <code>main()</code> hay khởi tạo toàn cục. Cách sửa: bấm <code>R</code>; nếu vẫn không đổi thì dừng và <code>flutter run</code> lại.</li>
<li><strong>Widget test thiếu <code>MaterialApp</code>.</strong> Dấu hiệu: <code>FlutterError: No Directionality widget found</code> khi <code>pumpWidget(const QuoteScreen(...))</code> không bọc gì. Nguyên nhân: <code>Scaffold</code> cần tổ tiên cung cấp <code>Directionality</code> và <code>MaterialLocalizations</code> (thêm <code>Directionality</code> thì lỗi kế tiếp là <code>No MaterialLocalizations found</code>); <code>Theme.of</code> có giá trị dự phòng. Cách sửa: bọc bằng <code>MaterialApp(home: ...)</code> như <code>_host</code>.</li>
<li><strong>Tìm chữ bằng <code>find.text</code> với một phần chuỗi.</strong> Dấu hiệu: <code>findsOneWidget</code> báo tìm thấy 0 widget dù chữ có trên màn hình. Nguyên nhân: <code>find.text</code> so khớp cả chuỗi. Cách sửa: viết đủ chuỗi ("25.500 VND") hoặc dùng <code>find.textContaining</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/07-widget-man-hinh-bao-gia
flutter pub get
flutter analyze
flutter test --reporter expanded
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` báo `No issues found!`, `flutter test` có **25 test** xanh. Mức: **Kiểm chứng tự động**, không cần thiết bị.

| Test | Chứng minh điều gì |
|---|---|
| `quote_screen_test.dart`, test case chính | `ToiNoiApp` mặc định 5 km hiện "Quãng đường: 5 km" và "25.500 VND", có icon và dòng nhắc giá minh hoạ |
| `quote_screen_test.dart`, bảng test case biên | 0, 2, 2,001, 5, 100 km hiện 12.000, 12.000, 12.005, 25.500, 453.000 VND; `build` không ném |
| `quote_screen_test.dart`, ngoài miền | -1, 100,5, NaN, hai vô cực hiện thông báo thay cước, không có `ErrorWidget` |
| `quote_screen_test.dart`, `ToiNoiApp` và `formatKm` | `MaterialApp` Material 3, `distanceKm` đi xuống `QuoteScreen` (10 km thành 48.000 VND); `formatKm` cho "5", "2,001", không ném với NaN và vô cực |
| `widget_tree_test.dart`, ba cây | `QuoteScreen` và `Text` là `StatelessElement`; `Padding` là `SingleChildRenderObjectElement` với `RenderPadding` 16; chữ vẽ bởi `RenderParagraph` của `RichText` |
| `widget_tree_test.dart`, dựng lại | cùng kiểu và key thì Element, RenderObject là đối tượng cũ; khác kiểu thì Element mới; `Widget.canUpdate` đúng quy tắc |
| `widget_tree_test.dart`, hot reload | `const` cùng instance không build lại; `reassembleApplication` ép build lại nhưng giữ Element |
| `widget_tree_test.dart`, build ném lỗi | `calculateFare(101)` trong `build` ném `ArgumentError` và để lại `ErrorWidget` |

**Chưa làm**: chạy app trên iOS Simulator, Android emulator hay thiết bị thật (máy kiểm chưa có Android SDK và Xcode đầy đủ), nên hot reload và hot restart chỉ được mô tả theo tài liệu chính thức và mô phỏng phần dựng lại bằng test; `StatefulWidget` và `setState` (bài 10); nhập quãng đường từ form (bài 11); định dạng số theo locale bằng `intl` (bài 14). Để chạy trên máy, README của lab ghi lệnh `flutter create` sinh runner một lần; thư mục `android/` và `ios/` không được commit.

## Bài tập

1. Trong `QuoteScreen`, ta đổi `size: 48` của `Icon` thành 64 rồi hot reload. Element của `Icon` có bị thay không? Nếu ta đổi `Column` thành `Row` thì sao?
2. Với mỗi thay đổi, cần hot reload, hot restart hay full restart: (a) đổi chữ "Cước minh hoạ, không phải bảng giá thực tế" thành "Giá tham khảo"; (b) đổi `runApp(const ToiNoiApp())` thành `runApp(const ToiNoiApp(distanceKm: 10))`; (c) đổi một `enum` thành `class`; (d) sửa file `AndroidManifest.xml`.
3. Viết widget test cho `QuoteScreen(distanceKm: 2.001)` kiểm cước 12.005 VND. Test cần bọc gì, và cần finder nào nếu ta chỉ muốn kiểm số "12.005"?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Đổi <code>size</code> thì không: widget mới vẫn là <code>Icon</code> không key, <code>Widget.canUpdate</code> đúng nên Element cũ được giữ và nhận cấu hình mới. Đổi <code>Column</code> thành <code>Row</code> thì <code>runtimeType</code> khác, <code>canUpdate</code> sai, nên Element cũ của <code>Column</code> bị bỏ và Element mới được dựng cùng các Element con bên dưới.</p>
<p><strong>2.</strong> (a) <strong>hot reload</strong>, vì nằm trong <code>build</code>. (b) <strong>hot restart</strong>: <code>main()</code> không chạy lại khi hot reload nên widget gốc cũ vẫn đứng đó. (c) <strong>hot restart</strong>, tài liệu liệt kê đổi enum và class là trường hợp hot reload không áp dụng được. (d) <strong>full restart</strong>, vì là cấu hình native.</p>
<p><strong>3.</strong> Cần bọc bằng <code>MaterialApp(home: QuoteScreen(distanceKm: 2.001))</code> để <code>Scaffold</code> có <code>Directionality</code> và <code>MaterialLocalizations</code>. Với chuỗi đầy đủ dùng <code>find.text('12.005 VND')</code> và <code>findsOneWidget</code>. Chỉ kiểm số "12.005" thì phải dùng <code>find.textContaining('12.005')</code>, vì <code>find.text</code> so khớp cả chuỗi.</p>
</details>

Bài trước: [Package, pubspec, lint và test: thư viện toi_noi_core kiểm chứng được](/articles/dart-package-pubspec-lint-test-toi-noi-core). Bài tiếp: [Layout và constraints: Row, Column, Expanded và thẻ chuyến](/articles/flutter-layout-constraints-row-column-the-chuyen).

## Tài liệu gốc

- [Building user interfaces with Flutter](https://docs.flutter.dev/ui/widgets-intro) (docs.flutter.dev).
- [Inside Flutter](https://docs.flutter.dev/resources/inside-flutter): widget, element, render object và quy tắc so khớp theo kiểu và key (docs.flutter.dev).
- [Hot reload](https://docs.flutter.dev/tools/hot-reload) (docs.flutter.dev).
- [Handling errors in Flutter](https://docs.flutter.dev/testing/errors) (docs.flutter.dev).
- [An introduction to widget testing](https://docs.flutter.dev/cookbook/testing/widget/introduction) (docs.flutter.dev).
- [StatelessWidget](https://api.flutter.dev/flutter/widgets/StatelessWidget-class.html), [Element](https://api.flutter.dev/flutter/widgets/Element-class.html) và [Widget.canUpdate](https://api.flutter.dev/flutter/widgets/Widget/canUpdate.html) (api.flutter.dev).
- [RenderObject](https://api.flutter.dev/flutter/rendering/RenderObject-class.html) và [WidgetTester.pumpWidget](https://api.flutter.dev/flutter/flutter_test/WidgetTester/pumpWidget.html) (api.flutter.dev).
