---
slug: flutter-hieu-nang-rebuild-isolate-devtools
title: "Hiệu năng: rebuild, const, RepaintBoundary, isolate và DevTools"
summary: "Bài 41: đo trước khi sửa hiệu năng app Tới Nơi. Đếm rebuild và paint bằng test (const, tách widget, select, RepaintBoundary), danh sách lười, ResizeImage, Isolate.run cho JSON lớn, DevTools ở profile mode. flutter test không đo được mili giây khung hình."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, performance, isolate, devtools]
language: vi
---

Tài xế Tới Nơi mở màn hình lịch sử thu nhập và chờ gần một giây mới thấy dòng đầu tiên. Cùng lúc, trên app khách, ô "tài xế tới sau 30 giây" đếm ngược từng giây và chấm vị trí nhấp nháy trên bản đồ; trên máy cũ, cả màn hình như bị kéo theo mỗi nhịp. Cả hai đều là "giật", nhưng một bên là việc nặng nằm nhầm chỗ, một bên là việc nhẹ bị làm quá nhiều lần.

Nguyên tắc của bài là **đo trước khi sửa**, và ta nói thẳng giới hạn: `flutter test` chạy ở debug, không có GPU, nên không đo được mili giây khung hình. Thứ đếm được là số lần build, số lần paint, số dòng được tạo và số byte ảnh giải mã. Chúng không phụ thuộc máy nên biến được thành test chặn lỗi quay lại. Mili giây thật cần `flutter run --profile` trên thiết bị thật; lab không có thiết bị nên bài **không có số đo nào trên điện thoại**.

Đây là bài 41 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 40 về platform channel, Pigeon và FFI](/articles/flutter-platform-channel-pigeon-ffi-goi-native). Ta nên đọc trước [bài 17 về ranh giới rebuild](/articles/flutter-state-inheritedwidget-changenotifier-bao-gia), [bài 18 về Riverpod](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen) và [bài 05](/articles/dart-future-async-await-stream-bao-gia-va-vi-tri) (có `Isolate.run` sơ lược). Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); mọi con số thời gian và quy mô là minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một màn hình có <code>setState</code> ở gốc, chạy mỗi giây. Bên trong có <code>QuoteCard(fare: fare)</code> và <code>fare</code> không đổi qua các lần build. Sau 5 giây, <code>QuoteCard</code> được build lại mấy lần?</p>
<details><summary>Xem đáp án</summary><p>5 lần. Mỗi lần <code>build</code> của cha tạo một <code>QuoteCard</code> mới, và framework chỉ bỏ qua khi gặp đúng cùng một instance (như widget <code>const</code>). Cùng giá trị <code>fare</code> là chưa đủ; test ở mục Rebuild đếm ra đúng 5.</p></details>
</div>

## Đo trước khi sửa: đếm ở máy học, đo ở máy thật

Mỗi công cụ chỉ thấy một phần sự thật (Hình 1).

| Câu hỏi | Công cụ | Chạy ở đâu |
|---|---|---|
| Widget nào build lại, bao nhiêu lần? | `BuildLog`, `debugOnRebuildDirtyWidget`, DevTools | test và máy phát triển |
| Vùng nào bị vẽ lại? | `CountingPainter`, `debugRepaintRainbowEnabled` | test và máy phát triển |
| Mỗi khung mất bao nhiêu mili giây? | DevTools Performance | profile mode, máy thật |
| Người dùng có thấy giật không? | `FrameTiming` | release, máy người dùng |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 1. Ba bước theo thứ tự: đo trên thiết bị bằng DevTools ở profile mode, sửa rồi khoá kết quả bằng test đếm, giám sát khung hụt ở bản release bằng FrameTiming."><defs><marker id="fl41-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="250" height="150" rx="10" class="f-phys-soft s-phys"/><text x="36" y="62" class="t-b">1. Đo trên thiết bị</text><text x="36" y="92" class="t-small">DevTools Performance,</text><text x="36" y="114" class="t-mono">flutter run --profile</text><text x="36" y="144" class="t-small">Thấy: mili giây build</text><text x="36" y="164" class="t-small">và raster mỗi khung.</text>
<rect x="320" y="30" width="250" height="150" rx="10" class="f-pk-soft s-pk"/><text x="336" y="62" class="t-b">2. Sửa, khoá bằng test</text><text x="336" y="92" class="t-small">flutter test đếm build,</text><text x="336" y="114" class="t-small">paint, dòng, byte ảnh.</text><text x="336" y="144" class="t-small">Không thấy: mili giây,</text><text x="336" y="164" class="t-small">GPU, raster.</text>
<rect x="620" y="30" width="240" height="150" rx="10" class="f-phys-soft s-phys"/><text x="636" y="62" class="t-b">3. Giám sát bản release</text><text x="636" y="92" class="t-small">FrameTiming qua</text><text x="636" y="114" class="t-mono">addTimingsCallback</text><text x="636" y="144" class="t-small">Thấy: khung hụt trên</text><text x="636" y="164" class="t-small">máy người dùng.</text>
<path d="M272 105 L318 105" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-1-a)"/>
<path d="M572 105 L618 105" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-1-a)"/>
<text x="20" y="224" class="t-small">Kết luận về tốc độ chỉ rút ra ở bước 1 và 3; bước 2 chứng minh sửa đúng chỗ và chặn regression.</text>
</svg>
</div>
<figcaption>Hình 1. Ba công cụ, ba câu hỏi. Test đếm không thay được đo trên thiết bị; nó giữ cho điều đã sửa không hỏng lại.</figcaption>
</figure>

