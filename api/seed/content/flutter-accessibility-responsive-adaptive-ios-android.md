---
slug: flutter-accessibility-responsive-adaptive-ios-android
title: "Accessibility, responsive và adaptive: một giao diện cho iOS và Android"
summary: "Bài 16: làm cho app Tới Nơi nhỏ dùng được với trình đọc màn hình, chữ cỡ lớn, tai thỏ, tablet, iOS và Android: Semantics, guideline vùng chạm, text scale, SafeArea, breakpoint, Theme.platform; kiểm bằng widget test, nêu rõ phần còn thiếu của Phần 2."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, accessibility, responsive, adaptive, ios, android]
language: vi
---

Bản thử nghiệm app khách Tới Nơi vừa qua tay ba người dùng. Chị Lan đặt cỡ chữ lớn nhất vì mắt kém, và thẻ chuyến cắt mất dòng giá cước. Anh Minh dùng điện thoại có tai thỏ, nút "Đặt chuyến" đè lên vạch home indicator. Bạn Hà dùng trình đọc màn hình, và nút xoá chỉ được đọc là "nút". Tên và tình huống là hư cấu.

Cả ba lỗi có chung một gốc: ta dựng giao diện cho một cỡ chữ, một cỡ màn hình và một cách tương tác. Bài này sửa bằng `Semantics`, guideline vùng chạm, chữ phóng to, `SafeArea`, breakpoint, và chỉ tạo khác biệt giữa iOS và Android ở chỗ hai hệ điều hành có quy ước riêng. Lab là app nhỏ gom lại phần lớn màn hình của Phần 2.

Đây là bài 16 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 15 về animation](/articles/flutter-animation-implicit-hero-tim-tai-xe), bài cuối của Phần 2. Bạn cần biết theme `toi_noi_ui` ([bài 13](/articles/flutter-theme-material-3-dark-mode-design-system-toi-noi)) và `Navigator` ([bài 12](/articles/flutter-navigator-route-truyen-du-lieu-xac-nhan-chuyen)). Bối cảnh nằm ở hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); bảng giá là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một màn hình dùng <code>Scaffold</code> với <code>body</code> là <code>Column</code>: phần trên co giãn, dưới cùng là nút "Đặt chuyến" có đệm 16. Màn cao 800, điện thoại báo vùng không an toàn ở đáy là 34. Cạnh dưới của nút nằm ở đâu, và <code>Scaffold</code> có tự chừa 34 đó không?</p>
<details><summary>Xem đáp án</summary><p>Cạnh dưới của nút ở y = 784, tức là nằm trong 34 pixel cuối (từ 766). <code>Scaffold</code> không tự đệm phần thân khi không có thanh đáy; chỉ các thành phần tự có <code>SafeArea</code> bên trong (như <code>AppBar</code>, <code>NavigationBar</code>) mới chừa. Bọc nút trong <code>SafeArea</code> thì cạnh dưới lên y = 750 (766 trừ 16 đệm). Test <code>safe_area_test.dart</code> kiểm đúng hai con số 784 và 750.</p></details>
</div>

## Semantics: cây thứ hai cho trình đọc màn hình

Bên cạnh cây widget, Flutter dựng **cây semantics** cho VoiceOver trên iOS và TalkBack trên Android. Mỗi node có `label` (câu đọc), `value`, cờ như `isButton` và hành động như `tap` hay `increase`. Widget chuẩn đã tự điền phần lớn; ta sửa chỗ mặc định chưa đủ.

Ba công cụ: `Semantics(label: ...)` thêm hoặc ghi đè thông tin, `excludeSemantics: true` loại cả cây con, `MergeSemantics` gộp nhiều node thành một. Hình 1: `TripTile` dùng `Semantics` và `excludeSemantics` để bốn widget chữ bên trong chỉ còn một nút; `MergeSemantics` xuất hiện ở `Slider` bên dưới.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Cây widget của TripTile gồm Semantics bọc Card, InkWell, Column và bốn widget chữ. Semantics xuất ra một nút duy nhất có câu đọc đầy đủ và hành động tap. Bốn widget chữ bên trong bị loại khỏi cây semantics bởi excludeSemantics."><defs><marker id="fl16-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl16-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="20" width="400" height="300" rx="10" class="f-surface s-line"/><text x="36" y="48" class="t-b">Cây widget của TripTile</text>
<rect x="40" y="64" width="360" height="44" rx="8" class="f-pk-soft s-pk"/><text x="56" y="92" class="t-small">Semantics(label, onTap)</text>
<rect x="40" y="124" width="360" height="40" rx="8" class="f-phys-soft s-phys"/><text x="56" y="149" class="t-small">Card, InkWell, Column</text>
<rect x="40" y="184" width="170" height="40" rx="8" class="f-phys-soft s-phys"/><text x="52" y="209" class="t-small">Text: Đặt xe</text>
<rect x="230" y="184" width="170" height="40" rx="8" class="f-phys-soft s-phys"/><text x="242" y="209" class="t-small">TripStatusChip</text>
<rect x="40" y="240" width="170" height="40" rx="8" class="f-phys-soft s-phys"/><text x="52" y="265" class="t-small">Text: mã · km</text>
<rect x="230" y="240" width="170" height="40" rx="8" class="f-phys-soft s-phys"/><text x="242" y="265" class="t-small">MoneyText</text>
<path d="M220 110 L220 122" class="f-none s-line" stroke-width="2"/><path d="M220 166 L220 182" class="f-none s-line" stroke-width="2"/>
<rect x="500" y="20" width="360" height="300" rx="10" class="f-surface s-line"/><text x="516" y="48" class="t-b">Cây semantics</text>
<rect x="520" y="64" width="320" height="130" rx="8" class="f-pk-soft s-pk"/><text x="536" y="92" class="t-b">1 nút</text><text x="536" y="118" class="t-small">cờ isButton, hành động tap</text><text x="536" y="144" class="t-small">Đặt xe, 5 km, trạng thái</text><text x="536" y="168" class="t-small">Đang tìm tài xế, 25.500 đồng</text>
<rect x="520" y="226" width="320" height="76" rx="8" class="f-warn-soft s-warn"/><text x="536" y="254" class="t-b">Bị loại khỏi cây</text><text x="536" y="280" class="t-small">bốn widget chữ bên trong</text>
<path d="M402 86 L518 110" class="f-none s-pk" stroke-width="2" marker-end="url(#fl16-1-a)"/><text x="432" y="82" class="t-small">xuất ra</text>
<path d="M402 232 L518 262" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl16-1-w)"/><text x="428" y="226" class="t-small">loại bỏ</text>
</svg>
</div>
<figcaption>Hình 1. TripTile trong hai cây. Semantics bọc cả thẻ và đặt nhãn một lần; chữ và chip bên trong bị loại để trình đọc không đọc mỗi thứ hai lần.</figcaption>
</figure>

