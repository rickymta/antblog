---
slug: flutter-animation-implicit-hero-tim-tai-xe
title: "Animation cơ bản: implicit animation và Hero khi tìm tài xế"
summary: "Bài 15: dùng AnimatedSwitcher, AnimatedContainer, AnimatedOpacity, TweenAnimationBuilder và Hero để màn hình tìm tài xế Tới Nơi chuyển trạng thái mượt, rồi gom thời lượng vào một helper đọc cờ disableAnimations (Android); iOS chưa đọc reduceMotion. Có 43 test dùng pump(Duration)."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, animation, hero, accessibility]
language: vi
---

Trên màn hình tìm tài xế của bản đầu tiên, chip "Đang tìm tài xế" bị thay bằng "Tài xế đã nhận" trong đúng một khung hình. Thẻ tài xế bật ra, vòng tiến độ nhảy từ 17% lên 33%. Mọi thứ đều đúng, nhưng mắt không kịp thấy cái gì vừa đổi.

Animation cho mắt có thời gian theo kịp một thay đổi. Nhưng có người bật cài đặt giảm chuyển động, và với họ chuyển động phải bớt hoặc tắt hẳn. Bài này dựng bốn hiệu ứng ngầm định và một Hero cho màn hình tìm tài xế và gom quyết định "có chuyển động hay không" vào một chỗ. Mọi con số trong ví dụ (thời lượng, đánh giá, số phút) là minh hoạ.

Đây là bài 15 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 14 về asset, intl và localization](/articles/flutter-asset-intl-localization-tien-vnd-ngay-gio). Bạn cần biết `setState` ([bài 10](/articles/flutter-statefulwidget-lifecycle-setstate-chon-dich-vu)), `TripStatus` và `Trip.apply` của `toi_noi_core` ([bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core)), và `TripStatusChip` của `toi_noi_ui` ([bài 13](/articles/flutter-theme-material-3-dark-mode-design-system-toi-noi)). Bối cảnh nghiệp vụ: hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một khung hiển thị trạng thái dùng <code>AnimatedSwitcher(duration: 300 ms, child: Text(status.label))</code>. <code>status</code> đổi từ "Đang tìm tài xế" sang "Tài xế đã nhận". Chữ có mờ dần rồi hiện dần không?</p>
<details><summary>Xem đáp án</summary><p>Không. Hai <code>Text</code> cùng kiểu và không có key, nên <code>Widget.canUpdate</code> trả về true: <code>AnimatedSwitcher</code> coi đó là cùng một widget được cập nhật và đổi chữ ngay. Chuyển cảnh chỉ chạy khi widget con mới khác kiểu hoặc khác key. Test <code>cùng kiểu widget, không key</code> chứng minh điều này.</p></details>
</div>

## Implicit animation: bạn đổi đích, widget lo đường đi

Widget animation ngầm định (implicit) có `duration`, thường kèm `curve` và `onEnd`. Ta không điều khiển từng khung hình: ta build lại widget với giá trị đích mới, widget so với giá trị đang hiển thị và tự chạy từ cũ tới mới. Bên trong nó giữ một `AnimationController`; bài này không dùng controller trực tiếp, cách tự điều khiển nằm ở bài 61 của loạt.

| Widget | Chuyển động gì | Dùng trong lab |
|---|---|---|
| `AnimatedContainer` | Màu, viền, bo góc, kích thước, padding, margin... | Thẻ tài xế đổi màu nền và viền |
| `AnimatedOpacity` | Độ mờ | Dòng "tới điểm đón sau ... phút" |
| `AnimatedSwitcher` | Từ widget con cũ sang widget con mới | Nội dung khung trạng thái |
| `TweenAnimationBuilder<T>` | Một giá trị `T` bất kỳ có `Tween` | Vòng tiến độ |