Quy trình: nghi ngờ một màn hình, đo ở profile mode, chọn **một** thay đổi, đo lại, rồi viết test đếm để khoá kết quả: lần sau ai thêm `setState` ở gốc, test đỏ ngay trên CI.

Tài liệu Flutter nêu profile mode bị tắt trên simulator và emulator vì hành vi không đại diện, nên `flutter run --profile` phải chạy trên máy thật (kiểm tại 11/2026). Trong DevTools, tab **Performance** có biểu đồ khung hình với khung jank tô đỏ, và mục Enhance tracing có **Track Widget Builds**, **Track Layouts**, **Track Paints** để xem sự kiện theo từng widget. Chú thích trong `widgets/debug.dart` nói `WidgetInspectorService` dùng hook `debugOnRebuildDirtyWidget` để thống kê rebuild; ta dùng chính hook đó ở mục sau. Tài liệu không mô tả bộ đếm rebuild của DevTools, nên hãy kiểm trên bản đang dùng.

Con số cần so là **ngân sách khung hình**: khoảng 16 ms ở 60 Hz và khoảng 8 ms ở 120 Hz. Tài liệu `FrameTiming` nói pha build và pha raster mỗi pha không nên vượt ngân sách. `JankMeter` của lab phân loại đúng như vậy:

### frame_budget.dart · 41-performance/lib/frame_budget.dart (trích)

```dart
Duration frameBudget(int refreshHz) {
  if (refreshHz <= 0) {
    throw ArgumentError.value(refreshHz, 'refreshHz', 'phải dương');
  }
  return Duration(
    microseconds: (Duration.microsecondsPerSecond / refreshHz).round(),
  );
}

  bool isMissed(FrameTiming t) =>
      t.buildDuration > budget || t.rasterDuration > budget;

  /// Ghi nhận một lô khung hình, cùng chữ ký với `TimingsCallback`.
  void onTimings(List<FrameTiming> timings) {
    for (final t in timings) {
      _frames++;
      if (isMissed(t)) {
        _missed++;
      }
      if (t.totalSpan > budget) {
        _slow++;
      }
    }
  }
```

Test dùng `FrameTiming` tự dựng: cùng một khung, ổn ở 60 Hz và hụt ở 120 Hz.

### frame_budget_test.dart · 41-performance/test/frame_budget_test.dart (trích)

```dart
    test('cùng một khung 12 ms build: ổn ở 60 Hz, hụt ở 120 Hz', () {
      final frame = _frame(buildMs: 12, rasterMs: 4);

      expect(JankMeter(refreshHz: 60).isMissed(frame), isFalse);
      expect(JankMeter(refreshHz: 120).isMissed(frame), isTrue);
    });
```

Đó mới là phần logic. Số thật chỉ có khi gắn `JankMeter.attach()` ở profile hoặc release trên thiết bị.

## Rebuild: const, tách widget và select

Khi `setState` chạy, Flutter build lại widget đó rồi đi xuống các con. Ở mỗi con, `Element.updateChild` so widget mới với widget cũ: cùng một instance thì bỏ qua, khác instance thì cập nhật và gọi lại `build` (đọc `framework.dart`). Màn hình chưa tối ưu của lab:

### rebuild_screens.dart · 41-performance/lib/rebuild_screens.dart (trích)

```dart
  void _onTick() => setState(() {});

  @override
  Widget build(BuildContext context) {
    BuildLog.hit('screen');
    return Column(
      children: [
        Text('Tài xế tới sau ${widget.ticker.value} giây'),
        QuoteCard(fare: widget.fare), // widget mới mỗi lần build
        const BuildProbe('const-footer'), // cùng một instance mỗi lần
        BuildProbe('plain-footer'), // ignore: prefer_const_constructors
      ],
    );
  }
}
```

Test đếm sau 5 nhịp:

### rebuild_test.dart · 41-performance/test/rebuild_test.dart (trích)

```dart
    testWidgets('5 nhịp: thẻ báo giá và footer thường build lại 5 lần', (
      tester,
    ) async {
      await tester.pumpWidget(
        _app(NaiveTripScreen(ticker: ticker, fare: fare)),
      );
      BuildLog.reset();

      await tick(tester, 5);

      expect(BuildLog.of('screen'), 5);
      // Cước không đổi nhưng widget được tạo mới mỗi lần nên bị gọi lại build.
      expect(BuildLog.of('quote-card'), 5);
      expect(BuildLog.of('plain-footer'), 5);
    });
```