Số tiền là ví dụ rõ nhất. Màn hình hiện `25.500 VND`, còn câu đọc là `25.500 đồng`, để không phụ thuộc cách từng trình đọc xử lý chữ hoa liền nhau. Lab chỉ kiểm nhãn trong cây semantics, chưa nghe bằng VoiceOver hay TalkBack. Hai widget dưới đây bọc `MoneyText` và `TripStatusChip` mà không sửa package:

### accessible.dart · 16-a11y-responsive-adaptive/lib/accessible.dart (trích)

```dart
String spokenMoney(Money money) =>
    '${money.toString().replaceAll(' VND', '')} đồng';

class AccessibleMoney extends StatelessWidget {
  const AccessibleMoney(this.money, {super.key, this.prefix, this.style});

  final Money money;
  final String? prefix;
  final TextStyle? style;

  @override
  Widget build(BuildContext context) {
    final spoken = spokenMoney(money);
    return Semantics(
      label: prefix == null ? spoken : '$prefix: $spoken',
      excludeSemantics: true,
      child: MoneyText(money, style: style),
    );
  }
}

class AccessibleStatus extends StatelessWidget {
  const AccessibleStatus(this.status, {super.key, this.liveRegion = false});

  final TripStatus status;
  final bool liveRegion;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: 'Trạng thái: ${status.label}',
      liveRegion: liveRegion,
      excludeSemantics: true,
      child: TripStatusChip(status: status),
    );
  }
}
```

`liveRegion: true` báo rằng nội dung node có thể đổi khi người dùng đang xem; màn chi tiết bật nó vì trạng thái đổi sau khi huỷ. Thẻ chuyến gom cả thẻ thành một nút như Hình 1:

### trip_tile.dart · 16-a11y-responsive-adaptive/lib/trip_tile.dart (trích)

```dart
    return Semantics(
      button: true,
      selected: selected,
      label: spokenTrip(trip),
      onTap: onTap,
      excludeSemantics: true,
      child: Card(
        // ... InkWell, Column, Wrap, MoneyText như trong lab
      ),
    );
```

`excludeSemantics` loại luôn hành động của `InkWell` bên trong, nên ta truyền `onTap` cho chính `Semantics`. `Slider` có sẵn hành động tăng và giảm, nhưng tham số `label` của nó vừa là bong bóng hiển thị vừa là nhãn semantics. Ta bỏ `label`, đặt nhãn bằng `Semantics` bên ngoài, gộp bằng `MergeSemantics`, và dùng `semanticFormatterCallback` cho câu đọc giá trị:

### quote_screen.dart · 16-a11y-responsive-adaptive/lib/quote_screen.dart (trích)

```dart
                    MergeSemantics(
                      child: Semantics(
                        label: 'Quãng đường',
                        child: Slider(
                          value: _km,
                          min: _minKm,
                          max: _maxKm,
                          divisions: (_maxKm - _minKm).round(),
                          semanticFormatterCallback: formatKm,
                          onChanged: (value) => setState(() => _km = value),
                        ),
                      ),
                    ),
```

`testWidgets` mặc định đã bật cây semantics (tham số `semanticsEnabled` có giá trị `true`), nên ta dùng ngay `find.bySemanticsLabel`, `matchesSemantics` và `tester.semantics` để kiểm và thao tác như một công nghệ hỗ trợ:

### semantics_test.dart · 16-a11y-responsive-adaptive/test/semantics_test.dart (trích)

