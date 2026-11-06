---
slug: flutter-golden-test-semantics-test-giao-dien-on-dinh
title: "Golden test và Semantics test: khoá giao diện và accessibility"
summary: "Bài 29: khoá thẻ chuyến và chip trạng thái Tới Nơi bằng golden test sáng và tối, kích thước cố định, comparator có ngưỡng, tag golden cho CI, quy trình cập nhật ảnh chuẩn; khoá cây semantics bằng meetsGuideline. Nêu rõ phần chưa chạy trên Linux."
category: programming
level: intermediate
tags: [flutter, dart, mobile, toi-noi, testing, golden-test, accessibility]
language: vi
---

Hai thay đổi nhỏ lọt qua bộ test của app khách Tới Nơi trong cùng một tuần. Một lần dọn theme làm chip "Tài xế đã nhận" đổi màu mà không ai để ý. Một lần refactor bọc cả thẻ chuyến vào `ExcludeSemantics`, nên nút "Ẩn chuyến" biến mất với người dùng trình đọc màn hình. Widget test vẫn xanh vì nó tìm chữ và bấm nút, không nhìn pixel, không nghe cây semantics. Tên và tình huống là hư cấu.

Bài này thêm hai lớp khoá. **Golden test** chụp widget thành ảnh rồi so với ảnh chuẩn đã commit. **Semantics test** kiểm cây semantics mà VoiceOver và TalkBack đọc, cùng guideline về vùng chạm, nhãn và độ tương phản. Golden có cái giá riêng: ảnh phụ thuộc máy sinh ra nó, và bài nói thẳng điều lab chưa kiểm được.

Đây là bài 29 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 28 về widget test](/articles/flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia). Bạn cần biết `testWidgets`, `pumpWidget` và finder ở bài đó, theme `toi_noi_ui` ở [bài 13](/articles/flutter-theme-material-3-dark-mode-design-system-toi-noi) và `Semantics` ở [bài 16](/articles/flutter-accessibility-responsive-adaptive-ios-android). Bối cảnh nằm ở hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); bảng giá và mọi con số là minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một golden test chụp thẻ chuyến có địa chỉ đón "Chung cư Hoa Sen, 12 Đường Mẫu". Ai đó sửa thành "Chung cư Hoa Sun, 99 Đường Mẫu", cùng độ dài. Golden có trượt không? Còn nếu bỏ nút ẩn chuyến (icon X) thì sao?</p>
<details><summary>Xem đáp án</summary><p>Ca đầu không trượt vì cả hai địa chỉ chỉ gồm chữ ASCII: font kiểm thử mặc định của <code>flutter test</code> (tên <code>FlutterTest</code>) vẽ chúng thành ô đặc như nhau, nên ảnh giống hệt từng byte. Test <code>golden_setup_test.dart</code> kiểm điều đó và kiểm rằng cây semantics thấy thay đổi. Chỉ thay ký tự cùng nhóm hình mới không đổi ảnh: thêm dấu tiếng Việt ("Sen" thành "Sến") làm lệch 178 trên 99.840 pixel, vượt ngưỡng 0,1%. Bỏ nút X làm lệch khoảng 0,45% ảnh một thẻ, ngưỡng 0,1% cũng bắt được. Golden khoá hình dạng và màu, không khoá nội dung chữ.</p></details>
</div>

## Golden test: so ảnh với ảnh chuẩn đã commit