Footer `const` build 0 lần, footer thường 5 lần. Hình 2 đặt bản này cạnh bản đã tách.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 350" role="img" aria-label="Hình 2. Hai cây widget. Bên trái, setState ở NaiveTripScreen làm Text ETA, QuoteCard và footer thường build lại 5 lần, footer const 0 lần. Bên phải, SplitTripScreen không nghe gì, chỉ EtaText build lại 5 lần, QuoteCard 0 lần."><defs><marker id="fl41-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="40" y="26" class="t-b">Trước: setState ở gốc</text><text x="480" y="26" class="t-b">Sau: tách widget</text>
<rect x="40" y="44" width="260" height="56" rx="8" class="f-warn-soft s-warn"/><text x="56" y="68" class="t-b">NaiveTripScreen</text><text x="56" y="88" class="t-small">setState mỗi nhịp</text>
<path d="M55 100 L55 306" class="f-none s-line" stroke-width="2"/>
<path d="M55 146 L68 146" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-2-a)"/><path d="M55 196 L68 196" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-2-a)"/><path d="M55 246 L68 246" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-2-a)"/><path d="M55 296 L68 296" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-2-a)"/>
<rect x="70" y="128" width="250" height="36" rx="8" class="f-warn-soft s-warn"/><text x="84" y="152" class="t-small">Text ETA</text><text x="334" y="152" class="t-b">5 lần</text>
<rect x="70" y="178" width="250" height="36" rx="8" class="f-warn-soft s-warn"/><text x="84" y="202" class="t-mono">QuoteCard(fare: fare)</text><text x="334" y="202" class="t-b">5 lần</text>
<rect x="70" y="228" width="250" height="36" rx="8" class="f-surface s-muted"/><text x="84" y="252" class="t-mono">const BuildProbe</text><text x="334" y="252" class="t-b">0 lần</text>
<rect x="70" y="278" width="250" height="36" rx="8" class="f-warn-soft s-warn"/><text x="84" y="302" class="t-mono">BuildProbe (không const)</text><text x="334" y="302" class="t-b">5 lần</text>
<rect x="480" y="44" width="260" height="56" rx="8" class="f-surface s-muted"/><text x="496" y="68" class="t-b">SplitTripScreen</text><text x="496" y="88" class="t-small">không nghe gì</text>
<path d="M495 100 L495 206" class="f-none s-line" stroke-width="2"/>
<path d="M495 146 L508 146" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-2-a)"/><path d="M495 196 L508 196" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-2-a)"/>
<rect x="510" y="128" width="250" height="36" rx="8" class="f-warn-soft s-warn"/><text x="524" y="152" class="t-mono">EtaText</text><text x="774" y="152" class="t-b">5 lần</text>
<rect x="510" y="178" width="250" height="36" rx="8" class="f-surface s-muted"/><text x="524" y="202" class="t-mono">QuoteCard(fare: fare)</text><text x="774" y="202" class="t-b">0 lần</text>
<rect x="480" y="248" width="24" height="16" rx="3" class="f-warn-soft s-warn"/><text x="514" y="261" class="t-small">build lại mỗi nhịp</text>
<rect x="480" y="278" width="24" height="16" rx="3" class="f-surface s-muted"/><text x="514" y="291" class="t-small">không build lại</text>
</svg>
</div>
<figcaption>Hình 2. Phạm vi rebuild trước và sau khi tách. Mũi tên đi từ cha xuống con; tô màu cảnh báo là widget bị build lại mỗi nhịp.</figcaption>
</figure>

**Tách widget** là cách đáng dùng nhất: phần đổi nằm trong `EtaText` tự nghe `ticker`, cha trở thành `StatelessWidget` không nghe gì.

### rebuild_screens.dart · 41-performance/lib/rebuild_screens.dart (trích)

```dart
  Widget build(BuildContext context) => ValueListenableBuilder<int>(
    valueListenable: ticker,
    builder: (context, seconds, _) {
      BuildLog.hit('eta');
      return Text('Tài xế tới sau $seconds giây');
    },
  );
}
```

**Tham số `child`** của builder cho cùng kết quả: widget tĩnh dựng một lần ngoài builder, nên mỗi nhịp framework gặp lại đúng instance cũ và dừng.

### rebuild_screens.dart · 41-performance/lib/rebuild_screens.dart (trích)

```dart
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: ticker,
    // Dựng một lần trong build của màn hình, không dựng trong builder.
    child: QuoteCard(fare: fare),
    builder: (context, child) {
      BuildLog.hit('builder');
      return Column(children: [Text('Còn ${ticker.value} giây'), child!]);
    },
  );
}
```

**`const`** chỉ có tác dụng khi cả đối số là hằng. `const QuoteCard(fare: Money(25500))` là một instance dùng chung nên build 0 lần; `QuoteCard(fare: fare)` với `fare` là biến, dù cùng giá trị, là instance mới và build 1 lần (hai test đối chiếu nhau trong `rebuild_test.dart`). Với dữ liệu chỉ biết lúc chạy, ta quay về tách widget.

Để biết widget nào bị build mà không gắn nhãn thủ công, ta dùng `debugOnRebuildDirtyWidget`:

### build_log.dart · 41-performance/lib/build_log.dart (trích)

```dart
  factory RebuildRecorder.start() {
    final recorder = RebuildRecorder._();
    debugOnRebuildDirtyWidget = (element, _) {
      recorder.types.add(element.widget.runtimeType.toString());
    };
    return recorder;
  }
```