```dart
      await pumpThemed(
        tester,
        const Scaffold(
          body: Center(child: AccessibleMoney(Money(25500), prefix: 'Cước')),
        ),
      );

      // Chữ trực quan vẫn vẽ ra màn hình...
      expect(find.text('25.500 VND'), findsOneWidget);
      // ...nhưng cây semantics chỉ có câu đọc, không có chữ viết tắt.
      expect(find.bySemanticsLabel('Cước: 25.500 đồng'), findsOneWidget);
      expect(find.bySemanticsLabel(RegExp('VND')), findsNothing);
```

Một test khác gọi `tester.semantics.increase(...)`: giá trị đọc đổi từ `5 km` sang `6 km` và cước từ 25.500 sang 30.000 VND (do `calculateFare` tính).

Chuỗi do framework tạo, như "Tab 1 trong tổng số 3", cũng phải bằng tiếng Việt. `MaterialApp` đặt `Locale('vi')` và `GlobalMaterialLocalizations.delegates`, gồm cả bộ Material, Widgets và Cupertino (`showCupertinoDialog` cần bộ Cupertino); một test khẳng định nhãn tab.

## Vùng chạm, nhãn và độ tương phản: guideline chạy tự động

`flutter_test` có sẵn các **accessibility guideline** chạy trên cây semantics bằng `meetsGuideline`. Ta dùng bốn guideline: `androidTapTargetGuideline` (vùng chạm tối thiểu 48 x 48), `iOSTapTargetGuideline` (44 x 44), `labeledTapTargetGuideline` (node có hành động chạm phải có `label` hoặc `tooltip`) và `textContrastGuideline`. Tài liệu Flutter cũng khuyến nghị vùng chạm 48 và tỉ lệ tương phản 4,5:1.

Guideline chỉ có giá trị khi ta biết nó bắt được lỗi, nên lab có bốn widget để đo cả hai chiều:

| Widget | Android 48 | iOS 44 | Có nhãn |
|---|---|---|---|
| `UnlabeledTinyIcon`: `GestureDetector` + icon 20, không nhãn | không đạt | không đạt | không đạt |
| `Square44Button`: vuông 44, có `Semantics` | không đạt | đạt | đạt |
| `IconButtonWithoutTooltip`: nút chuẩn, quên `tooltip` | đạt | đạt | không đạt |
| `LabeledCloseButton`: `IconButton` có `tooltip` | đạt | đạt | đạt |

Hàng thứ hai đáng nhớ: 44 đủ cho iOS nhưng chưa đủ cho Android, nên app dùng chung lấy 48. Hàng thứ ba: nút biểu tượng phải có `tooltip`, vì guideline chấp nhận `tooltip` thay cho `label`.

### guidelines_test.dart · 16-a11y-responsive-adaptive/test/guidelines_test.dart (trích)

```dart
Future<void> expectMeetsTapGuidelines(WidgetTester tester) async {
  await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
  await expectLater(tester, meetsGuideline(iOSTapTargetGuideline));
  await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
}

    testWidgets('vuông 44: qua chuẩn iOS, trượt chuẩn Android', (tester) async {
      await pumpCentered(tester, Square44Button(onTap: () {}));

      await expectLater(tester, meetsGuideline(iOSTapTargetGuideline));
      await expectLater(
        tester,
        doesNotMeetGuideline(androidTapTargetGuideline),
      );
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
    });
```

Ba guideline này chạy trên app thật ở ba bề rộng (390, 700, 1000) và ba tab, thêm trang chi tiết và cỡ chữ gấp đôi: tất cả đều đạt, với một giới hạn. Guideline bỏ qua node sát mép, nên thanh điều hướng không được đo ở đó; ba đích điều hướng được đo riêng bằng kích thước node semantics (đủ 48 x 48 ở 390 và 700 pixel). `textContrastGuideline` chạy trên màn lịch sử ở theme sáng và tối, kèm test case đối chứng: chữ `0xFFBBBBBB` trên nền trắng (tỉ lệ khoảng 1,9) bị bắt đúng. Guideline này phân màu khá thô, nên là lưới an toàn; tỉ lệ từng cặp màu token đã kiểm ở bài 13.

Hai cái bẫy. Một, guideline **bỏ qua** node sát mép cửa sổ hoặc mép vùng cuộn; helper `pumpCentered` đặt widget giữa màn hình, và một test riêng chứng minh icon 20 ở góc trên trái "đạt" guideline 48 một cách oan. Hai, guideline chỉ xét node có hành động `tap` hoặc `longPress`, nên không đo `Slider`.

## Cỡ chữ hệ thống: đừng giả định chữ cao 14 pixel

Cỡ chữ người dùng đặt trong hệ điều hành đi tới widget qua `MediaQuery.textScalerOf(context)`. Kiểu trả về là `TextScaler`, không phải một số: Flutter đã đánh dấu `textScaleFactor` là deprecated để chuẩn bị cho text scaling phi tuyến, nên hãy gọi `scaler.scale(14)`.

Ba quy tắc: không đặt chiều cao cố định cho container chứa chữ; dùng `Wrap`, `Flexible`, `Expanded` để nội dung xuống dòng; cho màn hình cuộn được. Trường hợp sai điển hình là `FixedHeightTripTile` cao đúng 72:

### broken_examples.dart · 16-a11y-responsive-adaptive/lib/broken_examples.dart (trích)