Khi đích đổi, mã nguồn của `ImplicitlyAnimatedWidgetState` lấy giá trị đang hiển thị làm `begin`, đặt đích mới làm `end`, rồi chạy controller lại từ 0 trong đủ `duration`. Hệ quả: đổi đích giữa chừng không nhảy, và lượt mới luôn chạy đủ thời lượng. Hình 1 vẽ lúc vòng đang đi từ 50% lên 100% thì khách huỷ chuyến.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 1. Đồ thị giá trị của vòng tiến độ theo thời gian. Lượt một đi từ 50% hướng tới 100%. Tại 300 ms, giá trị đang hiển thị khoảng 93,75% thì đích đổi thành 0%. Phần còn lại của lượt một bị bỏ. Lượt hai bắt đầu đúng tại 93,75% và chạy đủ 600 ms tới 0%."><defs><marker id="fl15-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<path d="M100 60 L850 60" class="f-none s-line"/><path d="M100 150 L850 150" class="f-none s-line"/>
<path d="M100 36 L100 240" class="f-none s-muted" stroke-width="1.5"/><path d="M100 240 L852 240" class="f-none s-muted" stroke-width="1.5" marker-end="url(#fl15-1-a)"/>
<text x="92" y="64" text-anchor="end" class="t-small">100%</text><text x="92" y="154" text-anchor="end" class="t-small">50%</text><text x="92" y="244" text-anchor="end" class="t-small">0%</text>
<text x="100" y="260" text-anchor="middle" class="t-small">0 ms</text><text x="240" y="260" text-anchor="middle" class="t-small">300 ms</text><text x="380" y="260" text-anchor="middle" class="t-small">600 ms</text><text x="520" y="260" text-anchor="middle" class="t-small">900 ms</text>
<polyline points="240,71 252,69 263,67 275,65 287,64 298,63 310,62 322,61 333,61 345,60 357,60 368,60 380,60" class="f-none s-muted" stroke-width="2"/>
<polyline points="100,150 112,139 123,129 135,119 147,111 158,103 170,96 182,90 193,85 205,81 217,77 228,74 240,71" class="f-none s-pk" stroke-width="3"/>
<polyline points="240,71 263,110 287,145 310,173 333,193 357,208 380,219 403,227 427,232 450,236 473,238 497,240 520,240" class="f-none s-phys" stroke-width="3"/>
<circle cx="100" cy="150" r="4" class="f-pk"/><circle cx="240" cy="71" r="4" class="f-warn"/><circle cx="520" cy="240" r="4" class="f-phys"/>
<text x="108" y="178" class="t-small">Lượt 1: đích 100%</text>
<text x="250" y="44" class="t-b">Tại 300 ms đích đổi thành 0%</text>
<text x="390" y="82" class="t-small">phần bỏ của lượt 1</text>
<text x="330" y="136" class="t-b t-phys">Lượt 2: bắt đầu tại 93,75%</text>
<text x="330" y="170" class="t-small">không nhảy về 50% hay 100%</text>
<text x="528" y="226" class="t-small">xong sau đủ 600 ms</text>
<text x="846" y="282" text-anchor="end" class="t-small">thời gian</text>
</svg>
</div>
<figcaption>Hình 1. Đổi đích giữa chừng: lượt 1 (xanh dương), phần bị bỏ (xám), lượt 2 (xanh lá). Mỗi lượt dùng easeOutCubic và đủ 600 ms.</figcaption>
</figure>

Vòng tiến độ dùng `TweenAnimationBuilder`, widget cho phép animate một giá trị bất kỳ mà không cần viết `State`:

### progress_ring.dart · 15-animation-co-ban/lib/progress_ring.dart (trích)

```dart
  @override
  Widget build(BuildContext context) {
    return TweenAnimationBuilder<double>(
      // Tween mới ở mỗi build, chỉ có `end`: lần dựng đầu không chạy, lần
      // đổi đích sau đó chạy từ giá trị đang hiển thị.
      tween: Tween<double>(end: progress),
      duration: ToiNoiMotion.duration(context, ToiNoiMotion.slow),
      curve: ToiNoiMotion.enterCurve,
      onEnd: onEnd,
      builder: (context, value, _) {
        return SizedBox.square(
          dimension: size,
          child: Stack(
            alignment: Alignment.center,
            children: [
              SizedBox.expand(
                child: CircularProgressIndicator(
                  value: value,
                  strokeWidth: 10,
                  semanticsLabel: 'Tiến độ chuyến',
                ),
              ),
              Text(
                '${(value * 100).round()}%',
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ],
          ),
        );
      },
    );
  }
```

`Tween<double>(end: progress)` không có `begin`. Ở lần dựng đầu, `TweenAnimationBuilder` đặt `begin` bằng `end` nên vòng hiện đúng đích ngay, không chạy từ 0. Widget này gán `begin` vào chính `Tween` ta truyền vào, nên ta tạo `Tween` mới ở mỗi lần build thay vì giữ chung một hằng.

`progress` lấy từ `TripStatus`: sáu bước của đường đi bình thường chia đều từ 0 đến 1, huỷ thì về 0, bằng một `switch` vét cạn không có nhánh `_`. Test của hình 1 chạy đúng kịch bản: đổi đích sang 1, cho 300 ms trôi, đổi tiếp sang 0, rồi kiểm giá trị ngay khung đổi đích:

### progress_ring_test.dart · 15-animation-co-ban/test/progress_ring_test.dart (trích)

```dart
      host.value = 0;
      await tester.pump(); // khung đổi đích, chưa có thời gian trôi

      // Không nhảy về 0.5 (đích cũ ban đầu) hay 1 (đích vừa bỏ).
      expect(_ringValue(tester), closeTo(midway, 1e-9));

      // Đường cong chạy lại từ đầu với đủ 600 ms.
      await tester.pump(half);
      expect(_ringValue(tester), inExclusiveRange(0, midway));
      await tester.pump(half + const Duration(milliseconds: 1));
      expect(_ringValue(tester), 0);
```

Cách đọc test animation: `pump()` dựng một khung và bắt đầu animation, `pump(d)` cho thời gian trôi thêm `d`, nên ta kiểm được giá trị trung gian mà không đợi thật. Chỗ dễ sai là mốc thời gian đúng bằng `duration`: giá trị đã tới đích nhưng controller chỉ báo hoàn tất khi thời gian trôi quá `duration` (mã nguồn `_InterpolationSimulation.isDone` dùng `>`). Vì thế test cộng thêm 1 ms, và `onEnd` chỉ chạy sau mốc đó.

## AnimatedContainer và AnimatedOpacity: thẻ tài xế khi tới điểm đón

