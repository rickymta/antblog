---
slug: flutter-test-riverpod-async-stream-fake-async-providercontainer
title: "Kiểm thử Riverpod, async và Stream: ProviderContainer, fakeAsync, fake clock"
summary: "Bài 30: test provider Riverpod bằng ProviderContainer.test và container.listen, tắt hoặc chạy retry trên fake_async, tiêm Clock cho hạn báo giá và đếm ngược nhận cuốc, test Stream vị trí tài xế với emitsInOrder, timeout, huỷ subscription; phân biệt fakeAsync với testWidgets."
category: flutter
level: intermediate
tags: [flutter, dart, mobile, toi-noi, testing, riverpod, fake-async]
language: vi
---

App tài xế Tới Nơi nhận một cuốc mới và cho tài xế 15 giây để bấm **Nhận**. Hết giờ, cuốc chuyển cho người khác. Bên app khách, báo giá chỉ có hiệu lực 2 phút, và nếu vị trí tài xế im quá 10 giây thì bản đồ phải báo mất tín hiệu. Cả ba chuyện đều là chuyện của thời gian, và cả ba có thể sai theo kiểu rất khó thấy: nút vẫn bấm được ở giây thứ 16, báo giá hết hạn nhưng vẫn đặt chuyến được, bản đồ im lặng mà không báo gì.

Kiểm bằng tay nghĩa là ngồi đợi 15 giây, 2 phút, 10 giây, lần nào cũng vậy. Test tự động chỉ có ích khi nó không đợi thật và không lệ thuộc giờ của máy. Bài này dựng bộ công cụ đó: `ProviderContainer` để chạy provider không cần widget, `fake_async` để thời gian trôi theo lệnh, `Clock` để giờ tiêm được, và cách thử một `Stream`.

Đây là bài 30 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 18 về Riverpod](/articles/flutter-riverpod-provider-notifier-asyncnotifier-dat-chuyen), [bài 20 về retry và timeout](/articles/flutter-xu-ly-loi-retry-timeout-connectivity-mang-di-dong) và [bài 29 về golden test](/articles/flutter-golden-test-semantics-test-giao-dien-on-dinh) trước. Hai bài [27](/articles/flutter-unit-test-mocktail-fake-quy-tac-cuoc-use-case) và [28](/articles/flutter-widget-test-pumpwidget-finder-matcher-man-hinh-bao-gia) cho nền về test double và `WidgetTester`. Ví dụ lấy từ hồ sơ [Tới Nơi](/articles/du-an-mau-toi-noi); mọi thời hạn trong bài (2 phút, 15 giây, 10 giây) là số minh hoạ.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trong <code>fakeAsync</code>, ta viết <code>Future&lt;void&gt;.delayed(Duration.zero, () =&gt; ran = true)</code> rồi gọi <code>async.flushMicrotasks()</code>. Lúc này <code>ran</code> là <code>true</code> hay <code>false</code>?</p>
<details><summary>Xem đáp án</summary><p><code>false</code>. <code>Future.delayed</code> dựng một <code>Timer</code> dù độ trễ bằng 0, và <code>flushMicrotasks</code> chỉ rút hàng microtask. Phải gọi <code>async.elapse(Duration.zero)</code>. Test <code>Future(...) và Future.delayed(zero) là Timer</code> trong lab chứng minh điều này, và mục "fakeAsync: thời gian do ta quay" cho thấy vì sao nó làm hỏng cả test gọi API.</p></details>
</div>

## ProviderContainer.test: chạy provider không cần widget

Provider không phụ thuộc cây widget. `ProviderContainer` giữ state của chúng, còn `ProviderContainer.test` dựng một container và tự đăng ký `dispose` khi test kết thúc. Lab dựng sẵn một `ApiHarness`: fake API của bài 19 chạy trong tiến trình qua `InProcessAdapter`, bọc `FaultyAdapter` để tiêm lỗi mạng theo kịch bản. `overrides` thay `apiClientProvider` bằng client của harness, nên không có socket nào được mở.

### quote_provider_test.dart · 30-test-riverpod-async/test/quote_provider_test.dart

```dart
void main() {
  late ApiHarness api;
  late ProviderContainer container;

  setUp(() {
    api = ApiHarness();
    // Tắt retry: test này kiểm trạng thái, không kiểm chính sách thử lại.
    container = ProviderContainer.test(
      overrides: [api.override],
      retry: (_, _) => null,
    );
  });

  group('đọc kết quả bằng .future', () {
    test('5 km ra 25.500 VND', () async {
      final quote = await container.read(quoteProvider(ride5km).future);

      expect(quote.fare, const Money(25500));
      expect(quote.dto.distanceMeters, 5000);
    });
```