```dart
    return SizedBox(
      height: 72,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(trip.serviceKind.label, style: textTheme.titleMedium),
          Text(trip.id),
          MoneyText(trip.fare),
        ],
      ),
    );
```

Ở cỡ chữ 1 widget này vừa khít; ở cỡ chữ 2 test báo `A RenderFlex overflowed by ... pixels on the bottom`. `TripTile` thật không có chiều cao cố định và dùng `Wrap` cho hàng đầu: chip nằm cùng hàng với tên dịch vụ khi đủ chỗ và xuống dòng khi hết chỗ. Test giả lập cỡ chữ bằng `platformDispatcher`, cùng đường mà cài đặt thật đi qua:

### support.dart · 16-a11y-responsive-adaptive/test/support.dart (trích)

```dart
void setTextScale(WidgetTester tester, double scale) {
  tester.platformDispatcher.textScaleFactorTestValue = scale;
  addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
}
```

Trường hợp cỡ chữ gấp đôi mà [bài 08](/articles/flutter-layout-constraints-row-column-the-chuyen) để lại được giải quyết nốt ở đây. `TripTile` ở 320 pixel với đủ 8 trạng thái và cỡ chữ 1, 1,5, 2, 3 không overflow; chip nằm cùng hàng tiêu đề ở cỡ 1 và xuống dưới ở cỡ 2. Toàn app ở 320 x 568 với cỡ chữ 2 cũng không overflow ở cả ba tab. Ở trang chi tiết, nút "Huỷ chuyến" bị đẩy xuống dưới khung nhìn nên test phải cuộn tới (`ensureVisible`).

Lưu ý: `flutter test` dùng font kiểm thử riêng, phần lớn ký tự là ô vuông bằng cỡ chữ, thường rộng hơn chữ thật; phép thử khắt khe nhưng không thay được việc nhìn app trên máy thật.

## Responsive: bề rộng cửa sổ, vùng an toàn và breakpoint

Hai API trả lời hai câu hỏi khác nhau. `MediaQuery.sizeOf(context)` cho kích thước **cả cửa sổ**, dùng để chọn thanh điều hướng. `LayoutBuilder` cho **ràng buộc mà widget cha cấp**, dùng khi một widget cần biết mình có bao nhiêu chỗ (như bài 08). Ở cửa sổ 1000, cột danh sách của bố cục hai cột chỉ rộng 360, và test đo đúng điều này.

Ta chia bề rộng thành ba lớp theo ngưỡng 600 và 840 của window size classes Android (tài liệu Flutter cũng dùng 600 để chọn giữa thanh đáy và thanh bên). Quyết định theo bề rộng cửa sổ, không theo "loại thiết bị", nên điện thoại xoay ngang hay cửa sổ chia đôi đều rơi vào lớp phù hợp.

### window_class.dart · 16-a11y-responsive-adaptive/lib/window_class.dart (trích)

```dart
enum WindowClass {
  compact,
  medium,
  expanded;

  static const double mediumMinWidth = 600;
  static const double expandedMinWidth = 840;

  static WindowClass fromWidth(double width) {
    if (width >= expandedMinWidth) return expanded;
    if (width >= mediumMinWidth) return medium;
    return compact;
  }
}
```

Khung app đọc lớp hiện tại và chọn bố cục như Hình 2; danh sách ba đích dùng chung cho `NavigationBar` và `NavigationRail`.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 2. Ba bố cục theo bề rộng cửa sổ. Compact dưới 600: thanh tiêu đề, một cột nội dung và NavigationBar ở đáy. Medium từ 600 đến 839: NavigationRail bên trái và một cột. Expanded từ 840: NavigationRail, cột danh sách và cột chi tiết."><defs><marker id="fl16-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="40" y="30" class="t-b">compact: dưới 600</text>
<rect x="40" y="44" width="150" height="236" rx="10" class="f-surface s-line"/>
<rect x="40" y="44" width="150" height="30" rx="10" class="f-phys-soft s-phys"/><text x="52" y="64" class="t-small">AppBar</text>
<text x="52" y="140" class="t-small">một cột</text><text x="52" y="162" class="t-small">chạm thẻ:</text><text x="52" y="182" class="t-small">đẩy trang</text>
<rect x="40" y="238" width="150" height="42" rx="10" class="f-pk-soft s-pk"/><text x="52" y="263" class="t-small">NavigationBar</text>
<text x="250" y="30" class="t-b">medium: 600 đến 839</text>
<rect x="250" y="44" width="260" height="190" rx="10" class="f-surface s-line"/>
<rect x="250" y="44" width="260" height="30" rx="10" class="f-phys-soft s-phys"/><text x="262" y="64" class="t-small">AppBar</text>
<rect x="250" y="74" width="62" height="160" class="f-pk-soft s-pk"/><text x="256" y="152" class="t-small">Rail</text>
<text x="326" y="130" class="t-small">một cột</text><text x="326" y="152" class="t-small">chạm thẻ:</text><text x="326" y="172" class="t-small">đẩy trang</text>
<text x="570" y="30" class="t-b">expanded: từ 840</text>
<rect x="570" y="44" width="290" height="190" rx="10" class="f-surface s-line"/>
<rect x="570" y="44" width="290" height="30" rx="10" class="f-phys-soft s-phys"/><text x="582" y="64" class="t-small">AppBar</text>
<rect x="570" y="74" width="62" height="160" class="f-pk-soft s-pk"/><text x="576" y="152" class="t-small">Rail</text>
<rect x="632" y="74" width="100" height="160" class="f-phys-soft s-phys"/><text x="640" y="146" class="t-small">Danh sách</text>
<rect x="732" y="74" width="128" height="160" class="f-pk-soft s-pk"/><text x="740" y="146" class="t-small">Chi tiết</text>
<path d="M40 322 L860 322" class="f-none s-pk" stroke-width="2" marker-end="url(#fl16-2-a)"/><text x="40" y="312" class="t-small">bề rộng tăng</text><text x="236" y="338" class="t-mono">600</text><text x="556" y="338" class="t-mono">840</text>
</svg>
</div>
<figcaption>Hình 2. Ba lớp cửa sổ và bố cục tương ứng. Ranh giới 600 và 840 là quy ước thiết kế, đổi được ở một chỗ (WindowClass).</figcaption>
</figure>