Callback nằm trong `assert` nên chỉ chạy ở debug. Tham số `builtOnce` chỉ được cập nhật khi `debugPrintRebuildDirtyWidgets` bật (đọc `Element.rebuild`), nên ta bỏ qua nó. Đây là biến toàn cục, nên `stop()` gỡ nó để test sau không bị ảnh hưởng. Sau 2 nhịp, test ghi được `NaiveTripScreen`, `QuoteCard`, `MoneyText` mỗi loại 2 lần, và `BuildProbe` chỉ 2 (footer thường).

Cờ `debugProfileBuildsEnabled` thêm Timeline event cho mỗi widget được build. Nếu test để cờ này bật, `flutter_test` báo lỗi khi thân test kết thúc, trước cả `addTearDown`; lab trả cờ về `false` bằng `try`/`finally`.

Đến **`select`**. Vị trí tài xế đổi nhiều lần mỗi giây, cước hiếm khi đổi; nhãn chỉ hiện cước không nên build lại theo vị trí:

### trip_live.dart · 41-performance/lib/trip_live.dart (trích)

```dart
class FareLabelSelect extends ConsumerWidget {
  /// Tạo nhãn cước dùng `select`.
  const FareLabelSelect({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    BuildLog.hit('fare-select');
    final fare = ref.watch(tripLiveProvider.select((s) => s.trip.fare));
    return Text(fare.toString());
  }
}
```

### select_test.dart · 41-performance/test/select_test.dart (trích)

```dart
    testWidgets('select cước không build lại lần nào', (tester) async {
      await pumpLabels(tester);
      BuildLog.reset();

      await moveDriverTimes(tester, 5);

      expect(BuildLog.of('fare-select'), 0);
    });
```

Tài xế di chuyển 5 lần: nhãn xem cả state build 5 lần, nhãn dùng `select` build 0 lần. Cước đổi thật (`reroute(100)`, 453.000 VND) thì cả hai build đúng 1 lần. Kiểu trả về của `select` quan trọng: `Money` có `==` theo giá trị nên không kích hoạt build lại khi giá trị không đổi, còn `[s.trip.fare]` là `List` mới mỗi lần, không có `==` theo giá trị, nên `FareLabelSelectList` vẫn build 5 lần. Muốn chọn nhiều giá trị, trả về một record.

## Paint: RepaintBoundary

Build và paint là hai giai đoạn khác nhau. Một widget không build lại vẫn có thể bị vẽ lại: `markNeedsPaint` đi ngược lên cha cho tới `RenderObject` đầu tiên là repaint boundary (đọc `RenderObject.markNeedsPaint`), rồi cả layer đó được vẽ lại. Framework tự thêm một `RepaintBoundary` quanh mỗi trang (`routes.dart`); trong lab, đó là layer chứa bản đồ.

Lab dựng một bản đồ tĩnh với chấm vị trí nhấp nháy, và painter ghi lại mỗi lần `paint` được gọi:

### paint_panels.dart · 41-performance/lib/paint_panels.dart (trích)

```dart
class CountingPainter extends CustomPainter {
  /// Tạo painter ghi vào [PaintLog] với [label]; [repaint] là tín hiệu vẽ lại.
  CountingPainter(this.label, {super.repaint});

  /// Nhãn trong [PaintLog].
  final String label;

  @override
  void paint(Canvas canvas, Size size) {
    PaintLog.hit(label);
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = const Color(0xFF0F766E),
    );
  }

  @override
  bool shouldRepaint(CountingPainter oldDelegate) => false;
}

    final pulse = CustomPaint(
      size: const Size(40, 40),
      painter: CountingPainter('pulse', repaint: _controller),
    );
    return SizedBox(
      width: 300,
      height: 200,
      child: Stack(
        children: [
          CustomPaint(
            size: const Size(300, 200),
            painter: CountingPainter('route'),
          ),
          Positioned(
            left: 130,
            top: 80,
            child: widget.isolatePulse ? RepaintBoundary(child: pulse) : pulse,
          ),
        ],
      ),
    );
  }
}
```

### repaint_test.dart · 41-performance/test/repaint_test.dart (trích)

```dart
  testWidgets('không có RepaintBoundary: 10 khung vẽ lại cả bản đồ', (
    tester,
  ) async {
    await pumpPanel(tester, isolate: false);
    PaintLog.reset();

    await runFrames(tester, 10);

    expect(PaintLog.of('pulse'), 10);
    // Bản đồ không đổi gì nhưng cùng layer với chấm nên bị vẽ lại theo.
    expect(PaintLog.of('route'), 10);
  });

  testWidgets('chấm trong RepaintBoundary: bản đồ không bị vẽ lại', (
    tester,
  ) async {
    await pumpPanel(tester, isolate: true);
    PaintLog.reset();

    await runFrames(tester, 10);

    expect(PaintLog.of('pulse'), 10);
    expect(PaintLog.of('route'), 0);
  });
```