Có hai cách quan sát một provider bất đồng bộ. `await container.read(p.future)` đợi kết quả cuối: trả giá trị, hoặc ném đúng lỗi của provider (ở đây là `ClientFailure`, không bọc thêm). Cách này hợp khi ta chỉ cần đáp án. Nhưng nhiều lỗi nằm ở đường đi: spinner có hiện không, lỗi có kèm dữ liệu cũ không, state có bị báo hai lần không. Với những câu hỏi đó ta dùng `container.listen(..., fireImmediately: true)` để ghi mọi state vào một danh sách.

### support.dart · 30-test-riverpod-async/test/support.dart

```dart
List<AsyncValue<T>> record<T>(
  ProviderContainer container,
  ProviderListenable<AsyncValue<T>> provider,
) {
  final states = <AsyncValue<T>>[];
  container.listen(
    provider,
    (_, next) => states.add(next),
    fireImmediately: true,
  );
  return states;
}
```

Hàm này có hai tác dụng. `fireImmediately` ghi lại state đầu tiên, mà `listen` thường bỏ qua. Việc `listen` giữ một người nghe cũng giữ provider autoDispose sống suốt test. Chuỗi state của một báo giá thành công chỉ gồm hai phần tử:

### quote_provider_test.dart · 30-test-riverpod-async/test/quote_provider_test.dart

```dart
    test('thành công: AsyncLoading rồi AsyncData', () async {
      final states = record(container, quoteProvider(ride5km));

      await container.read(quoteProvider(ride5km).future);

      expect(states, [
        isA<AsyncLoading<ValidQuote>>(),
        isA<AsyncData<ValidQuote>>().having(
          (s) => s.value.fare,
          'fare',
          const Money(25500),
        ),
      ]);
    });
```

Ta so từng phần tử bằng matcher `isA` thay vì `==`, vì `AsyncError` còn mang dấu vết ngăn xếp mà test không cần. Ca thất bại cùng cấu trúc: `AsyncLoading` rồi `AsyncError` mang `ClientFailure`.

Còn chuyện autoDispose. `container.read(p)` thực chất là `listen` rồi đóng người nghe ngay, nên khi không ai giữ, provider bị bỏ và lần đọc sau tính lại từ đầu. Hai lần đọc liên tiếp là hai request; giữ một `listen` thì chỉ một.

### quote_provider_test.dart · 30-test-riverpod-async/test/quote_provider_test.dart

```dart
    test('không ai nghe: mỗi lần đọc là một request mới', () async {
      await container.read(quoteProvider(ride5km).future);
      await container.read(quoteProvider(ride5km).future);

      expect(api.serverRequests, 2);
    });

    test('có listener: hai lần đọc dùng chung một request', () async {
      final subscription = container.listen(quoteProvider(ride5km), (_, _) {});

      await container.read(quoteProvider(ride5km).future);
      await container.read(quoteProvider(ride5km).future);

      expect(api.serverRequests, 1);
      subscription.close();
    });
```

Sau khi đóng người nghe cuối, việc bỏ provider chưa xảy ra ngay; test đợi nó bằng `await container.pump()`.

## Retry trong test: tắt, hoặc chạy trên đồng hồ giả

Bài 18 đã nói Riverpod 3 tự thử lại provider ném `Exception`, tối đa 10 lần, và bài 20 chỉ ra hệ quả khi nhân với retry của interceptor. `ApiFailure` là một `Exception`, nên cả lỗi 422 cũng bị gọi lại. Test cho thấy con số cụ thể: một yêu cầu 150 km gây ra 11 request thay vì một.

Lab chọn chính sách riêng, truyền vào container (hoặc `ProviderScope(retry: ...)` ở app): chỉ thử lại lỗi mà `ApiFailure.retryable` cho là tạm thời, tối đa 3 lần.

### retry_policy.dart · 30-test-riverpod-async/lib/retry_policy.dart

```dart
Duration? retryTransient(int retryCount, Object error) {
  if (error is! ApiFailure || !error.retryable || retryCount >= 3) return null;
  return Duration(milliseconds: 500 * (1 << retryCount));
}
```