Khi chuyến chuyển sang `arriving`, thẻ tài xế cần nổi lên: nền chuyển sang `primaryContainer`, viền dày từ 1 lên 2, và dòng "tới điểm đón sau khoảng 3 phút" hiện dần. Ta chỉ tính `highlighted` từ `status` rồi đưa giá trị đích vào `AnimatedContainer`:

### driver_card.dart · 15-animation-co-ban/lib/driver_card.dart (trích)

```dart
    final highlighted = status == TripStatus.arriving;
    final duration = ToiNoiMotion.duration(context, ToiNoiMotion.standard);

    return Hero(
      tag: driverHeroTag(driver),
      child: AnimatedContainer(
        key: const Key('driver-card-box'),
        duration: duration,
        curve: ToiNoiMotion.standardCurve,
        decoration: BoxDecoration(
          color: highlighted
              ? scheme.primaryContainer
              : scheme.surfaceContainerLow,
          borderRadius: _radius,
          border: Border.all(
            color: highlighted ? scheme.primary : scheme.outlineVariant,
            width: highlighted ? 2 : 1,
          ),
        ),
```

`AnimatedContainer` nội suy cả `BoxDecoration` (màu, bo góc, viền) nên ta không viết gì thêm cho chuyển động. Màu lấy từ `ColorScheme` của theme bài 13 nên dark mode vẫn đúng.

Dòng ETA dùng `AnimatedOpacity` và luôn chiếm chỗ trong layout, chỉ độ mờ đổi, nên thẻ không đổi chiều cao khi dòng hiện ra:

### driver_card.dart · 15-animation-co-ban/lib/driver_card.dart (trích)

```dart
                  AnimatedOpacity(
                    duration: ToiNoiMotion.duration(
                      context,
                      ToiNoiMotion.quick,
                    ),
                    curve: ToiNoiMotion.standardCurve,
                    opacity: highlighted ? 1 : 0,
                    child: Padding(
                      padding: const EdgeInsets.only(top: ToiNoiSpacing.sm),
                      child: Text(
                        'Tới điểm đón sau khoảng ${driver.etaMinutes} phút',
                        style: textTheme.bodyMedium,
                      ),
                    ),
                  ),
```

Hai điều kiểm được bằng test. Một, ở 150 ms của 300 ms màu hiển thị bằng `Color.lerp(cũ, mới, Curves.easeInOut.transform(0.5))`. Hai, ở độ mờ 0 dòng ETA không nằm trong cây semantics (`alwaysIncludeSemantics` mặc định false), nên trình đọc màn hình không đọc dòng người dùng không thấy; độ mờ về 1 thì nó xuất hiện lại.

Đường cong mặc định của `ImplicitlyAnimatedWidget` và `AnimatedSwitcher` là `Curves.linear`, nhìn khá cơ học. Lab chọn theo quy ước riêng, không phải luật: 150, 300 và 600 ms cho thay đổi nhỏ, thường và cần được thấy; `easeInOut` cho phần tử đổi tại chỗ, `easeOutCubic` cho vòng tiến độ.

Một cái bẫy với thẻ có thể bấm: `InkWell` vẽ hiệu ứng chạm lên `Material` gần nhất phía trên nó, và tài liệu của `Material` nói hiệu ứng bị che nếu giữa hai bên có widget tô nền như `BoxDecoration.color`. Vì vậy `Material` trong suốt nằm bên trong `AnimatedContainer`.

## AnimatedSwitcher: đổi nội dung theo TripStatus

Màu và độ mờ là giá trị liên tục. Khi đổi cả nội dung, từ chip "Đang tìm tài xế" sang chip "Tài xế đã nhận", không có giá trị nào để nội suy. `AnimatedSwitcher` giữ cả widget cũ lẫn mới trong cây lúc chuyển: cái cũ mờ đi, cái mới hiện lên.

### search_status_panel.dart · 15-animation-co-ban/lib/search_status_panel.dart (trích)

```dart
    return AnimatedSwitcher(
      duration: ToiNoiMotion.duration(context, ToiNoiMotion.standard),
      switchInCurve: Curves.easeOut,
      switchOutCurve: Curves.easeIn,
      // Key theo status: nhờ vậy AnimatedSwitcher thấy "một con khác" và chạy
      // chuyển cảnh. Thiếu key thì cùng kiểu widget nên chỉ cập nhật tại chỗ.
      child: _StatusMessage(key: ValueKey(status), status: status),
    );
```