### adaptive_shell.dart · 16-a11y-responsive-adaptive/lib/adaptive_shell.dart (trích)

```dart
    final windowClass = WindowClass.fromWidth(MediaQuery.sizeOf(context).width);
    final selectedIndex = _destination.index;

    final content = IndexedStack(
      key: _contentKey,
      index: selectedIndex,
      children: [
        QuoteScreen(onBook: _book),
        _history(windowClass),
        const SettingsScreen(),
      ],
    );

    if (windowClass == WindowClass.compact) {
      return Scaffold(
        appBar: AppBar(title: Text(_destination.label)),
        body: SafeArea(child: content),
        bottomNavigationBar: NavigationBar(
          // ... destinations
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(title: Text(_destination.label)),
      body: Row(
        children: [
          NavigationRail(
            // ... destinations
          ),
          const VerticalDivider(width: 1),
          Expanded(child: SafeArea(left: false, child: content)),
        ],
      ),
    );
```

`_contentKey` là một `GlobalKey` trên nội dung. Khi máy xoay, nội dung chuyển nhánh trong cây; có `GlobalKey` thì `State` đi theo, nên quãng đường đang kéo không về lại 5 km. Test kéo thanh trượt ở 390, đổi cửa sổ thành 1000 x 500 rồi đọc lại giá trị; khi tạm bỏ key, test thất bại vì giá trị về 5.

Tiếp theo là **vùng không an toàn**. `MediaQuery.padding` là phần màn hình bị hệ thống che; `SafeArea` đệm con để tránh nó. Tài liệu Flutter nói `SafeArea` lồng nhau thì chỉ cái ngoài cùng áp đệm. Hình 3 cho thấy ai chừa cạnh nào.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 360" role="img" aria-label="Hình 3. Điện thoại dọc có vùng không an toàn 47 ở trên và 34 ở dưới. AppBar tự chừa cạnh trên, Scaffold bỏ cạnh trên khỏi phần thân khi có AppBar, NavigationBar tự chừa cạnh dưới, và nút ghim đáy khi không có thanh đáy phải tự bọc SafeArea."><defs><marker id="fl16-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="60" y="20" width="220" height="320" rx="14" class="f-surface s-line"/>
<rect x="60" y="20" width="220" height="40" rx="14" class="f-warn-soft s-warn"/><text x="76" y="46" class="t-small">47 · tai thỏ</text>
<rect x="60" y="60" width="220" height="44" class="f-pk-soft s-pk"/><text x="76" y="88" class="t-small">AppBar (cao 56 + 47)</text>
<rect x="60" y="104" width="220" height="146" class="f-phys-soft s-phys"/><text x="76" y="158" class="t-small">thân Scaffold</text><text x="76" y="182" class="t-small">bắt đầu ở y = 103</text>
<rect x="60" y="250" width="220" height="48" class="f-pk-soft s-pk"/><text x="76" y="280" class="t-small">NavigationBar</text>
<rect x="60" y="298" width="220" height="42" rx="14" class="f-warn-soft s-warn"/><text x="76" y="324" class="t-small">34 · home indicator</text>
<rect x="380" y="30" width="480" height="56" rx="8" class="f-pk-soft s-pk"/><text x="396" y="64" class="t-small">AppBar tự chừa cạnh trên</text>
<rect x="380" y="108" width="480" height="56" rx="8" class="f-phys-soft s-phys"/><text x="396" y="142" class="t-small">Scaffold bỏ cạnh trên khỏi thân khi có AppBar</text>
<rect x="380" y="186" width="480" height="56" rx="8" class="f-pk-soft s-pk"/><text x="396" y="220" class="t-small">NavigationBar có SafeArea riêng ở đáy</text>
<rect x="380" y="264" width="480" height="56" rx="8" class="f-warn-soft s-warn"/><text x="396" y="298" class="t-small">không có thanh đáy: nút ghim tự bọc SafeArea</text>
<path d="M378 58 L282 82" class="f-none s-pk" stroke-width="2" marker-end="url(#fl16-3-a)"/>
<path d="M378 136 L282 150" class="f-none s-pk" stroke-width="2" marker-end="url(#fl16-3-a)"/>
<path d="M378 214 L282 262" class="f-none s-pk" stroke-width="2" marker-end="url(#fl16-3-a)"/>
<path d="M378 292 L282 318" class="f-none s-pk" stroke-width="2" marker-end="url(#fl16-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Ai chừa cạnh nào ở điện thoại dọc có tai thỏ. Con số 47 và 34 là số đo minh hoạ dùng trong test.</figcaption>
</figure>