Sau 10 khung hình (Hình 3): không có boundary, `route` bị vẽ 10 lần dù không đổi gì; có boundary quanh chấm, `route` vẽ 0 lần. `shouldRepaint` luôn `false` mà `route` vẫn bị vẽ, vì nó chỉ quyết định có đánh dấu vẽ lại khi widget được cập nhật; khi layer chứa nó vẽ lại, `paint` vẫn chạy. Bọc phần tĩnh thay vì phần động cũng khiến bản đồ không bị vẽ lại (một test riêng).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 3. Bên trái, bản đồ route và chấm pulse nằm chung một layer, nên mỗi khung cả hai đều được vẽ lại, route paint 10 lần sau 10 khung. Bên phải, pulse nằm trong RepaintBoundary riêng, route paint 0 lần, pulse paint 10 lần."><defs><marker id="fl41-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="390" height="170" rx="10" class="f-warn-soft s-warn"/><text x="36" y="58" class="t-b">Một layer cho cả bản đồ</text>
<rect x="40" y="80" width="170" height="100" rx="8" class="f-surface s-line"/><text x="56" y="112" class="t-mono">route</text><text x="56" y="136" class="t-small">tĩnh, vẫn bị vẽ</text><text x="56" y="158" class="t-small">paint: 10</text>
<rect x="230" y="80" width="160" height="100" rx="8" class="f-surface s-line"/><text x="246" y="112" class="t-mono">pulse</text><text x="246" y="136" class="t-small">nhấp nháy</text><text x="246" y="158" class="t-small">paint: 10</text>
<path d="M416 115 L464 115" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-3-a)"/><text x="418" y="100" class="t-small">bọc</text>
<rect x="470" y="30" width="190" height="170" rx="10" class="f-surface s-muted"/><text x="486" y="58" class="t-b">Layer của trang</text><text x="486" y="112" class="t-mono">route</text><text x="486" y="136" class="t-small">không vẽ lại</text><text x="486" y="158" class="t-small">paint: 0</text>
<rect x="680" y="30" width="190" height="170" rx="10" class="f-pk-soft s-pk"/><text x="696" y="58" class="t-b">RepaintBoundary</text><text x="696" y="112" class="t-mono">pulse</text><text x="696" y="136" class="t-small">nhấp nháy</text><text x="696" y="158" class="t-small">paint: 10</text>
<text x="20" y="232" class="t-small">Số lần paint sau 10 khung hình, đo bằng CountingPainter trong test.</text>
<text x="20" y="254" class="t-small">Mỗi RepaintBoundary là thêm một layer: bọc phần đổi liên tục, đừng bọc mọi thứ.</text>
</svg>
</div>
<figcaption>Hình 3. Cùng một animation, hai cách chia layer. Mũi tên chỉ phép biến đổi từ trái sang phải: bọc chấm trong RepaintBoundary.</figcaption>
</figure>

Mỗi boundary là một layer riêng nên tốn thêm bộ nhớ; chỉ bọc phần đổi liên tục. Để thấy vùng bị vẽ lại, bật **Highlight repaints** trong DevTools.

## Danh sách lười, cacheExtent và ảnh

Lab so ba cách dựng 1.000 dòng lịch sử, mỗi dòng cao 72 dp, trong khung nhìn 600 dp của test:

### trip_list.dart · 41-performance/lib/trip_list.dart (trích)

```dart
  Widget build(BuildContext context) => switch (strategy) {
    ListStrategy.column => SingleChildScrollView(
      child: Column(children: [for (final trip in trips) createTile(trip)]),
    ),
    ListStrategy.children => ListView(
      scrollCacheExtent: cacheExtent,
      children: [for (final trip in trips) createTile(trip)],
    ),
    ListStrategy.builder => ListView.builder(
      scrollCacheExtent: cacheExtent,
      itemCount: trips.length,
      itemExtent: tripTileHeight,
      itemBuilder: (context, index) => createTile(trips[index]),
    ),
  };
```

| Cách | Widget được tạo | Widget được build |
|---|---|---|
| `SingleChildScrollView` + `Column` | 1.000 | 1.000 |
| `ListView(children:)` | 1.000 | 12 |
| `ListView.builder` | 12 | 12 |

Hàng giữa tinh tế hơn lời khuyên thường nghe: `ListView(children:)` **không** build cả 1.000 dòng, vì `ListView` lười ở bước build. Nhưng 1.000 widget vẫn được tạo, và tạo lại mỗi khi cha dựng lại (một test riêng). `Column` trong `SingleChildScrollView` chịu cả hai chi phí. Cách `builder` còn đặt `itemExtent`; tài liệu nói chiều cao biết trước giúp cơ chế cuộn bớt việc.

Số 12 đến từ vùng đệm: mặc định framework dựng thêm 250 dp ngoài khung nhìn (`RenderAbstractViewport.defaultCacheExtent`), và (600 + 250) / 72 làm tròn lên là 12. Ở Flutter 3.47, `cacheExtent` đã deprecated, ta dùng `scrollCacheExtent`:

### lazy_list_test.dart · 41-performance/test/lazy_list_test.dart (trích)

```dart
    testWidgets('cacheExtent càng lớn thì càng build nhiều lúc dựng đầu', (
      tester,
    ) async {
      final counts = <int>[];
      for (final pixels in [0.0, 250.0, 1000.0]) {
        BuildLog.reset();
        await pumpList(
          tester,
          ListStrategy.builder,
          cacheExtent: ScrollCacheExtent.pixels(pixels),
        );
        counts.add(BuildLog.of('tile'));
        await tester.pumpWidget(const SizedBox());
      }

      expect(counts, [9, 12, 23]);
    });
```

Vùng đệm 0, 250 và 1.000 dp cho 9, 12 và 23 dòng. Đệm lớn giúp cuộn nhanh ít thấy khoảng trống, đổi lại build nhiều hơn ngay từ đầu.