`ValueKey(status)` là mấu chốt: `AnimatedSwitcher` so widget con mới với widget con hiện tại bằng `Widget.canUpdate`, khác key thì cho cái cũ "đi ra" và dựng cái mới. Transition mặc định là `FadeTransition`. Hình 2 vẽ những gì có trong cây ở từng mốc, theo đúng các lần `pump` của test.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Dòng thời gian của AnimatedSwitcher qua bốn mốc kiểm tra: sau khung dựng, 150 ms, 300 ms và 301 ms. Nội dung cũ mờ dần rồi ở độ mờ 0 cho tới khi bị gỡ ở mốc 301 ms. Nội dung mới hiện dần rồi đứng yên ở độ mờ 1. Cây có hai nội dung ở ba mốc đầu và một nội dung ở mốc cuối."><defs><marker id="fl15-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<path d="M150 44 L840 44" class="f-none s-muted" stroke-width="1.5" marker-end="url(#fl15-2-a)"/>
<path d="M150 38 L150 50" class="f-none s-muted"/><path d="M340 38 L340 50" class="f-none s-muted"/><path d="M530 38 L530 50" class="f-none s-muted"/><path d="M720 38 L720 50" class="f-none s-muted"/>
<text x="150" y="28" text-anchor="middle" class="t-mono">pump()</text><text x="340" y="28" text-anchor="middle" class="t-mono">+150 ms</text><text x="530" y="28" text-anchor="middle" class="t-mono">+300 ms</text><text x="720" y="28" text-anchor="middle" class="t-mono">+1 ms</text>
<text x="20" y="104" class="t-b">Nội dung cũ</text><text x="20" y="122" class="t-small">key: searching</text>
<rect x="150" y="80" width="380" height="50" rx="8" class="f-warn-soft s-warn"/><text x="165" y="110" class="t-small">mờ dần, độ mờ từ 1 về 0</text>
<rect x="530" y="80" width="190" height="50" rx="8" class="f-surface s-warn"/><text x="545" y="110" class="t-small">độ mờ 0, chưa bị gỡ</text>
<text x="728" y="110" class="t-b t-warn">gỡ khỏi cây</text>
<text x="20" y="184" class="t-b">Nội dung mới</text><text x="20" y="202" class="t-small">key: accepted</text>
<rect x="150" y="160" width="380" height="50" rx="8" class="f-pk-soft s-pk"/><text x="165" y="190" class="t-small">hiện dần, độ mờ từ 0 lên 1</text>
<rect x="530" y="160" width="310" height="50" rx="8" class="f-surface s-pk"/><text x="545" y="190" class="t-small">độ mờ 1, đứng yên</text>
<text x="20" y="262" class="t-b">Cây có</text>
<text x="150" y="262" text-anchor="middle" class="t-mono">2 nội dung</text><text x="340" y="262" text-anchor="middle" class="t-mono">2 nội dung</text><text x="530" y="262" text-anchor="middle" class="t-mono">2 nội dung</text><text x="720" y="262" text-anchor="middle" class="t-mono">1 nội dung</text>
<text x="20" y="290" class="t-small">Bốn mốc là các lần kiểm của test, không vẽ theo tỉ lệ thời gian.</text>
</svg>
</div>
<figcaption>Hình 2. Hai widget cùng nằm trong cây suốt lượt chuyển; widget cũ chỉ bị gỡ khi controller báo hoàn tất.</figcaption>
</figure>

Sau khung dựng và 150 ms, cả hai chữ cùng có trong cây, độ mờ mỗi cái nằm trong khoảng mở (0, 1). Sau đúng 300 ms, chữ mới đã rõ nhưng chữ cũ vẫn còn; cần thêm 1 ms nó mới bị gỡ:

### search_status_panel_test.dart · 15-animation-co-ban/test/search_status_panel_test.dart (trích)

```dart
      // Đúng 300 ms: giá trị đã tới đích nhưng controller chỉ báo hoàn tất
      // khi thời gian trôi quá duration, nên nội dung cũ chưa bị gỡ.
      expect(_opacityOf(tester, accepted), 1);
      expect(find.text(searching), findsOneWidget);

      await tester.pump(const Duration(milliseconds: 1));
      expect(find.text(searching), findsNothing);
      expect(find.text(accepted), findsOneWidget);
```

Test không có key chứng minh điều nêu trong phần "Thử trước khi đọc": với `Text(value)` không key, sau 150 ms chữ cũ đã mất; thêm `ValueKey(value)` thì cả hai cùng có. Khung này cũng tránh `CircularProgressIndicator` không có `value`, vì nó quay mãi và `pumpAndSettle` không bao giờ xong.

## Hero: thẻ tài xế bay sang màn chi tiết