`matchesGoldenFile` lấy `RepaintBoundary` gần nhất bao quanh widget tìm được, chụp thành ảnh (`captureImage`), mã hoá PNG rồi đưa cho `goldenFileComparator`. Dưới `flutter test`, comparator mặc định là `LocalFileComparator`: đọc ảnh chuẩn theo đường dẫn tương đối với thư mục file test và chỉ qua khi từng pixel khớp. Hình 1 vẽ luồng đó.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Luồng của một golden test. Cây widget được dựng ở kích thước cố định, chụp thành ảnh PNG, đưa vào comparator. Comparator đọc ảnh chuẩn trong thư mục test/goldens và so hai ảnh. Nếu pixel lệch không vượt ngưỡng thì qua; nếu vượt thì ném lỗi và ghi ảnh diff vào thư mục failures."><defs><marker id="fl29-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl29-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="110" width="150" height="70" rx="8" class="f-phys-soft s-phys"/><text x="34" y="138" class="t-b">Cây widget</text><text x="34" y="160" class="t-small">390 px, sáng/tối</text>
<rect x="230" y="110" width="150" height="70" rx="8" class="f-phys-soft s-phys"/><text x="244" y="138" class="t-b">Ảnh PNG</text><text x="244" y="160" class="t-mono">captureImage</text>
<rect x="440" y="110" width="150" height="70" rx="8" class="f-pk-soft s-pk"/><text x="454" y="138" class="t-b">Comparator</text><text x="454" y="160" class="t-small">so với ảnh chuẩn</text>
<rect x="440" y="220" width="150" height="64" rx="8" class="f-surface s-line"/><text x="454" y="246" class="t-b">Ảnh chuẩn</text><text x="454" y="268" class="t-mono">goldens/*.png</text>
<rect x="650" y="40" width="210" height="70" rx="8" class="f-phys-soft s-phys"/><text x="664" y="68" class="t-b">Qua</text><text x="664" y="90" class="t-small">pixel lệch trong ngưỡng</text>
<rect x="650" y="140" width="210" height="140" rx="8" class="f-warn-soft s-warn"/><text x="664" y="168" class="t-b">Trượt</text><text x="664" y="192" class="t-small">ném FlutterError</text><text x="664" y="214" class="t-small">ghi ảnh diff vào failures/</text><text x="664" y="236" class="t-small">thư mục này không commit</text>
<path d="M172 145 L228 145" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-1-a)"/>
<path d="M382 145 L438 145" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-1-a)"/>
<path d="M515 218 L515 182" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-1-a)"/>
<path d="M592 128 L648 86" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-1-a)"/>
<path d="M592 164 L648 196" class="f-none s-warn" stroke-width="2" marker-end="url(#fl29-1-w)"/>
</svg>
</div>
<figcaption>Hình 1. Một golden test từ widget tới kết quả. Ảnh chuẩn là đầu vào của comparator, nằm trong git; thư mục failures/ chỉ là đầu ra khi trượt.</figcaption>
</figure>

Ta gom việc dựng khung vào một helper để mỗi golden chỉ còn vài dòng. Khung rộng cố định 390 pixel, và `RepaintBoundary` mang khoá `goldenKey` là vùng được chụp:

### golden_harness.dart · 29-golden-semantics/test/support/golden_harness.dart (trích)

```dart
Future<void> pumpGolden(
  WidgetTester tester,
  Widget child, {
  Brightness brightness = Brightness.light,
  double textScale = 1,
  double viewHeight = 844,
}) async {
  setGoldenView(tester, height: viewHeight);
  tester.platformDispatcher.textScaleFactorTestValue = textScale;
  addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);

  await tester.pumpWidget(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      themeMode: brightness == Brightness.dark
          ? ThemeMode.dark
          : ThemeMode.light,
      home: Scaffold(
        body: Align(
          alignment: Alignment.topLeft,
          child: RepaintBoundary(
            key: goldenKey,
            child: SizedBox(
              width: goldenWidth,
              child: Builder(
                builder: (context) => ColoredBox(
                  color: Theme.of(context).colorScheme.surface,
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: child,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
}
```

Mỗi golden sau đó là một vòng lặp qua hai giao diện. Khoá ảnh tính từ thư mục chứa file test, nên `goldens/trip_card_light.png` nằm ở `test/goldens/`:

### golden_trip_card_test.dart · 29-golden-semantics/test/golden_trip_card_test.dart (trích)

```dart
    for (final brightness in Brightness.values) {
      testWidgets('bốn trạng thái, giao diện ${brightness.name}', (
        tester,
      ) async {
        await pumpGolden(
          tester,
          sampleCardColumn(),
          brightness: brightness,
          viewHeight: 1200,
        );

        await expectLater(
          find.byKey(goldenKey),
          matchesGoldenFile('goldens/trip_card_${brightness.name}.png'),
        );
      });
    }
```

Lần đầu chưa có ảnh, ta chạy `flutter test --update-goldens`: cờ này đổi `matchesGoldenFile` từ "so" thành "ghi". Lab có năm ảnh: thẻ chuyến sáng, tối và chữ gấp đôi, chip tám trạng thái sáng và tối.

## Cố định những gì ta kiểm soát được

Ảnh chỉ ổn định khi mọi đầu vào ổn định. Ta khoá bốn thứ.

**Kích thước cửa sổ.** Cửa sổ test mặc định là 800 x 600 pixel logic. Ta đặt 390 x 844 và `devicePixelRatio` bằng 1 qua `tester.view`, rồi trả lại bằng `addTearDown(tester.view.reset)`. Một test của lab kiểm ba điều: giá trị mặc định, giá trị sau khi đặt, và ảnh chụp rộng đúng 390.

### golden_harness.dart · 29-golden-semantics/test/support/golden_harness.dart (trích)

```dart
void setGoldenView(
  WidgetTester tester, {
  double width = goldenWidth,
  double height = 844,
}) {
  tester.view.devicePixelRatio = 1;
  tester.view.physicalSize = Size(width, height);
  addTearDown(tester.view.reset);
}
```

**Theme và cỡ chữ.** Theme chọn bằng `themeMode`, không theo cài đặt hệ thống của máy chạy test. Cỡ chữ đặt bằng `textScaleFactorTestValue`, cùng đường mà bài 16 dùng.

**Bóng đổ.** Mã nguồn `debug.dart` nói bóng mờ không đảm bảo vẽ giống nhau giữa các phiên bản, thậm chí giữa các lần chạy, nên binding của `testWidgets` bật sẵn `debugDisableShadows` (một test của lab kiểm cờ này là `true`). Theo `proxy_box.dart`, khi cờ bật, nền có `elevation` lớn hơn 0 được vẽ viền đặc thay cho bóng mờ; card của `toi_noi_ui` có elevation 0 nên ít bị ảnh hưởng.

**Font.** Tài liệu `matchesGoldenFile` vẫn ghi framework mặc định dùng font Ahem, nhưng đo thực tế thì font mặc định là `FlutterTest`, khác Ahem (baseline khác nhau). Hệ quả, đều có test: chữ ASCII thành ô đặc, còn một số ký tự tiếng Việt như ư, đ, ơ thành ô rỗng; mỗi ký tự trong "Tới Nơi" rộng đúng một cỡ chữ (cỡ 16 rộng 112 pixel) nên bố cục khác máy thật. Chỉ thay chữ bằng chữ cùng nhóm hình (ASCII sang ASCII cùng độ dài) mới không đổi ảnh. Ta chấp nhận giới hạn vì lab không commit font; muốn chữ thật, nạp font có giấy phép phù hợp bằng `FontLoader` và chấp nhận ảnh đổi theo từng bản font.

Icon thì khác. Khi dựng lab, ta thấy mọi icon là ô vuông rỗng vì `flutter test` không nạp `MaterialIcons`. Font này nằm sẵn trong asset do `uses-material-design: true` sinh ra; ta chỉ đọc và đăng ký, không commit file nào:

### golden_harness.dart · 29-golden-semantics/test/support/golden_harness.dart (trích)

```dart
Future<void> loadMaterialIcons() async {
  final loader = FontLoader('MaterialIcons')
    ..addFont(rootBundle.load('fonts/MaterialIcons-Regular.otf'));
  await loader.load();
}
```

`flutter_test_config.dart` gọi hàm này một lần (mục sau). Test đối chứng vẽ `Icons.check_circle` và `Icons.cancel` rồi khẳng định hai ảnh khác nhau.

## Golden lệch giữa hệ điều hành: ngưỡng, tag và CI

Ảnh chuẩn của lab sinh trên macOS Apple silicon, còn CI của ta là ubuntu. Tài liệu `matchesGoldenFile` cảnh báo font tuỳ chỉnh có thể vẽ khác giữa các hệ điều hành và phiên bản Flutter; lab tự nạp font icon `MaterialIcons` nên cũng nằm trong nhóm rủi ro đó. **Lab chưa chạy trên Linux**, nên ta không biết ảnh lệch bao nhiêu và bài không đoán con số. Ba biện pháp dưới đây là cách làm an toàn khi chưa biết.

**Một, comparator có hai ngưỡng.** `LocalFileComparator` coi pixel là lệch khi bất kỳ kênh nào chênh dù chỉ một mức trên 255, nên nhiễu nhẹ làm trượt cả ảnh. `TolerantGoldenComparator` chỉ tính pixel lệch khi một kênh chênh hơn 4 mức và chỉ trượt khi hơn 0,1% pixel lệch; ảnh khác kích thước luôn trượt:

### tolerant_comparator.dart · 29-golden-semantics/test/support/tolerant_comparator.dart (trích)

```dart
  @override
  Future<bool> compare(Uint8List imageBytes, Uri golden) async {
    final master = await getGoldenBytes(golden);
    final result = await GoldenFileComparator.compareLists(imageBytes, master);
    if (result.passed) {
      result.dispose();
      return true;
    }

    final diff = await measurePixelDiff(
      imageBytes,
      master,
      channelTolerance: channelTolerance,
    );
    if (diff != null && diff.fraction <= pixelTolerance) {
      debugPrint(
        'Golden "$golden": lệch ${diff.differing}/${diff.total} pixel '
        '(<= $pixelTolerance), chấp nhận.',
      );
      result.dispose();
      return true;
    }

    final error = await generateFailureOutput(result, golden, basedir);
    result.dispose();
    // ... ghép thông báo có số pixel và ngưỡng, rồi ném FlutterError
  }
```

Flutter chạy `flutter_test_config.dart` trước mọi file test cùng thư mục hoặc thư mục con (file gần nhất thắng). Ta nạp font icon và thay comparator ở đây:

### flutter_test_config.dart · 29-golden-semantics/test/flutter_test_config.dart (trích)

```dart
Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  TestWidgetsFlutterBinding.ensureInitialized();
  await loadMaterialIcons();

  final current = goldenFileComparator;
  if (current is LocalFileComparator) {
    goldenFileComparator = TolerantGoldenComparator(
      current.basedir.resolve('golden_test.dart'),
    );
  }
  await testMain();
}
```

**Hai, ngưỡng phải nhỏ hơn thay đổi nhỏ nhất ta muốn bắt.** Đo trên macOS, bỏ nút X của một thẻ làm lệch 446 trên 99.840 pixel, khoảng 0,45%. Ngưỡng 0,5% nuốt lỗi này, ngưỡng 0,1% bắt được; một test của lab dựng cả hai comparator và khẳng định đúng vậy. Con số 0,1% là lựa chọn của ta, chưa hiệu chỉnh trên Linux.

**Ba, buộc ảnh chuẩn với hệ điều hành đã sinh ra nó.** Mọi test đụng ảnh chuẩn mang tag `golden` (`@Tags(['golden'])` đầu file) và nằm trong `group(..., skip: goldenSkip)`; test cố ý làm trượt dùng `goldenCompareSkip`, thêm điều kiện không phải `--update-goldens`. Ở máy khác `goldenBaselineOs`, `goldenSkip` trả lý do và các test được bỏ qua thay vì so với ảnh của máy khác:

### golden_harness.dart · 29-golden-semantics/test/support/golden_harness.dart (trích)

```dart
const goldenBaselineOs = 'macos';

Object get goldenSkip =>
    autoUpdateGoldenFiles || Platform.operatingSystem == goldenBaselineOs
    ? false
    : 'Ảnh chuẩn sinh trên $goldenBaselineOs, máy này là '
          '${Platform.operatingSystem}. Chạy golden ở job $goldenBaselineOs.';
```

Tag được khai báo trong `dart_test.yaml`. Bảng dưới là hành vi theo thiết kế; cột Linux chỉ là mô phỏng bằng cách tạm đặt `goldenBaselineOs = 'linux'` trên macOS, không phải Linux thật.

| Lệnh | macOS | Linux (mô phỏng) |
|---|---|---|
| `flutter test` | 51 test, gồm 11 golden | 40 test chạy, 11 golden bỏ qua kèm lý do |
| `flutter test --exclude-tags golden` | 40 test | 40 test |
| `flutter test --tags golden` | 11 golden | 11 golden bỏ qua |

Hình 2 đặt chúng vào CI. Job ubuntu chạy `flutter test --exclude-tags golden`, loại golden một cách tường minh thay vì dựa vào `goldenSkip`; golden chạy ở job macOS, nơi ảnh chuẩn được sinh. Runner macOS của GitHub chưa được kiểm; workflow đầy đủ ở bài 33. Muốn golden chạy ở ubuntu, sinh lại ảnh chuẩn trên Linux và đổi `goldenBaselineOs` trong cùng commit.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Hai job CI. Mỗi lần push, job ubuntu chạy flutter test với exclude-tags golden và không so ảnh chuẩn. Job macOS chạy flutter test với tags golden và so với ảnh chuẩn sinh trên macOS. Mũi tên nét đứt từ ảnh chuẩn sang job ubuntu thể hiện phép so mà ta cố ý không làm."><defs><marker id="fl29-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl29-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="125" width="150" height="80" rx="8" class="f-surface s-line"/><text x="36" y="156" class="t-b">Push / PR</text><text x="36" y="180" class="t-small">đổi code hoặc ảnh</text>
<rect x="260" y="30" width="340" height="110" rx="8" class="f-phys-soft s-phys"/><text x="278" y="60" class="t-b">Job ubuntu</text><text x="278" y="86" class="t-mono">flutter test --exclude-tags golden</text><text x="278" y="112" class="t-small">40 test, không so ảnh chuẩn</text>
<rect x="260" y="190" width="340" height="110" rx="8" class="f-pk-soft s-pk"/><text x="278" y="220" class="t-b">Job macOS</text><text x="278" y="246" class="t-mono">flutter test --tags golden</text><text x="278" y="272" class="t-small">11 golden, so ảnh chuẩn</text>
<rect x="690" y="190" width="170" height="110" rx="8" class="f-surface s-line"/><text x="706" y="220" class="t-b">Ảnh chuẩn</text><text x="706" y="246" class="t-small">sinh trên macOS</text><text x="706" y="270" class="t-mono">goldens/*.png</text>
<path d="M172 150 L258 94" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-2-a)"/>
<path d="M172 182 L258 238" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-2-a)"/>
<path d="M688 245 L602 245" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-2-a)"/>
<path d="M760 188 L760 85 L602 85" class="f-none s-warn" stroke-width="2" stroke-dasharray="8 6" marker-end="url(#fl29-2-w)"/>
<text x="624" y="70" class="t-small">không so ở đây</text>
</svg>
</div>
<figcaption>Hình 2. Golden chạy ở nơi ảnh chuẩn được sinh. Đường nét đứt là phép so mà ta cố ý không làm, vì lab chưa kiểm ảnh macOS trên Linux.</figcaption>
</figure>

## Cập nhật ảnh chuẩn và đọc ảnh diff

Khi giao diện đổi có chủ đích, ảnh chuẩn phải đổi theo. `flutter test --update-goldens` ghi đè ảnh thay vì so, nên `matchesGoldenFile` luôn "qua"; đừng chạy nó trên cả bộ test cho xanh. Thu hẹp bằng tag và tên: `flutter test --update-goldens --tags golden --plain-name TripCard` chỉ chạy ba golden của thẻ chuyến. Rồi xem `git diff --stat test/goldens`, mở từng ảnh đổi và đối chiếu với thay đổi giao diện trong PR.

Khi golden trượt, comparator ghi bốn ảnh vào `failures/` cạnh file test (chỉ hai ảnh đầu nếu hai ảnh khác kích thước): `*_masterImage.png` (ảnh chuẩn), `*_testImage.png` (ảnh vừa chụp), `*_isolatedDiff.png` (chỉ pixel lệch) và `*_maskedDiff.png` (pixel lệch chồng lên ảnh). Thư mục này chỉ để xem, nằm trong `.gitignore`, không commit. Hình 3 là quy trình đọc chúng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 320" role="img" aria-label="Hình 3. Quy trình khi golden trượt. Đọc bốn ảnh trong failures, rồi phân biệt hai trường hợp. Nếu là lỗi thật thì sửa code và chạy lại. Nếu là thay đổi có chủ đích thì chạy update-goldens với tag golden, xem git diff stat và từng ảnh đổi, rồi commit ảnh cùng code."><defs><marker id="fl29-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl29-3-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="40" width="170" height="70" rx="8" class="f-warn-soft s-warn"/><text x="36" y="68" class="t-b">Golden trượt</text><text x="36" y="92" class="t-small">báo % pixel lệch</text>
<rect x="240" y="40" width="200" height="70" rx="8" class="f-surface s-line"/><text x="256" y="68" class="t-b">Mở failures/</text><text x="256" y="92" class="t-small">masked và isolated diff</text>
<rect x="490" y="40" width="170" height="70" rx="8" class="f-pk-soft s-pk"/><text x="506" y="68" class="t-b">Do đâu?</text><text x="506" y="92" class="t-small">lỗi hay chủ đích</text>
<rect x="710" y="20" width="150" height="70" rx="8" class="f-phys-soft s-phys"/><text x="724" y="48" class="t-b">Lỗi thật</text><text x="724" y="72" class="t-small">sửa code, chạy lại</text>
<rect x="710" y="130" width="150" height="70" rx="8" class="f-pk-soft s-pk"/><text x="724" y="158" class="t-b">Chủ đích</text><text x="724" y="182" class="t-mono">--update-goldens</text>
<rect x="400" y="230" width="460" height="64" rx="8" class="f-surface s-line"/><text x="416" y="258" class="t-b">Soát ảnh đổi</text><text x="416" y="280" class="t-small">git diff --stat, xem từng ảnh, commit cùng code</text>
<path d="M192 75 L238 75" class="f-none s-warn" stroke-width="2" marker-end="url(#fl29-3-w)"/>
<path d="M442 75 L488 75" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-3-a)"/>
<path d="M662 60 L708 55" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-3-a)"/>
<path d="M662 92 L708 150" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-3-a)"/>
<path d="M785 202 L785 228" class="f-none s-pk" stroke-width="2" marker-end="url(#fl29-3-a)"/>
</svg>
</div>
<figcaption>Hình 3. Từ lúc golden trượt tới lúc commit ảnh mới. Bước "Soát ảnh đổi" là bước không được bỏ khi dùng --update-goldens.</figcaption>
</figure>

Ta không tin comparator chỉ vì nó chạy. `comparator_test.dart` dựng ảnh tổng hợp 100 x 100 từ hình chữ nhật nguyên pixel, nên giống hệt trên mọi hệ điều hành: nhiễu 3 mức trên mọi pixel qua (dù comparator mặc định thấy 100% pixel khác), lệch 10 mức trượt, đổi 10 pixel qua và 11 pixel trượt, khác kích thước luôn trượt, và lúc trượt có đúng bốn tên file trong `failures/`. Trên thẻ thật, `golden_proof_test.dart` chép ảnh chuẩn sang thư mục tạm, đổi trạng thái thẻ đầu thành "Tài xế đã nhận" rồi khẳng định comparator ném lỗi:

### golden_proof_test.dart · 29-golden-semantics/test/golden_proof_test.dart (trích)

```dart
    Future<Object?> compareToBaseline(WidgetTester tester) async {
      final png = await capturePng(tester);
      return tester.runAsync<Object?>(() async {
        try {
          await goldenFileComparator.compare(png, Uri.parse('baseline.png'));
          return null;
        } on FlutterError catch (error) {
          return error;
        }
      });
    }
```

Cơ chế: `matchesGoldenFile` chạy comparator trong `tester.runAsync`; ở đó `FlutterError` bị bắt và báo bằng `FlutterError.reportError`, còn `expectLater` vẫn hoàn tất bình thường, nên `throwsA` không thấy gì ("emitted null"). Lỗi nằm chờ ở binding và `tester.takeException()` lấy được; một test của lab dùng đúng cách đó. Ở các ca còn lại ta gọi thẳng comparator trên ảnh vừa chụp. Thư mục tạm giữ cho ảnh chuẩn đã commit không bị đụng tới.

## Semantics test: khoá thứ người dùng trình đọc màn hình nghe

Golden không biết gì về trình đọc màn hình. Với `TripCard`, ta khoá cây semantics bằng `isSemantics`. Thân thẻ là một nút duy nhất đọc trọn câu, số cước do `calculateFare` tính (5 km là 25.500 đồng), còn chữ, chip và icon bên trong bị loại để không đọc lần hai:

### semantics_test.dart · 29-golden-semantics/test/semantics_test.dart (trích)

```dart
      const spoken =
          'Đặt xe, mã T-1045, từ Chung cư Hoa Sen, 12 Đường Mẫu đến Ga Trung '
          'Tâm, cổng B, 5 km, trạng thái Đang tìm tài xế, 25.500 đồng';
      expect(calculateFare(5).vnd, 25500);
      expect(
        tester.getSemantics(find.bySemanticsLabel(spoken)),
        isSemantics(label: spoken, isButton: true, hasTapAction: true),
      );
```

`isSemantics` thay cho `containsSemantics`, hàm đã deprecated và bị `flutter analyze` báo. Nút ẩn chuyến là nút riêng có `tooltip` làm nhãn, nên phải nằm ngoài vùng `excludeSemantics` của thân thẻ. Một test dựng lại sự cố mở đầu: bọc cả thẻ trong `ExcludeSemantics` thì `find.bySemanticsLabel('Ẩn chuyến')` không thấy gì, dù `IconButton` vẫn có trên màn hình. Tám trạng thái đều có test kiểm nhãn.

Ba guideline `androidTapTargetGuideline` (48 pixel), `iOSTapTargetGuideline` (44 pixel) và `labeledTapTargetGuideline` đã có ở bài 16; ở đây ta áp chúng lên `TripCard` sáng và tối, đặt giữa màn hình vì guideline bỏ qua node sát mép. Ba ca sai trong `broken_cards.dart` chứng minh chúng bắt được lỗi: icon 20 không nhãn trượt cả ba, `IconButton` quên `tooltip` trượt nhãn, nhãn nhạt trên nền nhạt trượt contrast.

Contrast có một cái bẫy mới. Mã nguồn `textContrastGuideline` duyệt node semantics rồi tìm `find.text(nhãn của node)`. Thẻ chuyến gộp cả thẻ thành một câu, không `Text` nào trùng câu đó, nên guideline không đo gì và báo qua. Ta dùng `CustomMinimumContrastGuideline` với finder là mọi `Text`:

### semantics_test.dart · 29-golden-semantics/test/semantics_test.dart (trích)

```dart
AccessibilityGuideline allTextContrast() => CustomMinimumContrastGuideline(
  finder: find.byType(Text),
  description: 'mọi Text đạt tỉ lệ 4,5:1',
);

    testWidgets(
      'gộp semantics làm textContrastGuideline mù, bản custom vẫn thấy',
      (tester) async {
        await pumpCentered(
          tester,
          Semantics(
            label: 'Trạng thái: Đang tìm tài xế',
            excludeSemantics: true,
            child: const FaintStatusBadge(status: TripStatus.searching),
          ),
        );

        await expectLater(tester, meetsGuideline(textContrastGuideline));
        await expectLater(tester, doesNotMeetGuideline(allTextContrast()));
      },
    );
```

Với bản custom, tám trạng thái của thẻ và của chip đều đạt 4,5:1 ở cả giao diện sáng lẫn tối. Ca sai `FaintStatusBadge` trượt ở giao diện sáng với cả tám trạng thái và qua ở giao diện tối (một test kiểm cả hai), nên ca sai kiểm riêng giao diện sáng.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với golden và semantics test</span>
<ul>
<li><strong>Golden xanh trên máy mình, đỏ trên CI.</strong> Dấu hiệu: thông báo "Pixel test failed" kèm số pixel lệch ở nhiều ảnh cùng lúc. Nguyên nhân có thể là ảnh chuẩn sinh ở hệ điều hành khác với nơi chạy (lab chưa đo mức lệch). Cách sửa: chạy golden ở đúng hệ điều hành đã sinh ảnh, hoặc sinh lại ảnh chuẩn ở nơi CI chạy; đừng nâng ngưỡng cho tới khi xanh.</li>
<li><strong>Golden xanh dù đổi chữ.</strong> Dấu hiệu: sửa lỗi chính tả mà không ảnh nào đổi. Nguyên nhân: font kiểm thử vẽ chữ thành ô, và ký tự cùng nhóm hình (chữ ASCII cùng độ dài) cho cùng ảnh. Cách sửa: khoá nội dung chữ bằng finder và <code>isSemantics</code>, để golden lo hình dạng và màu.</li>
<li><strong>Ngưỡng rộng hơn lỗi cần bắt.</strong> Dấu hiệu: mất một icon nhỏ mà golden vẫn xanh. Nguyên nhân: ngưỡng tính theo phần trăm của cả ảnh, ảnh to thì icon nhỏ lọt qua. Cách sửa: đo phần trăm lệch của thay đổi nhỏ nhất bằng <code>measurePixelDiff</code>, đặt ngưỡng dưới số đó, và chụp ảnh nhỏ hơn.</li>
<li><strong><code>--update-goldens</code> trên cả bộ test rồi commit.</strong> Dấu hiệu: PR đổi hàng chục ảnh không ai xem. Nguyên nhân: cờ này luôn làm test qua. Cách sửa: thu hẹp bằng <code>--tags golden --plain-name</code>, xem <code>git diff --stat</code> và từng ảnh.</li>
<li><strong>Guideline xanh vì không đo gì.</strong> Dấu hiệu: <code>textContrastGuideline</code> qua dù chữ nhạt. Nguyên nhân: nhãn semantics đã gộp nên không trùng chữ của <code>Text</code> nào; guideline vùng chạm cũng bỏ qua node sát mép. Cách sửa: dùng <code>CustomMinimumContrastGuideline</code> với finder, đặt widget giữa màn hình, và luôn có một ca sai chứng minh guideline bắt được lỗi.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/29-golden-semantics
flutter pub get
flutter analyze
flutter test
flutter test --exclude-tags golden   # lệnh cho CI ubuntu
flutter test --tags golden           # chỉ golden, ở máy khớp hệ điều hành baseline
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3 trên macOS Apple silicon: `flutter analyze` báo 0 issue, `flutter test` chạy 51 test xanh, trong đó 11 mang tag `golden`; `--exclude-tags golden` chạy 40 test. Mức: **Kiểm chứng tự động**, không cần thiết bị.

Các nhóm test: 5 golden; 6 test chứng minh comparator bắt thẻ đổi trạng thái và mất nút, `takeException` và ngưỡng 0,5% nuốt lỗi icon; 8 test comparator trên ảnh tổng hợp; 9 test về cửa sổ, bóng đổ, font kiểm thử và icon; 23 test semantics và guideline, gồm các ca sai có chủ đích.

**Chưa làm**: chưa chạy trên Linux và trên runner macOS của CI, nên chưa biết ảnh lệch bao nhiêu giữa hai hệ điều hành và ngưỡng 0,1% chưa được hiệu chỉnh; chữ trong ảnh là font kiểm thử, không phải font thật; chưa nghe bằng VoiceOver hay TalkBack; chưa có workflow GitHub Actions (bài 33).

## Bài tập

1. Giảm padding của `TripCard` từ 16 xuống 12. Golden nào trượt, vì sao, và lệnh nào cập nhật đúng những ảnh cần đổi?
2. Ảnh `trip_card_light.png` rộng 390 và cao 944 pixel. Ngưỡng 0,1% và 0,5% cho phép lệch tối đa bao nhiêu pixel? Giả sử bỏ một nút X làm lệch khoảng 446 pixel, golden trượt hay qua ở mỗi ngưỡng?
3. Chạy `FaintStatusBadge` ở giao diện tối với `allTextContrast()`. Dự đoán kết quả, rồi giải thích vì sao ca sai trong lab chỉ dùng giao diện sáng.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Chiều cao thẻ đổi nên ba ảnh thẻ (<code>trip_card_light</code>, <code>trip_card_dark</code>, <code>trip_card_text_x2_light</code>) trượt vì khác kích thước, và khác kích thước thì ngưỡng nào cũng không cứu. Ảnh chip không đổi vì không dùng padding đó. Hai test trong <code>golden_proof_test.dart</code> so với <code>trip_card_light.png</code> cũng đỏ cho tới khi ảnh được cập nhật. Lệnh: <code>flutter test --update-goldens --tags golden --plain-name TripCard</code>, rồi xem <code>git diff --stat test/goldens</code>.</p>
<p><strong>2.</strong> Tổng là 368.160 pixel, nên 0,1% cho phép 368 pixel và 0,5% cho phép 1.840. Với 446 pixel, golden trượt ở ngưỡng 0,1% và qua ở ngưỡng 0,5%. Đây là phép tính giả định; số thật đo bằng <code>measurePixelDiff</code>, và nó cho thấy vì sao ngưỡng rộng nuốt lỗi nhỏ.</p>
<p><strong>3.</strong> Qua. Màu chữ của <code>FaintStatusBadge</code> được pha 55% trắng; trên nền tối, chữ sáng hơn nền nên tỉ lệ vẫn cao, còn trên nền sáng chữ nhạt chạm nền nhạt và trượt. Ca sai phải chứng minh guideline bắt lỗi, nên ta chọn giao diện làm lỗi xảy ra.</p>
</details>

Bài trước: [Widget test: pumpWidget, finder, matcher và màn hình báo giá](/articles/flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia). Bài tiếp: [Kiểm thử Riverpod, async và Stream: ProviderContainer, fakeAsync, fake clock](/articles/flutter-test-riverpod-async-stream-fake-async-providercontainer), nơi ta kiểm hành vi theo thời gian mà ảnh tĩnh không chụp được.

## Tài liệu gốc

- [matchesGoldenFile](https://api.flutter.dev/flutter/flutter_test/matchesGoldenFile.html), [GoldenFileComparator](https://api.flutter.dev/flutter/flutter_test/GoldenFileComparator-class.html) và [LocalFileComparator](https://api.flutter.dev/flutter/flutter_test/LocalFileComparator-class.html) (api.flutter.dev).
- [flutter_test library](https://api.flutter.dev/flutter/flutter_test/flutter_test-library.html): cấu hình test theo thư mục bằng `flutter_test_config.dart` (api.flutter.dev).
- [AccessibilityGuideline](https://api.flutter.dev/flutter/flutter_test/AccessibilityGuideline-class.html), [meetsGuideline](https://api.flutter.dev/flutter/flutter_test/meetsGuideline.html) và [CustomMinimumContrastGuideline](https://api.flutter.dev/flutter/flutter_test/CustomMinimumContrastGuideline-class.html) (api.flutter.dev).
- [isSemantics](https://api.flutter.dev/flutter/flutter_test/isSemantics.html) và [TestFlutterView](https://api.flutter.dev/flutter/flutter_test/TestFlutterView-class.html) (api.flutter.dev).
- [AutomatedTestWidgetsFlutterBinding](https://api.flutter.dev/flutter/flutter_test/AutomatedTestWidgetsFlutterBinding-class.html) và [FontLoader](https://api.flutter.dev/flutter/services/FontLoader-class.html) (api.flutter.dev).
- [Testing Flutter apps](https://docs.flutter.dev/testing/overview) và [Accessibility](https://docs.flutter.dev/ui/accessibility) (docs.flutter.dev).
- [Tagging tests](https://pub.dev/packages/test#tagging-tests) (pub.dev).