Ảnh được giải mã thành bitmap, tốn width × height × 4 byte bất kể file nén nhỏ thế nào. Ảnh đại diện tài xế 512 × 512 hiển thị ở 48 dp chỉ cần giải mã ở kích thước hiển thị, tính bằng pixel **vật lý**:

### avatar.dart · 41-performance/lib/avatar.dart (trích)

```dart
int decodeSizeFor(double logicalSize, double devicePixelRatio) =>
    (logicalSize * devicePixelRatio).ceil();

ImageProvider<Object> avatarProvider(
  ImageProvider<Object> source, {
  required double logicalSize,
  required double devicePixelRatio,
}) => ResizeImage(source, width: decodeSizeFor(logicalSize, devicePixelRatio));
```

### image_cache_test.dart · 41-performance/test/image_cache_test.dart (trích)

```dart
  testWidgets('avatar 48 dp ở dpr 3 chỉ giải mã 144 px, giữ tỉ lệ', (
    tester,
  ) async {
    final provider = avatarProvider(
      MemoryImage(png),
      logicalSize: 48,
      devicePixelRatio: 3,
    );
    final image = await decode(tester, provider);

    expect(image.width, 144);
    expect(image.height, 144);
    expect(PaintingBinding.instance.imageCache.currentSizeBytes, 144 * 144 * 4);
    // Giảm gần 12,6 lần bộ nhớ ảnh giải mã so với giải mã đầy đủ.
    expect(fullBytes / (144 * 144 * 4), closeTo(12.64, 0.01));
  });
```

Giải mã đầy đủ tốn 1.048.576 byte; ở 144 px (48 dp, `devicePixelRatio` 3) chỉ 82.944 byte, giảm khoảng 12,6 lần. Các test khác: đặt một cạnh thì cạnh kia theo tỉ lệ (400 × 200 với `width: 100` ra 100 × 50); đòi lớn hơn ảnh gốc thì bị chặn ở kích thước gốc trừ khi `allowUpscaling: true`; cùng một ảnh ở hai kích thước là hai mục trong `ImageCache`. `Image.memory`, `Image.asset`, `Image.network`, `Image.file` có tham số `cacheWidth` làm đúng việc này bằng cách bọc nguồn trong `ResizeImage`.

## Isolate.run cho JSON lớn

Lịch sử 20.000 chuyến là chuỗi JSON vài megabyte. Parse nó trên main isolate là việc đồng bộ: trong lúc đó không khung hình nào được dựng. Ta đưa việc sang isolate khác (Hình 4):

### trip_json.dart · 41-performance/lib/trip_json.dart (trích)

```dart
Future<List<Trip>> parseTripsInIsolate(String json) =>
    Isolate.run(() => parseTrips(json), debugName: 'parse-trips');
```

`parseTrips` là hàm top-level thuần, không đụng `dart:ui`; closure chỉ bắt `json`, một `String` bất biến. Theo tài liệu, `Isolate.run` gửi **kết quả** về bằng `Isolate.exit` nên không sao chép; dữ liệu **mutable** qua ranh giới thì bị sao chép, còn chuỗi và đối tượng bất biến đi bằng tham chiếu. Sao chép chạy ở isolate đang gọi, tức main isolate. Test chứng minh ngữ nghĩa sao chép:

### isolate_test.dart · 41-performance/test/isolate_test.dart (trích)

```dart
    test(
      'dữ liệu mutable bị SAO CHÉP: sửa ở isolate kia không ảnh hưởng',
      () async {
        final fares = <int>[12000, 25500];

        final lengthInside = await Isolate.run(() {
          fares.add(453000);
          return fares.length;
        });

        expect(lengthInside, 3);
        expect(fares, [12000, 25500]);
      },
    );
```

Kết quả giống bản parse đồng bộ, và `FormatException` ném trong isolate được ném lại cùng kiểu. Cạm bẫy lớn nhất là closure bắt thêm thứ ta không ngờ tới. Ví dụ: class dưới đây giữ một `ReceivePort`, và closure nhắc tới `port` nên kéo cả `this` đi:

### isolate_test.dart · 41-performance/test/isolate_test.dart (trích)

```dart
class _LoaderWithPort {
  final ReceivePort port = ReceivePort();

  // Closure nhắc tới `port` nên bắt luôn `this`, mà `this` có ReceivePort.
  Future<int> countWrong(String json) =>
      Isolate.run(() => port.sendPort.hashCode + parseTrips(json).length);

  void close() => port.close();
}
```

Kết quả là `ArgumentError` với thông báo `object is unsendable`. Cách sửa là hàm top-level nhận đủ đối số, như `parseTripsInIsolate`.

`compute` của Flutter là lớp vỏ mỏng: trên native, `compute(fun, message)` tương đương `Isolate.run(() => fun(message))`; trên web nó chạy ngay trên main isolate (theo source và tài liệu).

Test cuối không đo mili giây, chỉ hỏi thứ tự của event loop: một `Timer` đã tới hạn chạy trước hay sau khi parse xong. Test này chỉ cho thấy `Isolate.run` trả quyền cho event loop trước khi có kết quả, không chứng minh main isolate hoàn toàn không bị chặn.