Khi khách bấm vào thẻ tài xế để xem biển số, ta muốn thẻ "bay" từ vị trí của nó sang đầu màn chi tiết. Đó là việc của `Hero`: bọc một widget ở màn nguồn và một widget ở màn đích bằng cùng `tag`. Khi `Navigator` đẩy một `PageRoute`, `HeroController` ghép các cặp cùng tag và bay một bản sao qua `Overlay` của Navigator, còn hai bản gốc tạm là chỗ giữ chỗ cùng cỡ (child của chúng vẫn nằm trong `Offstage`, theo `heroes.dart`). Test của lab xác nhận điều đó: giữa chặng, `find.byKey` chỉ ra đúng một thẻ, nằm giữa vị trí đầu và cuối; bản trong `Offstage` không được tính vì `find` mặc định bỏ qua widget offstage.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Chuyến bay của Hero. Thẻ tài xế ở màn tìm tài xế được thay bằng chỗ giữ chỗ; một bản sao bay trong Overlay của Navigator theo animation của route, rồi hạ cánh thành thẻ ở đầu màn chi tiết."><defs><marker id="fl15-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="30" width="260" height="240" rx="10" class="f-surface s-line"/><text x="36" y="56" class="t-b">Màn tìm tài xế</text>
<rect x="36" y="80" width="228" height="56" rx="8" class="f-pk-soft s-pk"/><text x="48" y="104" class="t-b">DriverCard</text><text x="48" y="124" class="t-mono">driver-card-drv-001</text>
<text x="36" y="170" class="t-small">Khi đang bay, chỗ này</text><text x="36" y="188" class="t-small">là chỗ giữ chỗ cùng cỡ.</text>
<rect x="600" y="30" width="260" height="240" rx="10" class="f-surface s-line"/><text x="616" y="56" class="t-b">Màn chi tiết</text>
<rect x="616" y="110" width="228" height="56" rx="8" class="f-pk-soft s-pk"/><text x="628" y="134" class="t-b">DriverCard</text><text x="628" y="154" class="t-mono">driver-card-drv-001</text>
<text x="616" y="200" class="t-small">Cùng tag, nên chuyến bay</text><text x="616" y="218" class="t-small">kết thúc ở đây.</text>
<rect x="330" y="86" width="220" height="56" rx="8" class="f-warn-soft s-warn"/><text x="342" y="110" class="t-b">Bản bay</text><text x="342" y="130" class="t-small">trong Overlay của Navigator</text>
<path d="M266 108 L328 112" class="f-none s-warn" stroke-width="2" marker-end="url(#fl15-3-a)"/>
<path d="M552 118 L614 132" class="f-none s-warn" stroke-width="2" marker-end="url(#fl15-3-a)"/>
<text x="440" y="190" text-anchor="middle" class="t-b">Thời lượng bay = thời lượng route</text>
<text x="440" y="210" text-anchor="middle" class="t-small">curve của Hero mặc định fastOutSlowIn</text>
<text x="440" y="240" text-anchor="middle" class="t-small">route dài 0 ms thì không có chuyến bay</text>
</svg>
</div>
<figcaption>Hình 3. Mũi tên là chiều bay, từ màn nguồn tới màn đích. Chuyến bay chạy theo animation của route và cần hai Hero cùng tag.</figcaption>
</figure>

Hai điều ta phải tự lo. Thứ nhất là tag: một `PageRoute` không được có hai `Hero` cùng tag, nên tag gắn với **thực thể** chứ không với loại widget. Thứ hai là thời lượng: chuyến bay chạy theo animation của route (mã nguồn lấy animation của route đích khi push, của route nguồn khi pop), nên muốn kiểm soát thời lượng bay thì kiểm soát thời lượng route. Ta dùng `PageRouteBuilder` và đưa thời lượng qua `ToiNoiMotion`:

### driver_detail_screen.dart · 15-animation-co-ban/lib/driver_detail_screen.dart (trích)

```dart
Route<void> driverDetailRoute(
  BuildContext context, {
  required Driver driver,
  required TripStatus status,
}) {
  final duration = ToiNoiMotion.duration(context, ToiNoiMotion.standard);
  return PageRouteBuilder<void>(
    transitionDuration: duration,
    reverseTransitionDuration: duration,
    pageBuilder: (context, animation, secondaryAnimation) =>
        DriverDetailScreen(driver: driver, status: status),
    transitionsBuilder: (context, animation, secondaryAnimation, child) =>
        FadeTransition(opacity: animation, child: child),
  );
}
```

Màn chi tiết dùng lại chính `DriverCard`, nên hai `Hero` có sẵn cùng tag `'driver-card-${driver.id}'`. Bản bay nằm trong `Overlay` của Navigator chứ không dưới `Scaffold` của hai màn, nên `Material` trong suốt nằm ngay trong child của `Hero` để chữ vẫn có kiểu chữ mặc định.

Tag trùng là lỗi hay gặp nhất và chỉ lộ khi chuyển trang. Màn hình có hai thẻ cùng một tài xế dựng không lỗi, nhưng bấm vào thẻ thì ném `FlutterError` "There are multiple heroes that share the same tag within a subtree":

### hero_test.dart · 15-animation-co-ban/test/hero_test.dart (trích)

```dart
      await tester.pumpWidget(_list([a, a]));
      await tester.tap(find.text(a.name).first);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));

      final error = tester.takeException();
      expect(error, isA<FlutterError>());
      expect(
        (error! as FlutterError).toString(),
        contains('multiple heroes that share the same tag'),
      );
```

Việc kiểm tra nằm trong `assert` của framework, nên chỉ chạy ở debug và test. Hai thẻ của hai tài xế khác nhau thì mở chi tiết không lỗi, và `driverHeroTag` của ba tài xế mẫu đôi một khác nhau.

## Giảm chuyển động: một chỗ quyết định thời lượng

Mỗi widget ở trên lấy thời lượng từ `ToiNoiMotion.duration(context, ...)` thay vì tự gõ `Duration`. Hàm này trả `Duration.zero` khi cờ `disableAnimations` bật (Android "Remove animations"); iOS Reduce Motion chưa được đọc, xem bên dưới:

### motion.dart · 15-animation-co-ban/lib/motion.dart (trích)

```dart
  static bool reduced(BuildContext context) =>
      MediaQuery.disableAnimationsOf(context);

  /// Trả [normal], hoặc `Duration.zero` khi [reduced].
  static Duration duration(BuildContext context, Duration normal) =>
      reduced(context) ? Duration.zero : normal;
```

Widget dựng lại khi cài đặt đổi vì `disableAnimationsOf` đăng ký phụ thuộc vào cờ đó (route chỉ lấy thời lượng một lần, lúc tạo). Với `Duration.zero`, `AnimatedSwitcher` đổi nội dung ngay trong khung đầu, vòng tiến độ và `AnimatedContainer` nhảy tới đích, và route dài 0 ms nên `Hero` không bay. Nội dung vẫn hiện đủ (chip, câu mô tả, phần trăm); ta chỉ bỏ chuyển động.