Nút "Đặt chuyến" ghim ở đáy màn báo giá nằm trong `SafeArea(top: false)`. Ở bố cục thanh bên với đáy 34, cạnh dưới nút ở đúng y = 900 - 34 - 16, dù `SafeArea` lồng ở cả khung app lẫn màn hình. Khi xoay ngang, `NavigationRail` tự chừa cạnh trái, vùng nội dung chừa cạnh phải. Test giả lập bằng `tester.view.padding`:

### support.dart · 16-a11y-responsive-adaptive/test/support.dart (trích)

```dart
void setInsets(
  WidgetTester tester, {
  double top = 0,
  double bottom = 0,
  double left = 0,
  double right = 0,
}) {
  final padding = FakeViewPadding(
    top: top,
    bottom: bottom,
    left: left,
    right: right,
  );
  tester.view.padding = padding;
  tester.view.viewPadding = padding;
  addTearDown(tester.view.resetPadding);
  addTearDown(tester.view.resetViewPadding);
}
```

Ta đặt cả `padding` lẫn `viewPadding`: `SafeArea` đọc `padding`, nhưng phần đáy của `padding` có thể bị bàn phím mở "ăn" mất, còn `maintainBottomViewPadding` dùng `viewPadding` để giữ lại.

Cặp `UnsafeFooter` và `SafeFooter` là test case đối chứng: cạnh dưới nút ở đúng y = 784 khi thiếu `SafeArea` và y = 750 khi có, cả hai được `expect` trong test.

## Adaptive: khác biệt iOS và Android có chủ đích

Responsive là thích nghi theo **kích thước**; adaptive là thích nghi theo **nền tảng**. Nguyên tắc của lab: một cây widget cho cả hai hệ điều hành, chỉ khác ở chỗ hai hệ điều hành có quy ước riêng, mỗi chỗ có test.

Công cụ chính là `Theme.of(context).platform`, không phải `Platform` của `dart:io`: `Platform` là hệ điều hành của máy đang chạy (kể cả máy chạy test), không đổi được theo nhánh cây hay bằng override. `Theme.platform` mặc định là `defaultTargetPlatform`, đổi được cho một nhánh bằng `Theme(data: ...)` và trong test bằng `debugDefaultTargetPlatformOverride`. Các API `.adaptive` của Material đọc đúng giá trị này.

### platform_adaptive.dart · 16-a11y-responsive-adaptive/lib/platform_adaptive.dart (trích)

```dart
bool isApplePlatform(BuildContext context) =>
    switch (Theme.of(context).platform) {
      TargetPlatform.iOS || TargetPlatform.macOS => true,
      TargetPlatform.android ||
      TargetPlatform.fuchsia ||
      TargetPlatform.linux ||
      TargetPlatform.windows => false,
    };

Future<bool> confirmCancelTrip(BuildContext context, Trip trip) async {
  final result = await showAdaptiveDialog<bool>(
    context: context,
    builder: (dialogContext) => AlertDialog.adaptive(
      title: Text('Huỷ chuyến ${trip.id}?'),
      content: const Text('Chuyến sẽ chuyển sang trạng thái Đã huỷ.'),
      actions: [
        _DialogAction(label: 'Giữ chuyến', result: false),
        _DialogAction(label: 'Huỷ chuyến', result: true, destructive: true),
      ],
    ),
  );
  return result ?? false;
}
```

`switch` vét cạn trên `TargetPlatform`, nên nếu Flutter thêm nền tảng mới thì analyzer báo ngay. `showAdaptiveDialog` chọn kiểu route, còn `AlertDialog.adaptive` dựng `CupertinoAlertDialog` trên iOS và macOS và `AlertDialog` của Material ở nơi khác; nút bên trong ta chọn tay theo cùng điều kiện. Ta không dựng cả app bằng Cupertino: Material là nền, Cupertino chỉ xuất hiện ở chỗ người dùng iOS quen một cách khác. Các chỗ khác biệt và test chứng minh:

| Chỗ khác biệt | iOS | Android | Test |
|---|---|---|---|
| Hộp thoại huỷ chuyến | `CupertinoAlertDialog`, nút huỷ `isDestructiveAction` | `AlertDialog` Material, nút huỷ màu `error` | `platform_test.dart` |
| Quay lại | vuốt từ mép trái đóng trang chi tiết (trang dùng `CupertinoPageTransition`) | Flutter không tự bắt cử chỉ này; Back của hệ thống đóng trang | `platform_test.dart` |
| Cuộn quá đầu danh sách | kéo được, `pixels` âm (nảy lại) | `pixels` giữ ở 0 | `platform_test.dart` |
| Công tắc cài đặt | `SwitchListTile.adaptive` vẽ kiểu iOS | vẽ kiểu Material | chỉ kiểm hành vi: chạm đổi giá trị |