### isolate_test.dart · 41-performance/test/isolate_test.dart (trích)

```dart
    test('parse đồng bộ: Timer phải chờ parse xong', () async {
      final order = <String>[];
      final timer = Future<void>(() => order.add('timer'));

      parseTrips(generateTripsJson(5000));
      order.add('parse xong');
      await timer;

      expect(order, ['parse xong', 'timer']);
    });

    test('parse ở isolate khác: Timer chạy trong lúc chờ', () async {
      final order = <String>[];
      final timer = Future<void>(() => order.add('timer'));

      final parsing = parseTripsInIsolate(generateTripsJson(5000));
      await parsing;
      order.add('parse xong');
      await timer;

      expect(order, ['timer', 'parse xong']);
    });
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Hình 4. Dòng thời gian hai isolate. Main isolate liên tục dựng các khung. Isolate.run chuyển việc parse sang isolate worker, worker chạy khối parse dài, rồi trả kết quả về main isolate."><defs><marker id="fl41-4-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="20" y="86" class="t-b">Main isolate</text><text x="20" y="108" class="t-small">dựng khung</text>
<rect x="200" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="260" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="320" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="380" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="440" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="500" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="560" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="620" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="680" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="740" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/><rect x="800" y="62" width="40" height="36" rx="5" class="f-pk-soft s-pk"/>
<text x="20" y="186" class="t-b">Isolate worker</text><text x="20" y="208" class="t-small">tạm thời</text>
<rect x="270" y="160" width="370" height="56" rx="8" class="f-phys-soft s-phys"/><text x="286" y="193" class="t-b">parse JSON lớn</text>
<path d="M270 100 L270 158" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-4-a)"/><text x="150" y="134" class="t-mono">Isolate.run</text>
<path d="M640 158 L640 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl41-4-a)"/><text x="652" y="134" class="t-small">kết quả (Isolate.exit)</text>
<text x="20" y="242" class="t-small">Trong lúc worker parse, main isolate vẫn dựng khung.</text><text x="20" y="262" class="t-small">Gọi parseTrips trực tiếp thì khối parse chiếm chỗ các khung.</text>
</svg>
</div>
<figcaption>Hình 4. Việc nặng chạy ở isolate worker. Mũi tên đi từ main isolate sang worker lúc giao việc và từ worker về main isolate lúc trả kết quả.</figcaption>
</figure>

Lab có hai script Dart thuần, mỗi script in khoảng giá trị của nhiều lần chạy, kèm mốc "không làm gì" (nhiễu của chính `Timer`). Số dưới đây chạy trên laptop của tác giả, Dart VM JIT, khi máy đang tải nặng (load average 60 đến 97); chúng là **số minh hoạ, không phải số đo trên điện thoại**.

Khoảng cách lớn nhất giữa hai nhịp `Timer` 2 ms khi parse 50.000 chuyến (7,6 MB), 7 lần chạy: không làm gì 3 đến 14 ms, parse đồng bộ 117 đến 159 ms, `Isolate.run` 8 đến 21 ms. Parse đồng bộ chặn lâu gấp nhiều lần nhiễu nền; `Isolate.run` nằm sát vùng nhiễu, nên ta không kết luận nó đưa độ chặn về 0.

Chi phí sao chép nhìn từ main isolate (7 lần chạy, nền 5 đến 12 ms): đối số `List<Map>` một triệu phần tử làm main dừng 852 đến 2.570 ms; đối số `String` 50 MB chỉ 2 đến 7 ms. Cạm bẫy không chỉ là `this`: closure chỉ dùng `String` nhưng cùng scope với closure khác bắt `List` lớn vẫn dừng 1.025 đến 2.163 ms. Cách tránh: mỗi `Isolate.run` nằm trong hàm riêng chỉ thấy đối số của nó. Chiều ngược lại, trả một triệu `Map` bằng `Isolate.exit` không sao chép, nhưng main vẫn dừng 50 đến 91 ms; ta chưa xác định nguyên nhân.

Tài liệu Flutter nhắc thêm: isolate sinh ra không dùng được `dart:ui` hay `rootBundle`; việc nhỏ lặp lại hợp với isolate sống lâu hơn (lab chưa làm).

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi tối ưu hiệu năng</span>
<ul>
<li><strong>Đo ở debug hoặc trên simulator rồi kết luận.</strong> Dấu hiệu: đẹp trên máy dev, giật trên máy thật. Nguyên nhân: debug có assert và JIT, simulator dùng phần cứng khác. Cách sửa: <code>flutter run --profile</code> trên thiết bị thật, chọn máy yếu nhất người dùng có.</li>
<li><strong><code>setState</code> ở gốc màn hình cho phần chỉ đổi cục bộ.</strong> Dấu hiệu: bộ đếm của widget không liên quan tăng mỗi nhịp. Cách sửa: tách phần đổi thành widget tự nghe, hoặc dùng tham số <code>child</code>.</li>
<li><strong><code>select</code> trả về <code>List</code> hoặc <code>Map</code> mới.</strong> Dấu hiệu: nhãn vẫn build dù giá trị chọn không đổi. Nguyên nhân: tập hợp không có <code>==</code> theo giá trị. Cách sửa: trả giá trị có <code>==</code> (như <code>Money</code>) hoặc một record.</li>
<li><strong>Closure của <code>Isolate.run</code> bắt thừa state (<code>this</code>, hoặc biến chung scope).</strong> Dấu hiệu: <code>ArgumentError</code> với <code>unsendable</code>, hoặc main dừng vì sao chép dữ liệu lớn không dùng tới. Cách sửa: hàm top-level riêng nhận đúng đối số cần.</li>
<li><strong><code>cacheWidth</code> tính bằng pixel logic.</strong> Dấu hiệu: ảnh mờ trên màn hình có <code>devicePixelRatio</code> 3, vì giải mã 48 px rồi phóng lên 144 px vật lý. Cách sửa: nhân với <code>devicePixelRatio</code> như <code>decodeSizeFor</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/41-performance
flutter pub get
flutter analyze
flutter test
dart run tool/benchmark_isolate.dart   # minh hoạ, đổi theo máy
```