Framework cũng đã tự giảm. Theo mã nguồn `AnimationController`, khi cờ `disableAnimations` của nền tảng bật, controller có `animationBehavior` mặc định (như controller trong widget ngầm định) chạy với 5% thời lượng: test đo `AnimatedContainer` 1 giây tới đích sau 60 ms và chưa tới sau 40 ms. Vậy tại sao còn cần helper? Ba lý do, đều kiểm được.

| Tình huống | Kết quả | Bằng chứng |
|---|---|---|
| Không bật gì | Chạy đủ thời lượng | `motion_test.dart` |
| Nền tảng bật `disableAnimations`, widget không đọc | Framework rút còn 5%: 600 ms thành 30 ms, khoảng 2 khung hình ở 60 Hz, chưa phải tắt hẳn | `motion_test.dart` |
| Widget đi qua `ToiNoiMotion` | `Duration.zero`: tới đích trong khung đầu, `Hero` không bay | Test từng widget |

Một, 5% của 600 ms là 30 ms, khoảng 2 khung hình ở 60 Hz, chưa phải tắt hẳn. Hai, chú thích của `MediaQueryData.disableAnimations` trong Flutter 3.47 nói ghi đè giá trị này bằng `MediaQuery` không ảnh hưởng animation của framework nhưng có ích cho test và widget tự đọc cờ; `support.dart` của lab ép giảm chuyển động đúng bằng cách đó, nên test chứng minh phần việc của helper mà không lẫn với phần 5%. Ba, theo cùng chú thích, Reduce Motion của iOS đi qua cờ riêng `AccessibilityFeatures.reduceMotion` và không đặt cờ `disableAnimations`. Lab chưa đọc cờ đó, và một test ghi lại giới hạn: bật `reduceMotion` thì `ToiNoiMotion.reduced` vẫn là false.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 4. Luồng quyết định thời lượng. Cài đặt disableAnimations của hệ điều hành đi vào MediaQuery, rồi vào ToiNoiMotion.duration, rồi tới năm nơi dùng: AnimatedSwitcher, TweenAnimationBuilder, AnimatedContainer, AnimatedOpacity và PageRouteBuilder. Test ghi đè giá trị tại MediaQuery. Cờ reduceMotion của iOS không nằm trong luồng này."><defs><marker id="fl15-4-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl15-4-b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="95" width="210" height="90" rx="10" class="f-phys-soft s-phys"/><text x="36" y="124" class="t-b">Hệ điều hành</text><text x="36" y="146" class="t-mono">disableAnimations</text><text x="36" y="166" class="t-small">Android: Remove animations</text>
<rect x="260" y="95" width="160" height="90" rx="10" class="f-phys-soft s-phys"/><text x="276" y="124" class="t-b">MediaQuery</text><text x="276" y="146" class="t-mono">disableAnimationsOf</text><text x="276" y="166" class="t-small">dựng lại khi đổi</text>
<rect x="460" y="95" width="190" height="90" rx="10" class="f-pk-soft s-pk"/><text x="476" y="124" class="t-b">ToiNoiMotion</text><text x="476" y="146" class="t-mono">duration(context, d)</text><text x="476" y="166" class="t-small">Duration.zero hoặc d</text>
<rect x="690" y="20" width="185" height="38" rx="8" class="f-phys-soft s-phys"/><text x="702" y="43" class="t-mono">AnimatedSwitcher</text>
<rect x="690" y="68" width="185" height="38" rx="8" class="f-phys-soft s-phys"/><text x="702" y="91" class="t-mono">TweenAnimationBuilder</text>
<rect x="690" y="116" width="185" height="38" rx="8" class="f-phys-soft s-phys"/><text x="702" y="139" class="t-mono">AnimatedContainer</text>
<rect x="690" y="164" width="185" height="38" rx="8" class="f-phys-soft s-phys"/><text x="702" y="187" class="t-mono">AnimatedOpacity</text>
<rect x="690" y="212" width="185" height="38" rx="8" class="f-phys-soft s-phys"/><text x="702" y="235" class="t-mono">PageRouteBuilder</text>
<path d="M232 140 L258 140" class="f-none s-phys" stroke-width="2" marker-end="url(#fl15-4-a)"/>
<path d="M422 140 L458 140" class="f-none s-phys" stroke-width="2" marker-end="url(#fl15-4-a)"/>
<path d="M652 128 L688 39" class="f-none s-phys" stroke-width="1.5" marker-end="url(#fl15-4-a)"/>
<path d="M652 134 L688 87" class="f-none s-phys" stroke-width="1.5" marker-end="url(#fl15-4-a)"/>
<path d="M652 140 L688 135" class="f-none s-phys" stroke-width="1.5" marker-end="url(#fl15-4-a)"/>
<path d="M652 146 L688 183" class="f-none s-phys" stroke-width="1.5" marker-end="url(#fl15-4-a)"/>
<path d="M652 152 L688 231" class="f-none s-phys" stroke-width="1.5" marker-end="url(#fl15-4-a)"/>
<rect x="250" y="222" width="220" height="56" rx="8" class="f-warn-soft s-warn"/><text x="262" y="244" class="t-b">Test ghi đè ở đây</text><text x="262" y="264" class="t-small">copyWith(disableAnimations)</text>
<path d="M340 220 L340 187" class="f-none s-warn" stroke-width="2" marker-end="url(#fl15-4-b)"/>
<rect x="20" y="222" width="190" height="56" rx="8" class="f-surface s-warn"/><text x="32" y="244" class="t-b">iOS: reduceMotion</text><text x="32" y="264" class="t-small">cờ riêng, lab chưa đọc</text>
</svg>
</div>
<figcaption>Hình 4. Cờ của hệ điều hành đi theo mũi tên tới năm widget. Hộp cam là chỗ test ép giá trị; hộp viền cam là cờ iOS chưa nằm trong luồng.</figcaption>
</figure>

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với animation cơ bản</span>
<ul>
<li><strong><code>AnimatedSwitcher</code> đổi nội dung mà không có hiệu ứng.</strong> Dấu hiệu: chữ đổi ngay. Nguyên nhân: widget con cũ và mới cùng kiểu, không key, nên <code>Widget.canUpdate</code> là true. Cách sửa: <code>ValueKey</code> theo giá trị quyết định nội dung, như <code>ValueKey(status)</code>.</li>
<li><strong>Hai <code>Hero</code> cùng tag trong một route.</strong> Dấu hiệu: bấm vào thẻ thì ra <code>FlutterError</code> "multiple heroes that share the same tag", dù màn hình dựng bình thường. Nguyên nhân: tag cố định như <code>'driver-card'</code>, hoặc một thực thể xuất hiện hai lần. Cách sửa: tag theo thực thể; nếu có hai thẻ cùng thực thể, thêm tiền tố theo vị trí và truyền tag sang màn đích.</li>
<li><strong><code>Hero</code> không bay.</strong> Dấu hiệu: màn mới hiện ra như chuyển trang thường. Nguyên nhân: Hero phải có mặt ở khung đầu tiên của animation trang mới (tài liệu của <code>Hero</code>), nên thẻ chỉ dựng sau khi tải xong dữ liệu sẽ không bay; hai tag khác nhau; hoặc route không phải <code>PageRoute</code> (mã nguồn <code>HeroController</code> chỉ xét cặp <code>PageRoute</code>, nên dialog và bottom sheet không có Hero). Cách sửa: dựng thẻ ngay ở khung đầu, kiểm tag, dùng <code>PageRoute</code>.</li>
<li><strong><code>pumpAndSettle</code> treo trong test.</strong> Dấu hiệu: "pumpAndSettle timed out". Nguyên nhân: có animation lặp vô hạn, ví dụ <code>CircularProgressIndicator</code> không có <code>value</code>. Cách sửa: <code>pump(Duration)</code> cho đoạn có spinner.</li>
<li><strong>Kỳ vọng animation đã xong sau <code>pump(duration)</code>.</strong> Dấu hiệu: <code>onEnd</code> chưa chạy, widget cũ của <code>AnimatedSwitcher</code> còn đó. Nguyên nhân: controller chỉ báo hoàn tất khi thời gian trôi quá <code>duration</code>. Cách sửa: bơm thêm 1 ms, hoặc <code>pumpAndSettle</code> khi không có animation lặp.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/15-animation-co-ban
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: `flutter analyze` 0 issue, 43 test xanh. Mức: **Kiểm chứng tự động**, không cần thiết bị; test dùng đồng hồ giả của `flutter_test` nên không phụ thuộc thời gian thật.