Công tắc chỉ được kiểm hành vi vì `Switch.adaptive` không dựng `CupertinoSwitch`: `Switch` luôn dựng công tắc Material và chỉ đổi cấu hình vẽ theo nền tảng. Nút Back trên `AppBar` đóng trang ở cả hai nền tảng. Ở Android, test chỉ chứng minh Flutter không tự bắt cử chỉ: với điều hướng cử chỉ (Android 10 trở lên) hệ điều hành xử lý nó thành Back, nên một test khác gửi thẳng `popRoute`.

### platform_test.dart · 16-a11y-responsive-adaptive/test/platform_test.dart (trích)

```dart
    testWidgets('iOS: vuốt từ mép trái đóng trang chi tiết', (tester) async {
      await pumpApp(tester);
      await openDetail(tester);
      expect(find.byType(CupertinoPageTransition), findsWidgets);

      await swipeFromLeftEdge(tester);

      expect(find.byType(TripDetailPage), findsNothing);
      expect(find.byType(HistoryScreen), findsOneWidget);
    }, variant: _ios);

    testWidgets('Android: Flutter không tự bắt cử chỉ vuốt từ mép trái', (
      tester,
    ) async {
      await pumpApp(tester);
      await openDetail(tester);
      expect(find.byType(CupertinoPageTransition), findsNothing);

      // Flutter không tự xử lý cử chỉ này trên Android. Với điều hướng cử chỉ
      // (Android 10 trở lên), hệ điều hành bắt nó và gửi Back tới app (xem test
      // `systemBack` bên dưới), nên ở máy thật trang vẫn có thể đóng.
      await swipeFromLeftEdge(tester);

      expect(find.byType(TripDetailPage), findsOneWidget);
    }, variant: _android);
```