Đã kiểm với Flutter 3.47.4 / Dart 3.13.3: `flutter analyze` 0 issue, **77 test xanh**, `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`). Mức: **Kiểm chứng tự động**. Test chia bốn nhóm: đếm build và `select` (`const`, tách widget, tham số `child`, hook debug), đếm paint với và không có `RepaintBoundary`, danh sách và `ResizeImage` giải mã thật, và `Isolate.run` cùng phân loại `FrameTiming`.

**Chưa làm**: mọi số đo mili giây trên thiết bị, jank ở profile mode và thời gian raster (nên bài không có ảnh chụp hay số của DevTools), shader jank và Impeller (bài 61), khởi động và kích thước app (bài 62), isolate sống lâu, `Opacity`, `saveLayer` và clip. Muốn đo thật, sinh runner bằng lệnh trong README của lab rồi chạy `flutter run --profile` trên máy thật.

## Bài tập

1. `FareStatusLabel` trong lab dùng `select` trả record `(s.trip.fare, s.trip.status)`. Nhãn này có build lại khi tài xế di chuyển không? Khi `apply(const SearchStarted())` thì sao?
2. Đặt `scrollCacheExtent: ScrollCacheExtent.viewport(0.5)` cho `ListView.builder` trong khung nhìn 600 dp, dòng cao 72 dp. Lần dựng đầu build bao nhiêu dòng?
3. Một màn hình khác parse JSON gồm 200 bản ghi, khoảng vài chục kilobyte. Có nên bọc `Isolate.run` không?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Không build lại khi tài xế di chuyển, vì record <code>(Money, TripStatus)</code> so sánh theo giá trị và cả hai thành phần không đổi. Khi áp <code>SearchStarted</code>, trạng thái đổi nên record khác và nhãn build đúng 1 lần (hai test <code>select trả về record</code>).</p>
<p><strong>2.</strong> Vùng đệm là 300 dp, nên (600 + 300) / 72 = 12,5, làm tròn lên 13 dòng. Test <code>viewport(0.5)</code> trong lab kiểm đúng con số này.</p>
<p><strong>3.</strong> Chưa chắc. Spawn có chi phí cố định và bài không có số đo thiết bị để nêu ngưỡng. Đo ở profile mode trên máy yếu: nếu khung chứa nó còn trong ngân sách thì để nguyên, nếu vượt thì dùng <code>Isolate.run</code> rồi đo lại.</p>
</details>

Bài trước: [Platform channel, Pigeon và FFI: gọi native khi plugin không đủ](/articles/flutter-platform-channel-pigeon-ffi-goi-native). Bài tiếp: [Bảo mật mobile: secure storage, pinning, obfuscation và OWASP MASVS](/articles/flutter-bao-mat-mobile-secure-storage-pinning-obfuscation-masvs).

## Tài liệu gốc

- [Performance best practices](https://docs.flutter.dev/perf/best-practices) và [Flutter performance profiling](https://docs.flutter.dev/perf/ui-performance) (docs.flutter.dev).
- [Performance view](https://docs.flutter.dev/tools/devtools/performance) của DevTools và [Flutter's build modes](https://docs.flutter.dev/testing/build-modes) (docs.flutter.dev).
- [Concurrency and isolates](https://docs.flutter.dev/perf/isolates) (docs.flutter.dev), [Isolate.run](https://api.flutter.dev/flutter/dart-isolate/Isolate/run.html) và [compute](https://api.flutter.dev/flutter/foundation/compute.html) (api.flutter.dev).
- [FrameTiming](https://api.flutter.dev/flutter/dart-ui/FrameTiming-class.html) và [SchedulerBinding.addTimingsCallback](https://api.flutter.dev/flutter/scheduler/SchedulerBinding/addTimingsCallback.html) (api.flutter.dev).
- [RepaintBoundary](https://api.flutter.dev/flutter/widgets/RepaintBoundary-class.html), [ListView.builder](https://api.flutter.dev/flutter/widgets/ListView/ListView.builder.html) và [ScrollCacheExtent](https://api.flutter.dev/flutter/rendering/ScrollCacheExtent-class.html) (api.flutter.dev).
- [ResizeImage](https://api.flutter.dev/flutter/painting/ResizeImage-class.html) (api.flutter.dev) và [Riverpod](https://riverpod.dev) (riverpod.dev).