| Test | Chứng minh điều gì |
|---|---|
| `motion_test.dart` | Helper trả `Duration.zero` khi bật; đọc cờ nền tảng thật và đổi khi đang chạy; `reduceMotion` của iOS chưa được đọc; framework rút còn 5% (1 giây thành 50 ms) |
| `progress_ring_test.dart` | Lần dựng đầu không chạy; giá trị trung gian nằm giữa cũ và mới; đổi đích giữa chừng chạy tiếp từ giá trị đang hiển thị; huỷ lúc 50% sau 300 ms hiện 6%; `onEnd` đúng lúc; vòng không `value` làm `pumpAndSettle` hết thời gian chờ |
| `search_status_panel_test.dart` | Giữa chừng có hai nội dung với độ mờ dở dang; xong thì chỉ còn một; tắt animation thì đổi trong khung đầu; thiếu key thì không chuyển cảnh |
| `driver_card_test.dart` | Màu và viền trung gian đúng đường cong; ETA mờ dần và vào/ra khỏi semantics; tắt animation thì đổi ngay |
| `hero_test.dart` | Tag đôi một khác nhau; tag trùng ném lỗi khi chuyển trang (kể cả khi tắt animation); giữa chuyến bay thẻ nằm giữa hai vị trí; tắt animation thì không bay; quay lại về đúng chỗ cũ |
| `find_driver_screen_test.dart` | Cả hành trình từ `requested` tới `completed`; thẻ xuất hiện khi tài xế nhận; huỷ chỉ ở ba trạng thái; mở và đóng màn chi tiết; không có animation lặp vô hạn |

Khi ta thử bỏ nhánh `Duration.zero` của helper, năm test "tắt animation" đều đỏ, nên chúng thật sự kiểm helper.