`TargetPlatformVariant.only(...)` đặt `debugDefaultTargetPlatformOverride` trước thân test và trả lại sau đó. Khi tự đặt, hàm `withPlatform` của lab bọc thân test trong `try/finally` để trả biến về `null` ngay trong thân test.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi làm trợ năng, responsive và adaptive</span>
<ul>
<li><strong>Nhãn bị đọc hai lần.</strong> Dấu hiệu: một node có nhãn lặp như <code>Cước dự kiến</code> rồi <code>Cước dự kiến: 25.500 đồng</code>. Nguyên nhân: chữ bên cạnh vẫn có semantics và widget bọc lại thêm tiền tố. Cách sửa: bỏ tiền tố hoặc dùng <code>excludeSemantics</code>; soi bằng <code>debugDumpSemanticsTree()</code>.</li>
<li><strong>Tự mở <code>ensureSemantics()</code> rồi tắt trong <code>addTearDown</code>.</strong> (Thường không cần mở: <code>testWidgets</code> bật sẵn.) Dấu hiệu: <code>A SemanticsHandle was active at the end of the test</code>. Nguyên nhân: kiểm tra cuối test chạy trước tear-down. Cách sửa: <code>dispose()</code> trong <code>finally</code> ở thân test.</li>
<li><strong>Trả <code>debugDefaultTargetPlatformOverride</code> bằng <code>addTearDown</code>.</strong> Dấu hiệu: <code>The value of a foundation debug variable was changed by the test</code>. Nguyên nhân: cùng lý do trên. Cách sửa: <code>try/finally</code> trong thân test hoặc dùng <code>TargetPlatformVariant</code>.</li>
<li><strong>Tin guideline khi nút sát mép.</strong> Dấu hiệu: test xanh dù nút nhỏ. Nguyên nhân: guideline bỏ qua node sát mép cửa sổ hoặc vùng cuộn. Cách sửa: đặt widget giữa màn hình, như <code>pumpCentered</code>, hoặc đo kích thước trực tiếp.</li>
<li><strong>Mất state khi bố cục đổi.</strong> Dấu hiệu: xoay máy thì thanh trượt về mặc định. Nguyên nhân: widget nằm ở hai vị trí khác nhau của cây ở hai nhánh bố cục nên <code>State</code> bị tạo lại. Cách sửa: <code>GlobalKey</code> cho nội dung dùng chung, hoặc đưa state lên cao hơn.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/16-a11y-responsive-adaptive
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` báo 0 issue, 85 test xanh. Mức kiểm chứng: **kiểm chứng tự động**, không cần thiết bị. Mỗi file test tập trung vào một mảng:

| File test | Chứng minh điều gì |
|---|---|
| `semantics_test.dart` (12) | `25.500 đồng`; câu đọc 8 trạng thái; live region; thẻ chuyến là một nút; slider có nhãn, giá trị, tăng giảm |
| `guidelines_test.dart` (21) | Ba guideline đạt trên app thật; đo riêng thanh điều hướng; contrast sáng, tối; các ca sai bị bắt |
| `text_scale_test.dart` (13) | `TripTile` không overflow ở 320 px, cỡ x1 đến x3; ca cao 72 tràn ở x2; app ở 320 x 568 chữ x2 |
| `safe_area_test.dart` (9) | `view.padding` tới `MediaQuery`; 784 và 750; `AppBar`, `NavigationBar`, xoay ngang |
| `breakpoint_test.dart` (12) | Ngưỡng 600, 840; ba bố cục; hai cột; state giữ khi đổi cỡ cửa sổ |
| `platform_test.dart` (18) | Override đổi `Theme.platform`; hộp thoại, quay lại, vật lý cuộn khác nhau giữa hai nền tảng |

**Chưa làm**: chạy trên simulator hay máy thật; nghe bằng VoiceOver, TalkBack; focus order, bàn phím rời; giảm chuyển động, độ tương phản cao; `intl`; chuyển cột chi tiết thành trang riêng khi cửa sổ thu nhỏ; quản lý state (Phần 3).

## Bài tập

1. `Square44Button` vuông 44, có nhãn "Đóng". Trong ba guideline `android`, `iOS` và `labeled`, cái nào đạt, cái nào không đạt, và sửa thế nào?
2. Thẻ chuyến bọc trong `SizedBox(height: 72)` chứa ba dòng chữ. Chuyện gì xảy ra ở cỡ chữ gấp đôi, và sửa ra sao?
3. Test đặt `debugDefaultTargetPlatformOverride = TargetPlatform.iOS` rồi `addTearDown(() => debugDefaultTargetPlatformOverride = null)`. Mọi `expect` đều đúng nhưng test vẫn báo lỗi. Lỗi gì, vì sao, sửa thế nào?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> <code>iOSTapTargetGuideline</code> và <code>labeledTapTargetGuideline</code> đạt; <code>androidTapTargetGuideline</code> không đạt vì cần 48. Sửa: nâng <code>dimension</code> lên 48, hoặc dùng <code>IconButton</code> có <code>tooltip</code>.</p>
<p><strong>2.</strong> Ba dòng ở cỡ chữ gấp đôi cao hơn 72, <code>Column</code> báo <code>A RenderFlex overflowed by ... pixels on the bottom</code>. Sửa: bỏ chiều cao cố định (hoặc <code>minHeight</code>) và đặt thẻ trong vùng cuộn.</p>
<p><strong>3.</strong> Lỗi <code>The value of a foundation debug variable was changed by the test</code>: kiểm tra cuối test chạy trước các hàm <code>addTearDown</code>. Sửa: đặt <code>null</code> trong <code>finally</code> ở thân test, hoặc dùng <code>TargetPlatformVariant.only(TargetPlatform.iOS)</code>.</p>
</details>

Đối chiếu mục tiêu Phần 2 (các màn hình chính của app khách, theme Tới Nơi, song ngữ, có widget test): app có báo giá (bài 07, 10), danh sách (bài 08, 09), điều hướng và hộp thoại (bài 12), theme (bài 13) và test; chi tiết chuyến thay cho màn xác nhận. Thiếu màn nhập điểm đón và điểm đến (bài 11), song ngữ vi/en (bài 14, ta chỉ đặt locale vi) và animation (bài 15). Giới hạn hiện rõ ở `_trips` và `_replace` trong `_AdaptiveShellState`: danh sách chuyến phải đi qua các tầng bằng callback (`onBook`, `onSelect`, `onCancelled`). Đó là bài toán quản lý state.

Bài trước: [Animation cơ bản: implicit animation và Hero](/articles/flutter-animation-implicit-hero-tim-tai-xe). Bài tiếp: [Quản lý state: InheritedWidget, ChangeNotifier và ranh giới rebuild](/articles/flutter-state-inheritedwidget-changenotifier-bao-gia).

## Tài liệu gốc

- [Accessibility](https://docs.flutter.dev/ui/accessibility) và [General approach to adaptive apps](https://docs.flutter.dev/ui/adaptive-responsive/general) (docs.flutter.dev).
- [SafeArea & MediaQuery](https://docs.flutter.dev/ui/adaptive-responsive/safearea-mediaquery) (docs.flutter.dev).
- [Semantics](https://api.flutter.dev/flutter/widgets/Semantics-class.html), [MergeSemantics](https://api.flutter.dev/flutter/widgets/MergeSemantics-class.html) và [ExcludeSemantics](https://api.flutter.dev/flutter/widgets/ExcludeSemantics-class.html) (api.flutter.dev).
- [AccessibilityGuideline](https://api.flutter.dev/flutter/flutter_test/AccessibilityGuideline-class.html) và [meetsGuideline](https://api.flutter.dev/flutter/flutter_test/meetsGuideline.html) (api.flutter.dev).
- [MediaQuery](https://api.flutter.dev/flutter/widgets/MediaQuery-class.html), [TextScaler](https://api.flutter.dev/flutter/painting/TextScaler-class.html), [SafeArea](https://api.flutter.dev/flutter/widgets/SafeArea-class.html) và [LayoutBuilder](https://api.flutter.dev/flutter/widgets/LayoutBuilder-class.html) (api.flutter.dev).
- [AlertDialog.adaptive](https://api.flutter.dev/flutter/material/AlertDialog/AlertDialog.adaptive.html), [Switch.adaptive](https://api.flutter.dev/flutter/material/Switch/Switch.adaptive.html) và [TargetPlatformVariant](https://api.flutter.dev/flutter/flutter_test/TargetPlatformVariant-class.html) (api.flutter.dev).
- [Use window size classes](https://developer.android.com/develop/ui/compose/layouts/adaptive/use-window-size-classes) và [Touch target size](https://support.google.com/accessibility/android/answer/7101858) (developer.android.com, support.google.com).
- [Human Interface Guidelines: Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility) (developer.apple.com).