Từ đó có hai kiểu test. Test không quan tâm retry thì tắt bằng `retry: (_, _) => null`, như `setUp` ở mục trước; lỗi tới ngay, không cần đồng hồ giả. Test kiểm chính chính sách thì giữ `retry: retryTransient` và để `fakeAsync` đẩy thời gian (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Ba lần gọi báo giá trên đồng hồ giả. Lần 1 và lần 2 bị từ chối kết nối, mỗi lần listener nhận AsyncLoading mang lỗi cũ. Lần 3 lúc 1.500 ms thành công và listener nhận AsyncData."><defs><marker id="fl30-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl30-1-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<text x="135" y="24" class="t-b">t = 0</text><text x="320" y="24" class="t-b">ngay sau lần 1</text><text x="505" y="24" class="t-b">t = 500 ms</text><text x="690" y="24" class="t-b">t = 1.500 ms</text>
<path d="M135 36 L858 36" class="f-none s-muted" stroke-width="2" marker-end="url(#fl30-1-m)"/><text x="745" y="52" class="t-small">thời gian giả</text>
<text x="12" y="104" class="t-b">Adapter</text><text x="12" y="122" class="t-small">fake API</text>
<rect x="135" y="68" width="170" height="64" rx="10" class="f-phys-soft s-phys"/><text x="149" y="94" class="t-b">gọi lần 1</text><text x="149" y="116" class="t-small">đang chạy</text>
<rect x="320" y="68" width="170" height="64" rx="10" class="f-warn-soft s-warn"/><text x="334" y="94" class="t-b">lần 1: refuse</text><text x="334" y="116" class="t-small">chờ 500 ms</text>
<rect x="505" y="68" width="170" height="64" rx="10" class="f-warn-soft s-warn"/><text x="519" y="94" class="t-b">lần 2: refuse</text><text x="519" y="116" class="t-small">chờ 1.000 ms</text>
<rect x="690" y="68" width="170" height="64" rx="10" class="f-pk-soft s-pk"/><text x="704" y="94" class="t-b">lần 3: 200 OK</text><text x="704" y="116" class="t-small">fare 25.500 VND</text>
<path d="M220 134 L220 186" class="f-none s-pk" stroke-width="2" marker-end="url(#fl30-1-a)"/>
<path d="M405 134 L405 186" class="f-none s-pk" stroke-width="2" marker-end="url(#fl30-1-a)"/>
<path d="M590 134 L590 186" class="f-none s-pk" stroke-width="2" marker-end="url(#fl30-1-a)"/>
<path d="M775 134 L775 186" class="f-none s-pk" stroke-width="2" marker-end="url(#fl30-1-a)"/>
<text x="12" y="226" class="t-b">State trong</text><text x="12" y="244" class="t-b">listener</text>
<rect x="135" y="190" width="170" height="72" rx="10" class="f-phys-soft s-phys"/><text x="149" y="216" class="t-b">AsyncLoading</text><text x="149" y="238" class="t-small">chưa có giá trị</text>
<rect x="320" y="190" width="170" height="72" rx="10" class="f-warn-soft s-warn"/><text x="334" y="216" class="t-b">AsyncLoading</text><text x="334" y="238" class="t-small">retrying, lỗi cũ</text>
<rect x="505" y="190" width="170" height="72" rx="10" class="f-warn-soft s-warn"/><text x="519" y="216" class="t-b">AsyncLoading</text><text x="519" y="238" class="t-small">retrying, lỗi cũ</text>
<rect x="690" y="190" width="170" height="72" rx="10" class="f-pk-soft s-pk"/><text x="704" y="216" class="t-b">AsyncData</text><text x="704" y="238" class="t-small">retrying = false</text>
<text x="135" y="300" class="t-mono">states = [Loading, Loading(retrying), Loading(retrying), Data]</text>
<text x="135" y="324" class="t-small">Cả chuỗi chạy trong vài mili giây thật: 1,5 giây ở trên là giờ của fakeAsync.</text>
</svg>
</div>
<figcaption>Hình 1. Chuỗi state mà <code>container.listen</code> ghi lại khi hai lần gọi đầu bị từ chối kết nối. Mỗi lần thử lại, state vẫn là <code>AsyncLoading</code> nhưng giữ lỗi cũ.</figcaption>
</figure>

### retry_test.dart · 30-test-riverpod-async/test/retry_test.dart (trích)

```dart
void settle(FakeAsync async) => async.elapse(Duration.zero);
```

```dart
    test('hai lần mất kết nối rồi thành công: 3 lần gọi, 1,5 giây giả', () {
      fakeAsync((async) {
        final api = ApiHarness(
          faults: [NetworkFault.refuse, NetworkFault.refuse],
        );
        final container = ProviderContainer.test(
          overrides: [api.override],
          retry: retryTransient,
        );
        final states = record(container, quoteProvider(ride5km));

        settle(async);
        expect(api.attempts, 1);
        expect(states.last.isLoading, isTrue);

        async.elapse(const Duration(milliseconds: 499));
        expect(api.attempts, 1); // chưa tới 500 ms
        async.elapse(const Duration(milliseconds: 1));
        expect(api.attempts, 2);

        async.elapse(const Duration(milliseconds: 999));
        expect(api.attempts, 2); // chưa tới 500 + 1000 ms
        async.elapse(const Duration(milliseconds: 1));
        expect(api.attempts, 3);

        expect(states.last, isA<AsyncData<ValidQuote>>());
```

Test kiểm cả hai phía của mỗi mốc: 499 ms chưa có lần gọi mới, 500 ms thì có. Đổi lịch 500, 1000, 2000 ms là một dòng đỏ. Các ca còn lại kiểm hết lượt (4 lần gọi rồi `AsyncError`) và 422 (một request, lỗi ngay).

## fakeAsync: thời gian do ta quay

`fakeAsync((async) { ... })` chạy thân hàm trong một zone mà `Timer` và `scheduleMicrotask` bị thay thế. Chúng không chạy theo giờ thật, chỉ chạy khi ta ra lệnh. Có hai hàng đợi riêng (Hình 2), và phần lớn lỗi của người mới nằm ở chỗ nhầm hàng nào do lệnh nào rút.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. Hai hàng đợi của fakeAsync. flushMicrotasks chỉ rút hàng microtask. elapse rút microtask rồi bắn các Timer đến hạn. flushTimers bắn Timer cho tới khi hết. I/O thật và Timer tạo trước fakeAsync nằm ngoài cả hai hàng đợi."><defs><marker id="fl30-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl30-2-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="360" height="120" rx="10" class="f-phys-soft s-phys"/><text x="36" y="58" class="t-b">Hàng đợi microtask</text><text x="36" y="84" class="t-small">tiếp tục sau await, then() đã sẵn sàng</text><text x="36" y="108" class="t-mono">scheduleMicrotask</text>
<rect x="420" y="30" width="440" height="120" rx="10" class="f-pk-soft s-pk"/><text x="436" y="58" class="t-b">Hàng đợi Timer, xếp theo giờ đến hạn</text><text x="436" y="84" class="t-mono">Future(...), Future.delayed(d)</text><text x="436" y="106" class="t-mono">Timer, Timer.periodic, Stream.periodic</text><text x="436" y="130" class="t-small">dio chạy chuỗi interceptor bằng Future(...)</text>
<rect x="20" y="215" width="220" height="64" rx="10" class="f-surface s-line"/><text x="34" y="241" class="t-mono">async.flushMicrotasks()</text><text x="34" y="263" class="t-small">chỉ rút hàng này</text>
<rect x="260" y="215" width="360" height="64" rx="10" class="f-surface s-line"/><text x="274" y="238" class="t-mono">async.elapse(d)</text><text x="274" y="256" class="t-small">microtask trước, rồi Timer đến hạn;</text><text x="274" y="272" class="t-small">xen microtask sau mỗi lần bắn</text>
<rect x="640" y="215" width="220" height="64" rx="10" class="f-surface s-line"/><text x="654" y="241" class="t-mono">async.flushTimers()</text><text x="654" y="263" class="t-small">như elapse, tới khi hết Timer</text>
<path d="M130 213 L130 152" class="f-none s-phys" stroke-width="2" marker-end="url(#fl30-2-a)"/>
<path d="M340 213 L340 152" class="f-none s-phys" stroke-width="2" marker-end="url(#fl30-2-a)"/>
<path d="M560 213 L560 152" class="f-none s-pk" stroke-width="2" marker-end="url(#fl30-2-p)"/>
<path d="M750 213 L750 152" class="f-none s-pk" stroke-width="2" marker-end="url(#fl30-2-p)"/>
<rect x="20" y="310" width="840" height="58" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="336" class="t-b t-warn">Ngoài vùng của fakeAsync</text><text x="36" y="356" class="t-small">I/O thật (File, socket) và Timer tạo trước khi vào fakeAsync: elapse không chạm tới.</text>
</svg>
</div>
<figcaption>Hình 2. Hai hàng đợi của <code>fakeAsync</code> và lệnh rút chúng. Mũi tên chỉ từ lệnh tới hàng nó xử lý; hộp nét đứt là những thứ <code>fakeAsync</code> không điều khiển được.</figcaption>
</figure>

`elapse(d)` rút microtask, rồi bắn lần lượt các `Timer` đến hạn trong khoảng `d` theo thứ tự thời gian, rút microtask sau mỗi lần bắn. Đồng hồ giả nhảy tới đúng thời điểm của từng Timer. Bốn cạm bẫy dưới đây đều có test riêng trong lab.

**Một: `Future(...)` và `Future.delayed(Duration.zero)` là Timer.** Chỉ `Future.value().then` và phần tiếp theo của `await` mới là microtask.

### fake_async_pitfalls_test.dart · 30-test-riverpod-async/test/fake_async_pitfalls_test.dart

```dart
        final ran = <String>[];
        Future<void>(() => ran.add('Future'));
        Future<void>.delayed(Duration.zero, () => ran.add('delayed'));

        async.flushMicrotasks();
        expect(ran, isEmpty);

        async.elapse(Duration.zero);
        expect(ran, ['Future', 'delayed']);
```

Đây cũng là lý do mọi test gọi API bằng `fakeAsync` ở mục trước phải có `settle`, kể cả khi ta không tự thêm interceptor nào. Chuỗi interceptor của dio chạy trong `Future(() async {...})` (xem `dio_mixin.dart` của `dio` 5.11.1), tức qua một Timer. Sau `flushMicrotasks`, request vẫn chưa tới adapter; test `request của dio cần elapse(zero)` đếm `api.attempts` bằng 0 rồi 1 để chứng minh.

**Hai: chỉ Timer tạo trong zone mới bị điều khiển.** Timer dựng trước khi vào `fakeAsync` (ở `setUpAll`, trong một biến toàn cục) chạy theo giờ thật, nên một giờ giả không làm nó nổ.

### fake_async_pitfalls_test.dart · 30-test-riverpod-async/test/fake_async_pitfalls_test.dart

```dart
      var fired = false;
      final real = Timer(const Duration(milliseconds: 1), () => fired = true);

      fakeAsync((async) {
        async.elapse(const Duration(hours: 1));
      });

      expect(fired, isFalse); // một giờ giả không làm Timer thật chạy
```

**Ba: I/O thật không chạy.** Kết quả của `File.exists()` hay socket tới từ vòng lặp sự kiện thật, mà thân `fakeAsync` là hàm đồng bộ, chưa nhường cho vòng lặp đó lần nào. Test `I/O thật (đọc file)` cho thấy `done` vẫn `false` sau một giờ giả. Muốn dùng I/O thật trong test widget, bọc nó vào `tester.runAsync`; trong test thuần thì đừng đặt nó trong `fakeAsync`, mà thay bằng fake như mọi bài trước.

**Bốn: `fakeAsync` không phàn nàn Timer còn sót.** Test vẫn xanh dù `Timer.periodic` chưa huỷ; ta phải tự kiểm `async.pendingTimers`. Và `async.flushTimers()` với một periodic chưa huỷ sẽ không bao giờ hết việc: nó ném `StateError` sau 1 giờ giả (mặc định của tham số `timeout`).

**Khác `testWidgets` ở đâu?** `testWidgets` đã chạy thân test trong một `FakeAsync` do binding tạo, nên ta không gói thêm `fakeAsync`; `tester.pump(d)` với `d` khác 0 đóng vai `async.elapse(d)`. `pump()` không đối số chỉ rút microtask và vẽ frame nếu có, giống `flushMicrotasks`, nên Timer 0 giây chỉ chạy sau `pump(Duration.zero)`. Khác nữa: cuối mỗi `testWidgets`, binding khẳng định không còn Timer nào ("A Timer is still pending even after the widget tree was disposed"), còn `fakeAsync` trần thì im lặng. Test provider thuần dùng `fakeAsync`; test có widget dùng `testWidgets`; cả hai chung nguyên lý hai hàng đợi.

## Đồng hồ tiêm được với package clock

`DateTime.now()` không thay được: `fakeAsync` bẻ Timer, nhưng không bẻ được hàm đọc giờ hệ điều hành. Code cần hỏi giờ qua một cửa mà test thay được. Package `clock` cho cửa đó: getter toàn cục `clock` trả `Clock` của zone hiện tại, và `withClock(c, () => ...)` chạy một đoạn code với đồng hồ khác. Báo giá của lab đóng dấu hạn bằng `clock.now()`:

### quote_providers.dart · 30-test-riverpod-async/lib/quote_providers.dart (trích)

```dart
  /// Hết hạn khi đã tới đúng [expiresAt] (không còn "trước" nó).
  bool isExpiredAt(DateTime now) => !now.isBefore(expiresAt);

  /// Theo đồng hồ hiện hành của zone (xem `clock` trong package `clock`).
  bool get isExpired => isExpiredAt(clock.now());
```

```dart
    return ValidQuote(dto: dto, expiresAt: clock.now().add(quoteValidity));
```

Đúng hạn tính là hết hạn: test `ca biên` kiểm cả `hạn - 1 µs` (còn hiệu lực) lẫn đúng hạn (hết). Với `withClock(Clock.fixed(fixedNow), ...)`, hạn của báo giá là `03:02:00Z` chính xác. `clock` nằm trong zone: các `await` tiếp theo trong zone đó vẫn thấy đồng hồ ấy, còn ra khỏi `withClock` là về giờ máy (cả hai có test). Vì vậy provider phải build lần đầu bên trong zone đó: nếu nó đã build ngoài `withClock`, `expiresAt` lấy giờ máy mà không báo lỗi gì.

`fakeAsync` tự đặt luôn `clock`: bên trong, `clock.now()` bắt đầu ở `initialTime` (mặc định là `clock.now()` lúc tạo) rồi đi theo `elapse`. Vì thế một test hết hạn không cần `withClock`:

### clock_test.dart · 30-test-riverpod-async/test/clock_test.dart (trích)

```dart
        final quote = container.read(quoteProvider(ride5km)).requireValue;
        expect(quote.expiresAt, fixedNow.add(quoteValidity));
        expect(quote.isExpired, isFalse);

        async.elapse(quoteValidity - const Duration(seconds: 1));
        expect(quote.isExpired, isFalse);

        async.elapse(const Duration(seconds: 1));
        expect(quote.isExpired, isTrue);
      }, initialTime: fixedNow);
```

Bảng dưới tóm tắt ai quyết định hai cách đọc giờ ở từng môi trường:

| Môi trường | `clock.now()` | `DateTime.now()` |
|---|---|---|
| `test` thường | giờ máy | giờ máy |
| `withClock(Clock.fixed(t), ...)` | `t`, đứng yên | giờ máy |
| `fakeAsync(initialTime: t)` | `t` cộng thời gian đã `elapse` | giờ máy |
| `testWidgets` | giờ máy lúc test bắt đầu, cộng thời gian đã `pump` | giờ máy |

Trong `testWidgets`, ta dùng `clock`, không dùng `tester.binding.clock`: đó là một đồng hồ giả khác, bắt đầu từ năm 2015 (có test kiểm). Đếm ngược của lab tận dụng mọi thứ trên. Nó tính phần còn lại từ hạn, không đếm số lần tick:

### countdown.dart · 30-test-riverpod-async/lib/countdown.dart (trích)

```dart
  void emit() {
    final remaining = deadline.difference(clock.now());
    if (remaining <= Duration.zero) {
      timer?.cancel();
      controller.add(Duration.zero);
      controller.close();
    } else {
      controller.add(remaining);
    }
  }
```

Test dùng `_collect` (nghe stream, gom giá trị vào danh sách) để dựng ca "máy ngủ": đồng hồ nhảy 7 giây trong lúc Timer chưa kịp chạy.

### countdown_test.dart · 30-test-riverpod-async/test/countdown_test.dart (trích)

```dart
        async.elapseBlocking(const Duration(seconds: 7)); // Timer chưa chạy
        async.elapse(_second);

        // Đồng hồ đã ở giây thứ 8 nên còn 2 giây; một bộ đếm số lần tick
        // thì không hề biết 7 giây đã trôi.
        expect(result.values.last, const Duration(seconds: 2));
```

Cũng ở file này có ca cảnh báo: bọc đếm ngược trong `withClock(Clock.fixed(...))` bên trong `fakeAsync` thì Timer vẫn tick 10 lần nhưng `clock` đứng yên, nên màn hình mãi còn 3 giây. Hai nguồn thời gian trong một test phải khớp; muốn đồng hồ đi thì để `fakeAsync` tự đặt.

## Test Stream: thứ tự, timeout và huỷ subscription

Vị trí tài xế là một `Stream`. Provider của lab thêm `timeout` và tắt retry:

### driver_position.dart · 30-test-riverpod-async/lib/driver_position.dart (trích)

```dart
final driverPositionProvider = StreamProvider.autoDispose
    .family<DriverPosition, String>(
      (ref, driverId) => ref
          .watch(driverLocationSourceProvider)
          .watch(driverId)
          .timeout(positionStaleAfter),
      retry: (_, _) => null,
    );
```

`Stream.timeout` không đóng stream khi quá hạn: không có `onTimeout`, nó thả một `TimeoutException` vào kênh lỗi, và theo tài liệu thì giữa hai sự kiện nó chỉ báo tối đa một lần. Riverpod biến sự kiện lỗi đó thành `AsyncError` vẫn giữ vị trí cũ; vị trí kế tiếp đưa state về `AsyncData`. Retry phải tắt: `TimeoutException` là `Exception`, nên mặc định Riverpod sẽ nghe lại nguồn từ đầu sau 200 ms. Một test riêng chứng minh điều đó (`source.watched` từ 1 lên 2); với nguồn chỉ cho một người nghe, nó còn làm test vỡ vì "Stream has already been listened to".

Nguồn giả là một `StreamController`; test điều khiển bằng `emit` và `fail`, còn `cancels` đếm số lần provider huỷ subscription.

### support.dart · 30-test-riverpod-async/test/support.dart (trích)

```dart
  FakeLocationSource({bool broadcast = false})
    : controller = broadcast
          ? StreamController<DriverPosition>.broadcast()
          : StreamController<DriverPosition>() {
    controller.onCancel = () => cancels++;
  }

  bool get hasListener => controller.hasListener;

  void emit(DriverPosition value) => controller.add(value);

  void fail(Object error) => controller.addError(error);
```

Để kiểm thứ tự, ta đổi chuỗi state thành `Stream` bằng `stateStream` rồi dùng `emitsInOrder`. Ta gọi `expectLater` trước khi `emit` rồi `await` kết quả ở cuối. Với stream broadcast thứ tự này là bắt buộc, vì sự kiện phát trước khi có người nghe sẽ mất; ở đây nguồn có đệm nên đảo thứ tự vẫn xanh, và đây chỉ là thói quen an toàn:

### driver_position_test.dart · 30-test-riverpod-async/test/driver_position_test.dart (trích)

```dart
      final states = stateStream(newContainer(), driverPositionProvider('D1'));

      final expectation = expectLater(
        states,
        emitsInOrder([
          isA<AsyncLoading<DriverPosition>>(),
          _data(position(10.77)),
          _data(position(10.78)),
        ]),
      );
      source.emit(position(10.77));
      source.emit(position(10.78));

      await expectation;
```

Timeout thì không cần chờ 10 giây: `fakeAsync` đẩy thời gian, và test kiểm cả hai phía của biên.

### driver_position_test.dart · 30-test-riverpod-async/test/driver_position_test.dart (trích)

```dart
        async.elapse(positionStaleAfter - const Duration(milliseconds: 1));
        expect(states.last, _data(position(10.77)));

        async.elapse(const Duration(milliseconds: 1));
        expect(
          states.last,
          isA<AsyncError<DriverPosition>>()
              .having((s) => s.error, 'error', isA<TimeoutException>())
              .having((s) => s.value, 'vị trí cũ', position(10.77)),
        );
```

Cuối cùng là huỷ subscription, chỗ rò rỉ hay gặp nhất. Khi container bị dispose, nguồn phải hết người nghe và `Timer` của `timeout` phải biến mất:

### driver_position_test.dart · 30-test-riverpod-async/test/driver_position_test.dart (trích)

```dart
        container.dispose();

        expect(source.hasListener, isFalse);
        expect(source.cancels, 1);
        expect(async.pendingTimers, isEmpty);
```

Cùng nhóm còn ca đóng người nghe cuối của một provider autoDispose, và ca còn một người nghe thì chưa được huỷ.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi test async</span>
<ul>
<li><strong>State mãi là <code>AsyncLoading</code> trong <code>fakeAsync</code>.</strong> Dấu hiệu: <code>expect</code> đầu tiên sau <code>listen</code> đỏ, số lần gọi API bằng 0. Nguyên nhân: chỉ gọi <code>flushMicrotasks</code> trong khi chuỗi gọi có <code>Future(...)</code> hay <code>Future.delayed</code>. Cách sửa: <code>async.elapse(Duration.zero)</code>.</li>
<li><strong>Provider autoDispose tính lại giữa các lần đọc.</strong> Dấu hiệu: số request gấp đôi dự kiến, state mất. Nguyên nhân: không ai giữ <code>listen</code>, nên <code>read</code> mở rồi đóng. Cách sửa: giữ một <code>container.listen</code> suốt test, hoặc dùng <code>record</code>.</li>
<li><strong>Retry mặc định làm số lần gọi lệch hoặc test chạy lâu.</strong> Dấu hiệu: 11 request cho một lỗi 422, hoặc "Stream has already been listened to" với nguồn một người nghe. Nguyên nhân: <code>Exception</code> nào cũng được thử lại. Cách sửa: <code>ProviderContainer.test(retry: (_, _) =&gt; null)</code>, hoặc đặt <code>retry</code> riêng cho provider.</li>
<li><strong>Đếm ngược đứng yên hoặc hết hạn sai.</strong> Dấu hiệu: giá trị không giảm dù <code>elapse</code> nhiều giây. Nguyên nhân: code gọi <code>DateTime.now()</code>, hoặc test dựng <code>Clock.fixed</code> trong <code>fakeAsync</code>. Cách sửa: hỏi giờ qua <code>clock.now()</code> và để <code>fakeAsync</code> đặt đồng hồ.</li>
<li><strong>Timer sót.</strong> Dấu hiệu: <code>testWidgets</code> báo "A Timer is still pending even after the widget tree was disposed", hoặc <code>flushTimers</code> ném <code>StateError</code>. Nguyên nhân: <code>Timer.periodic</code> không bị huỷ khi provider bị bỏ. Cách sửa: huỷ trong <code>onCancel</code> hoặc <code>ref.onDispose</code>, và kiểm <code>async.pendingTimers</code> sau <code>dispose</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/30-test-riverpod-async
flutter pub get
flutter analyze
flutter test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3, `flutter_riverpod` 3.5.0 (ràng buộc `^3.4.3`), `fake_async` 1.3.3, `clock` 1.1.3: `flutter analyze` báo 0 issue, `flutter test` chạy 68 test xanh trong vài giây. Mức: **Kiểm chứng tự động**; không test nào chờ thời gian thật hay gọi mạng.

Nhóm báo giá (18 test) chứng minh chuỗi state, `read(p.future)`, autoDispose cần người nghe và ba kiểu retry. Nhóm đồng hồ và đếm ngược (22 test) chứng minh `withClock`, biên hết hạn, `fakeAsync(initialTime:)`, ca máy ngủ và việc huỷ Timer. Nhóm vị trí tài xế (12 test) chứng minh thứ tự, biên timeout, retry mặc định và huỷ subscription. Nhóm cạm bẫy (9 test) và nhóm `testWidgets` (7 test) chứng minh hai mục "fakeAsync: thời gian do ta quay" và "Đồng hồ tiêm được với package clock".

**Chưa làm**: nguồn vị trí thật bằng WebSocket có reconnect (bài 36); test tích hợp trên thiết bị (bài tiếp theo); `ProviderObserver` ghi log; `riverpod_generator`. Lab chỉ có logic và test, không có `main.dart` hay thư mục `android/`, `ios/`.

## Bài tập

1. Test gọi `async.flushMicrotasks()` rồi `expect(container.read(quoteProvider(ride5km)).hasValue, isTrue)` bị đỏ. Vì sao, và sửa thế nào?
2. Đổi `remainingUntil` thành bộ đếm giảm `tick` mỗi lần phát thay vì so với `clock.now()`. Theo bạn test nào trong `countdown_test.dart` đỏ, và điều đó nói gì về hai cách cài đặt?
3. Tài xế gửi một vị trí rồi im lặng 25 giây. Listener của `driverPositionProvider` nhận bao nhiêu `AsyncError`? Viết test để kiểm.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Request của dio đi qua <code>Future(...)</code>, là Timer, nên <code>flushMicrotasks</code> không đưa nó tới adapter và state còn <code>AsyncLoading</code>. Thay bằng <code>async.elapse(Duration.zero)</code> (hàm <code>settle</code> của lab).</p>
<p><strong>2.</strong> Hai test đỏ: <code>máy ngủ 7 giây</code>, vì bộ đếm không biết 7 giây đã trôi, và <code>Clock.fixed bên trong fakeAsync</code>, vì bộ đếm vẫn giảm dù <code>clock</code> đứng yên. Tính từ hạn đúng miễn <code>clock</code> đúng; đếm tick chỉ đúng khi mọi tick đến đúng giờ.</p>
<p><strong>3.</strong> Một. <code>Stream.timeout</code> chỉ báo tối đa một lần giữa hai sự kiện. Test: <code>fakeAsync</code>, <code>record</code> provider, <code>emit</code> một vị trí, <code>async.elapse(const Duration(seconds: 25))</code>, rồi <code>expect(states.where((s) =&gt; s.hasError), hasLength(1))</code>; lab có test <code>im lặng suốt 25 giây chỉ báo một lỗi</code>.</p>
</details>

Bài trước: [Golden test và Semantics test: khoá giao diện và accessibility](/articles/flutter-golden-test-semantics-test-giao-dien-on-dinh). Bài tiếp: [Integration test và patrol: luồng đặt chuyến trên simulator/emulator](/articles/flutter-integration-test-patrol-luong-dat-chuyen-tren-thiet-bi).

## Tài liệu gốc

- [Testing Flutter apps](https://docs.flutter.dev/testing/overview) và [An introduction to widget testing](https://docs.flutter.dev/cookbook/testing/widget/introduction) (docs.flutter.dev).
- [Testing your providers](https://riverpod.dev/docs/how_to/testing) (riverpod.dev) và [ProviderContainer.test](https://pub.dev/documentation/riverpod/latest/riverpod/ProviderContainer/ProviderContainer.test.html) (pub.dev).
- [package:fake_async](https://pub.dev/packages/fake_async) và [package:clock](https://pub.dev/packages/clock) (pub.dev).
- [Stream.timeout](https://api.dart.dev/stable/dart-async/Stream/timeout.html) và [Zone](https://api.dart.dev/stable/dart-async/Zone-class.html) (api.dart.dev).
- [Concurrency in Dart](https://dart.dev/language/concurrency) và [Asynchronous programming: Streams](https://dart.dev/libraries/async/using-streams) (dart.dev).
- [WidgetTester.runAsync](https://api.flutter.dev/flutter/flutter_test/WidgetTester/runAsync.html) và [TestWidgetsFlutterBinding.clock](https://api.flutter.dev/flutter/flutter_test/TestWidgetsFlutterBinding/clock.html) (api.flutter.dev).