**Chưa làm**: chạy trên iOS Simulator, Android emulator hay máy thật (máy kiểm chưa có Android SDK và Xcode đầy đủ), nên chưa nhìn chuyển động bằng mắt, chưa đo độ mượt, chưa thử bật "Remove animations" hay Reduce Motion thật. Chưa đọc cờ `reduceMotion` của iOS, chưa có animation tường minh, `Hero` tuỳ biến hay chuyển trang theo nền tảng; `ToiNoiMotion` còn nằm trong lab, chưa chuyển sang `toi_noi_ui`. Muốn chạy lab, sinh runner một lần: `flutter create --platforms=ios,android --project-name toi_noi_15 --org vn.toinoi .`.

## Bài tập

1. Vòng tiến độ đang ở 50% (`arriving`). Khách huỷ chuyến, tiến độ về 0. Sau 300 ms chữ trên vòng hiện bao nhiêu phần trăm, và vì sao không phải 25%?
2. Màn "Chuyến gần đây" có thêm thẻ của đúng tài xế đang ở khung "Tài xế hiện tại". Bấm thẻ nào cũng phải mở chi tiết mà không lỗi, và đúng thẻ vừa bấm bay đi. Sửa thế nào?
3. Người dùng iPhone bật Reduce Motion. `ToiNoiMotion.reduced` trả gì, test nào ghi lại điều đó, và hướng sửa là gì?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Khoảng 6%. Vòng chạy từ 0,5 về 0 bằng <code>easeOutCubic</code>; ở nửa thời gian đường cong này đã đi khoảng 87,5% quãng đường, nên còn lại 0,5 × (1 − 0,875), khoảng 0,0625. Không phải 25% vì đường cong không tuyến tính. Test <code>huỷ lúc 50%</code> kiểm cả giá trị lẫn chữ "6%".</p>
<p><strong>2.</strong> Hai thẻ cùng tài xế trên một route sẽ trùng tag. Thêm tham số <code>heroScope</code> (ví dụ <code>'current'</code> và <code>'recent'</code>) cho <code>DriverCard</code> và <code>DriverDetailScreen</code>, đặt tag là <code>'$heroScope-driver-card-${driver.id}'</code>, và truyền <code>heroScope</code> của thẻ được bấm sang <code>driverDetailRoute</code>. Chỉ cặp cùng scope mới bay. Mẫu test: hai tài xế khác nhau ở <code>hero_test.dart</code>.</p>
<p><strong>3.</strong> <code>false</code>: Reduce Motion của iOS đi qua <code>AccessibilityFeatures.reduceMotion</code>, còn <code>reduced</code> chỉ đọc <code>MediaQuery.disableAnimationsOf</code>; test <code>cờ reduceMotion riêng của iOS chưa được đọc</code> ghi lại. Hướng sửa: đọc thêm <code>reduceMotion</code> từ <code>platformDispatcher</code>, kiểm widget có dựng lại khi cài đặt đổi, và thử trên iOS thật.</p>
</details>

Bài trước: [Asset, intl và localization](/articles/flutter-asset-intl-localization-tien-vnd-ngay-gio). Bài tiếp: [Accessibility, responsive và adaptive](/articles/flutter-accessibility-responsive-adaptive-ios-android), nơi ta xử lý `Semantics`, text scale lớn và cách một giao diện thích ứng với iOS và Android.

## Tài liệu gốc

- [Implicit animations](https://docs.flutter.dev/ui/animations/implicit-animations), [Hero animations](https://docs.flutter.dev/ui/animations/hero-animations) và [Introduction to animations](https://docs.flutter.dev/ui/animations) (docs.flutter.dev).
- [ImplicitlyAnimatedWidget](https://api.flutter.dev/flutter/widgets/ImplicitlyAnimatedWidget-class.html), [TweenAnimationBuilder](https://api.flutter.dev/flutter/widgets/TweenAnimationBuilder-class.html) và [AnimatedSwitcher](https://api.flutter.dev/flutter/widgets/AnimatedSwitcher-class.html) (api.flutter.dev).
- [AnimatedContainer](https://api.flutter.dev/flutter/widgets/AnimatedContainer-class.html) và [AnimatedOpacity](https://api.flutter.dev/flutter/widgets/AnimatedOpacity-class.html) (api.flutter.dev).
- [Hero](https://api.flutter.dev/flutter/widgets/Hero-class.html) và [PageRouteBuilder](https://api.flutter.dev/flutter/widgets/PageRouteBuilder-class.html) (api.flutter.dev).
- [Curves](https://api.flutter.dev/flutter/animation/Curves-class.html) và [AnimationBehavior](https://api.flutter.dev/flutter/animation/AnimationBehavior.html) (api.flutter.dev).
- [MediaQueryData.disableAnimations](https://api.flutter.dev/flutter/widgets/MediaQueryData/disableAnimations.html), [MediaQuery.disableAnimationsOf](https://api.flutter.dev/flutter/widgets/MediaQuery/disableAnimationsOf.html) và [AccessibilityFeatures.reduceMotion](https://api.flutter.dev/flutter/dart-ui/AccessibilityFeatures/reduceMotion.html) (api.flutter.dev).
- [WidgetTester.pump](https://api.flutter.dev/flutter/flutter_test/WidgetTester/pump.html) (api.flutter.dev).
- [Motion](https://developer.apple.com/design/human-interface-guidelines/motion) (developer.apple.com).
