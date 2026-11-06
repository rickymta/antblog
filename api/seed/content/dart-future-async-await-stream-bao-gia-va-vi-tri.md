---
slug: dart-future-async-await-stream-bao-gia-va-vi-tri
title: "Future, async/await và Stream: gọi báo giá và lắng nghe vị trí tài xế"
summary: "Bài 05: hiểu event loop một luồng của Dart, Future và async/await khi gọi dịch vụ báo giá, timeout và lỗi; Stream, StreamController và await for khi theo dõi vị trí tài xế, huỷ subscription đúng lúc; Isolate.run cho việc nặng. Test dùng fake_async nên không chờ thời gian thật."
category: flutter
level: beginner
tags: [flutter, dart, mobile, toi-noi, async, stream]
language: vi
---

Khách mở màn hình đặt xe, nhập điểm đến rồi bấm **Xem giá**. App gọi dịch vụ báo giá, mất vài trăm mili giây. Cùng lúc đó, chấm xe tài xế vẫn phải nhích trên bản đồ và ngón tay khách vẫn cuộn được màn hình. Ba việc diễn ra "cùng lúc", trong khi Dart chạy mã của bạn trên đúng một luồng.

Bài này giải thích vì sao điều đó không mâu thuẫn: event loop, rồi `Future` và `async/await` cho lời gọi báo giá, rồi `Stream` cho vị trí tài xế, và `Isolate.run` khi có việc tính toán nặng thật sự. Đây là bài 05 của lộ trình **Flutter qua Tới Nơi**, nối tiếp [bài 04 về collections, generics và extension](/articles/dart-collections-generics-extension-danh-sach-chuyen). Ví dụ lấy từ dự án mẫu hư cấu [Tới Nơi](/articles/du-an-mau-toi-noi); độ trễ 300 ms và bảng giá là số minh hoạ. Nếu đã đọc bài C# [Task, async/await và CancellationToken](/articles/csharp-async-await-cancellation), ta sẽ chỉ ra chỗ Dart khác hẳn.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một hàm <code>async</code> chứa vòng <code>for</code> chạy 600 triệu lần, không có <code>await</code> nào bên trong. Hàm này có làm màn hình app Flutter giật không? Và <code>await</code> một lời gọi mạng dài 3 giây thì sao?</p>
<details><summary>Xem đáp án</summary><p>Vòng <code>for</code> làm màn hình đứng hình: từ khoá <code>async</code> không tạo luồng mới, mã chạy trên cùng luồng vẽ giao diện cho tới khi nó nhường. Còn <code>await</code> lời gọi mạng thì không chặn gì: trong lúc chờ, luồng vẫn xử lý chạm và vẽ khung hình.</p></details>
</div>

## Một luồng, một event loop

Mã Dart chạy trong một **isolate**: một vùng nhớ (heap) riêng, một luồng thực thi và một **event loop**. App Flutter có một isolate chính (main isolate) vừa chạy mã của ta vừa xử lý chạm và vẽ khung hình, nên mọi đoạn mã tranh nhau một luồng.

Event loop giữ **hai hàng đợi**:

- **Microtask queue**: việc rất ngắn phải xong ngay sau đoạn mã hiện tại, như `Future.microtask`, `scheduleMicrotask` và, về mặt khái niệm, phần mã tiếp sau `await` khi Future đã xong.
- **Event queue**: việc đến từ bên ngoài hoặc từ đồng hồ, như timer, I/O xong, chạm, `Future(() {})`, `Timer.run`.

Quy tắc: làm xong một việc, loop **rút cạn microtask queue**, chỉ khi nó rỗng mới lấy **một** việc từ event queue, rồi lặp lại (Hình 1).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 1. Main isolate lấy việc từ hai hàng đợi: microtask queue được rút cạn trước, sau đó mới lấy một việc từ event queue. Vòng lặp nặng giữ luồng làm cả hai hàng đợi phải chờ."><defs><marker id="fl05-0-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="20" y="40" width="230" height="170" rx="10" class="f-phys-soft s-phys"/><text x="36" y="76" class="t-b">Main isolate</text><text x="36" y="104" class="t-small">Một luồng, chạy từng việc</text><text x="36" y="128" class="t-small">tới hết rồi mới lấy việc kế</text><text x="36" y="168" class="t-mono">sync → microtask → event</text>
<rect x="320" y="20" width="540" height="90" rx="10" class="f-pk-soft s-pk"/><text x="336" y="50" class="t-b">1. Microtask queue (rút cạn trước)</text><text x="336" y="78" class="t-small">tiếp sau await (Future xong) · Future.microtask · scheduleMicrotask</text>
<rect x="320" y="140" width="540" height="90" rx="10" class="f-phys-soft s-phys"/><text x="336" y="170" class="t-b">2. Event queue (mỗi lượt lấy một việc)</text><text x="336" y="198" class="t-small">Timer · Future(() {}) · I/O xong · chạm · khung hình</text>
<path d="M320 65 L252 95" class="f-none s-pk" stroke-width="2" marker-end="url(#fl05-0-a)"/>
<path d="M320 185 L252 160" class="f-none s-phys" stroke-width="2" marker-end="url(#fl05-0-a)"/>
<rect x="20" y="260" width="840" height="60" rx="10" class="f-warn-soft s-warn"/><text x="36" y="296" class="t-b">Vòng lặp nặng không có await: cả hai hàng đợi đứng chờ, chạm và vẽ cũng vậy</text>
</svg>
</div>
<figcaption>Hình 1. Event loop của một isolate. Microtask luôn được rút cạn trước khi lấy một việc từ event queue; việc nặng giữ luồng sẽ chặn cả hai.</figcaption>
</figure>

Hàm dưới đây ghi lại thứ tự thực tế. Dòng `log.add('sync')` nằm gần cuối nhưng chạy **đầu tiên**.

### event_loop_demo.dart · 05-async-stream/lib/event_loop_demo.dart

```dart
import 'dart:async';

/// Ghi lại thứ tự chạy: mã đồng bộ, rồi microtask, rồi event.
Future<List<String>> traceEventLoop() {
  final log = <String>[];
  final done = Completer<List<String>>();

  Future<void>(() {
    log.add('event');
    done.complete(log);
  }); // vào event queue
  Future.microtask(() => log.add('microtask')); // vào microtask queue
  log.add('sync'); // chạy ngay

  return done.future;
}

/// Thứ tự đầy đủ của các cách xếp việc: mã đồng bộ, rồi microtask
/// (theo thứ tự đăng ký), rồi các event (theo thứ tự đăng ký).
Future<List<String>> traceQueues() {
  final log = <String>[];
  final done = Completer<List<String>>();

  Timer.run(() => log.add('Timer.run')); // event, đăng ký trước
  Future<void>(() {
    log.add('Future'); // event, đăng ký sau
    done.complete(log);
  });
  scheduleMicrotask(() => log.add('scheduleMicrotask'));
  Future.microtask(() => log.add('Future.microtask'));
  log.add('sync');

  return done.future;
}

/// Một hàm async chạy đồng bộ cho tới await đầu tiên; phần sau await
/// được tiếp tục như một microtask.
Future<List<String>> traceAwaitOrder() async {
  final log = <String>[];

  Future<void> work() async {
    log.add('work: trước await');
    await null;
    log.add('work: sau await');
  }

  final pending = work();
  log.add('caller: sau khi gọi work');
  await pending;
  return log;
}
```

Kết quả là `[sync, microtask, event]`, và test khẳng định đúng thứ tự đó. Hàm `traceQueues` ghi đủ cả năm cách xếp việc và test kiểm thứ tự `[sync, scheduleMicrotask, Future.microtask, Timer.run, Future]`: microtask chen lên trước, còn `Timer.run` và `Future(() {})` cùng vào event queue theo thứ tự đăng ký.

Hàm thứ hai cho thấy điều hay gây bất ngờ: hàm `async` chạy **đồng bộ cho tới `await` đầu tiên**. Thứ tự là `work: trước await`, `caller: sau khi gọi work`, rồi mới `work: sau await`; về mặt khái niệm, phần sau `await` được xếp như một microtask.

Hai hệ quả. Thứ nhất, `await` **không chặn** luồng: nó đăng ký "chạy tiếp khi có kết quả" rồi trả quyền cho loop. Thứ hai, vòng lặp nặng không có `await` thì **không ai nhường**. Màn hình 60 khung hình/giây chỉ có khoảng 16 ms mỗi khung, nên vài trăm ms tính toán là đủ gây giật.

So với C#: .NET có thread pool nên phần sau `await` có thể chạy ở luồng khác; trong một isolate Dart thì không, nhưng giữa hai `await` việc khác vẫn xen vào được. Dart cũng **không có `CancellationToken`**: ta dừng một luồng dữ liệu bằng `cancel` subscription (mục Stream bên dưới). `CancelableOperation` của `package:async` nằm ngoài phạm vi bài này.

## Future và async/await: gọi dịch vụ báo giá

`Future<T>` là **giá trị chưa có**. Nó bắt đầu ở trạng thái *pending* và kết thúc đúng một lần, với một giá trị hoặc một lỗi. Có hai cách chờ:

- `then` đăng ký callback chạy khi có kết quả, nối chuỗi được.
- `await` (trong hàm `async`) viết cùng việc đó như mã tuần tự. Lỗi của Future thành exception ngay tại dòng `await`, nên `try/catch/finally` dùng như mã đồng bộ.

`QuoteService` của lab trả lời sau một độ trễ do constructor tiêm vào, mặc định 300 ms, để test chỉnh nó mà không cần chờ thật.

### quote_service.dart · 05-async-stream/lib/quote_service.dart (dòng 1 đến 48)

```dart
import 'dart:async';

/// Lỗi nghiệp vụ của dịch vụ báo giá (khác với lỗi lập trình như ArgumentError).
class QuoteException implements Exception {
  QuoteException(this.message, {this.cause});

  final String message;
  final Object? cause;

  @override
  String toString() => 'QuoteException: $message';
}

/// Bảng giá minh hoạ: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt,
/// miền hợp lệ 0 đến 100 km. Không phải bảng giá thực tế.
int calculateFareVnd(double distanceKm) {
  if (!(distanceKm >= 0 && distanceKm <= 100)) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0 đến 100 km',
    );
  }
  // Tính bằng số nguyên mét để 2,001 km luôn ra 12.005 đồng (làm tròn nửa lên).
  final meters = (distanceKm * 1000).round();
  final extraMeters = meters > 2000 ? meters - 2000 : 0;
  return 12000 + (extraMeters * 4500 + 500) ~/ 1000;
}

/// Mô phỏng một dịch vụ báo giá gọi qua mạng: trả lời sau [latency].
class QuoteService {
  QuoteService({this.latency = const Duration(milliseconds: 300)});

  final Duration latency;

  /// Số lần gọi đã chạy xong phía "server", kể cả khi người gọi đã bỏ cuộc.
  int completedCalls = 0;

  Future<int> fetchFareVnd(double distanceKm) async {
    await Future<void>.delayed(latency);
    completedCalls++;
    try {
      return calculateFareVnd(distanceKm);
    } on ArgumentError catch (e) {
      throw QuoteException('Khoảng cách $distanceKm km không hợp lệ', cause: e);
    }
  }
}
```

Điểm cần để ý:

- Hàm `async` **luôn trả Future**. `return` thành Future có giá trị; `throw` thành Future lỗi, không ném ra ngay ở chỗ gọi.
- `calculateFareVnd` ném `ArgumentError` khi ngoài miền 0 đến 100 km (lỗi lập trình). `fetchFareVnd` đổi nó thành `QuoteException`, lỗi nghiệp vụ mà màn hình biết hiển thị.
- Cước tính theo số nguyên mét để 2,001 km luôn ra 12.005 đồng; dùng thẳng `double` sẽ ra 12.004 vì sai số dấu phẩy động.

Phía gọi dùng `then` hoặc `await`, và bắt lỗi đúng kiểu:

### main.dart · 05-async-stream/bin/main.dart (dòng 11 đến 20)

```dart
  final service = QuoteService();
  print('5 km => ${await service.fetchFareVnd(5)} VND');
  final parallel = await fetchFaresInParallel(service, [2, 5, 10]);
  print('2, 5, 10 km song song => $parallel');
  await service.fetchFareVnd(2).then((fare) => 'then: $fare VND').then(print);
  try {
    await service.fetchFareVnd(150);
  } on QuoteException catch (e) {
    print('150 km => $e');
  }
```

Có cả `await` trong biểu thức lẫn chuỗi `then`. Khối `try` chỉ bắt `QuoteException`, đúng kiểu ta biết xử lý.

Nếu một Future lỗi mà **không ai lắng nghe** (quên `await`), lỗi rơi xuống zone gốc như lỗi không bắt: chương trình console dừng, còn app Flutter chuyển nó tới handler lỗi toàn cục (`PlatformDispatcher.onError`), thường chỉ thành một dòng log dễ bị bỏ qua.

## Timeout và gọi song song

Mạng di động hay chậm, ta không muốn khách nhìn vòng quay vô hạn nên đặt hạn chờ:

### quote_service.dart · 05-async-stream/lib/quote_service.dart (dòng 50 đến 79)

```dart
/// Chờ tối đa [timeout]. Quá hạn thì ném [TimeoutException].
/// Lưu ý: việc đang chạy phía service không bị huỷ.
Future<int> fetchWithTimeout(
  QuoteService service,
  double distanceKm, {
  required Duration timeout,
}) {
  return service
      .fetchFareVnd(distanceKm)
      .timeout(
        timeout,
        onTimeout: () => throw TimeoutException(
          'Báo giá quá ${timeout.inMilliseconds} ms',
          timeout,
        ),
      );
}

/// Gọi song song nhiều quãng đường; kết quả giữ đúng thứ tự đầu vào.
Future<List<int>> fetchFaresInParallel(
  QuoteService service,
  List<double> distancesKm,
) {
  return Future.wait(distancesKm.map(service.fetchFareVnd));
}

/// Lấy báo giá của nguồn trả lời đầu tiên thành công hay thất bại.
Future<int> fetchFromFastest(List<QuoteService> services, double distanceKm) {
  return Future.any(services.map((s) => s.fetchFareVnd(distanceKm)));
}
```

Ba hàm, ba hành vi:

- `fetchWithTimeout`: `.timeout(timeout, onTimeout: ...)` trả một Future **mới**. Nếu Future gốc chưa xong khi hết hạn, Future mới hoàn tất bằng kết quả của `onTimeout`, ở đây là ném `TimeoutException`.
- `fetchFaresInParallel`: `Future.wait` trả kết quả **đúng thứ tự đầu vào**, bất kể cái nào xong trước. Ba quãng đường tốn khoảng 300 ms thay vì 900 ms. Nếu có Future lỗi, mặc định nó chờ các cái còn lại xong rồi báo lỗi xảy ra sớm nhất theo thời gian (không theo thứ tự danh sách); `eagerError: true` báo ngay khi có lỗi.
- `fetchFromFastest`: `Future.any` lấy Future **hoàn tất đầu tiên**, thành công hay lỗi.

Điểm quan trọng nhất: **timeout không huỷ việc đang chạy**. Future không có khái niệm huỷ. Hết hạn nghĩa là *ta thôi chờ*; dịch vụ vẫn xử lý tiếp và kết quả bị bỏ. Với báo giá thì vô hại, còn với lệnh tạo chuyến hay thanh toán, server có thể đã thực hiện thật (idempotency key và tra lại trạng thái sẽ học ở bài 20 và 46). Hình 2 là dòng thời gian của ca này.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 2. Dòng thời gian: dịch vụ báo giá xử lý 300 ms; phía gọi đặt timeout 100 ms nên nhận TimeoutException tại mốc 100 ms, trong khi dịch vụ vẫn chạy tới 300 ms và kết quả bị bỏ."><defs><marker id="fl05-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<text x="120" y="30" class="t-b">Phía dịch vụ (QuoteService)</text>
<rect x="120" y="42" width="580" height="44" rx="8" class="f-phys-soft s-phys"/><text x="134" y="70" class="t-small">Future.delayed 300 ms, vẫn chạy tới hết</text>
<text x="120" y="126" class="t-b">Phía gọi (await ... .timeout(100 ms))</text>
<rect x="120" y="138" width="193" height="44" rx="8" class="f-pk-soft s-pk"/><text x="134" y="166" class="t-small">đang chờ</text>
<rect x="313" y="138" width="237" height="44" rx="8" class="f-warn-soft s-warn"/><text x="327" y="166" class="t-small">TimeoutException</text>
<path d="M313 100 L313 200" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 4"/>
<path d="M700 100 L700 200" class="f-none s-muted" stroke-width="2" stroke-dasharray="6 4"/>
<text x="560" y="166" class="t-small">25500 bị bỏ</text>
<path d="M60 230 L840 230" class="f-none s-line" stroke-width="2" marker-end="url(#fl05-1-a)"/>
<text x="110" y="256" class="t-mono">0 ms</text><text x="283" y="256" class="t-mono">100 ms</text><text x="670" y="256" class="t-mono">300 ms</text>
<text x="60" y="284" class="t-small">Hết hạn nghĩa là thôi chờ, không phải huỷ việc đang chạy.</text>
</svg>
</div>
<figcaption>Hình 2. Timeout 100 ms trên một lời gọi mất 300 ms: nhánh timeout (cam) xảy ra tại 100 ms, còn dịch vụ vẫn chạy tới 300 ms.</figcaption>
</figure>

Test mà không ngồi chờ bằng `package:fake_async`: trong `fakeAsync((async) { ... })`, `Timer` và `Future.delayed` chạy theo **đồng hồ ảo**, còn `async.elapse(...)` tua đồng hồ đó.

`Outcome` gom kết quả hoặc lỗi của một Future để kiểm tra sau mỗi lần tua:

### quote_service_test.dart · 05-async-stream/test/quote_service_test.dart (dòng 8 đến 25)

```dart
class Outcome<T> {
  T? value;
  Object? error;
  bool done = false;

  void watch(Future<T> future) {
    future.then(
      (v) {
        value = v;
        done = true;
      },
      onError: (Object e) {
        error = e;
        done = true;
      },
    );
  }
}
```

### quote_service_test.dart · 05-async-stream/test/quote_service_test.dart (dòng 45 đến 55)

```dart
    test('fetchFareVnd trả 25500 cho 5 km sau đúng 300 ms ảo', () {
      fakeAsync((async) {
        final outcome = Outcome<int>()..watch(QuoteService().fetchFareVnd(5));

        async.elapse(const Duration(milliseconds: 299));
        expect(outcome.done, isFalse);

        async.elapse(const Duration(milliseconds: 1));
        expect(outcome.value, 25500);
      });
    });
```

### quote_service_test.dart · 05-async-stream/test/quote_service_test.dart (dòng 119 đến 138)

```dart
    test('timeout không huỷ việc đang chạy: service vẫn xử lý xong', () {
      fakeAsync((async) {
        final service = QuoteService();
        final outcome = Outcome<int>()
          ..watch(
            fetchWithTimeout(
              service,
              5,
              timeout: const Duration(milliseconds: 100),
            ),
          );

        async.elapse(const Duration(milliseconds: 100));
        expect(outcome.error, isA<TimeoutException>());
        expect(service.completedCalls, 0);

        async.elapse(const Duration(milliseconds: 200));
        expect(service.completedCalls, 1); // việc vẫn chạy, kết quả bị bỏ
      });
    });
```

Test đầu: tại 299 ms chưa có kết quả, tại 300 ms có 25.500. Test sau: hết hạn 100 ms thì có `TimeoutException`, `completedCalls` vẫn 0, rồi thành 1 sau 200 ms ảo, tức việc vẫn chạy. Cả hai chỉ tốn vài mili giây thật.

## Stream: lắng nghe vị trí tài xế

Báo giá là một giá trị, một lần. Vị trí tài xế là **một dãy giá trị theo thời gian**, việc của `Stream<T>`. Ta đăng ký bằng `listen` và nhận lại `StreamSubscription` để `pause`, `resume` hay `cancel`. Có hai loại:

- **Single-subscription** (mặc định): chỉ một người nghe, nghe lần hai ném `StateError`. Hợp với dữ liệu "của một chủ", như vị trí tài xế trên một màn hình.
- **Broadcast**: nhiều người nghe cùng lúc, nhưng ai nghe muộn **bỏ lỡ** các sự kiện đã phát.

`DriverLocationFeed` tạo stream bằng `StreamController`: ta tự đẩy sự kiện vào (`add`, `addError`, `close`), còn người nghe chỉ thấy `controller.stream`. Các hàm `onListen`, `onPause`, `onResume`, `onCancel` cho nguồn biết lúc nào chạy, lúc nào dừng.

### driver_location.dart · 05-async-stream/lib/driver_location.dart (dòng 36 đến 87)

```dart
/// Mô phỏng nguồn vị trí của một tài xế: phát lần lượt các điểm của [route].
class DriverLocationFeed {
  DriverLocationFeed(this.route);

  final List<Position> route;

  /// Phát một vị trí mỗi [every]. Timer chỉ chạy khi có người nghe.
  ///
  /// Mặc định là single-subscription: nghe lần hai ném StateError, tạm dừng
  /// (pause) thì nguồn dừng phát. Với [broadcast] = true, nhiều người cùng
  /// nghe được, nhưng người nghe muộn bỏ lỡ các vị trí đã phát.
  Stream<Position> positions({
    required Duration every,
    bool broadcast = false,
  }) {
    Timer? timer;
    var index = 0;
    late final StreamController<Position> controller;

    void stop() {
      timer?.cancel();
      timer = null;
    }

    void start() {
      timer = Timer.periodic(every, (_) {
        controller.add(route[index++]);
        if (index == route.length) {
          stop();
          controller.close();
        }
      });
    }

    controller = broadcast
        ? StreamController<Position>.broadcast(onListen: start, onCancel: stop)
        : StreamController<Position>(
            onListen: start,
            onPause: stop,
            onResume: start,
            onCancel: stop,
          );
    return controller.stream;
  }

  /// Chỉ giữ vị trí cách vị trí đã phát trước đó hơn [meters] mét.
  /// Vị trí đầu tiên luôn được phát.
  Stream<Position> movedMoreThan(
    double meters, {
    Duration every = const Duration(seconds: 1),
  }) => onlyMoved(positions(every: every), meters);
}
```

Timer **chỉ bắt đầu khi có người nghe** và **dừng khi huỷ** (`onCancel`); nguồn không biết người nghe đã đi sẽ chạy mãi và rò rỉ. Với stream mặc định, `onPause` dừng timer và `onResume` cho chạy lại: đây là **backpressure** sơ lược, người nghe bận thì nguồn ngừng đẩy thay vì dồn sự kiện vào bộ nhớ (broadcast không có cơ chế này). Đẩy xong điểm cuối ta `close()` để người nghe biết dãy đã hết.

Để chỉ cập nhật bản đồ khi tài xế dịch chuyển đủ xa, `movedMoreThan` biến đổi stream. Phần lõi là `onlyMoved`, một **generator `async*`** dùng `await for`:

### driver_location.dart · 05-async-stream/lib/driver_location.dart (dòng 89 đến 99)

```dart
/// Biến đổi một stream vị trí thành stream chỉ gồm các lần di chuyển đáng kể.
Stream<Position> onlyMoved(Stream<Position> source, double meters) async* {
  Position? last; // trạng thái riêng cho từng lần listen
  await for (final p in source) {
    final previous = last;
    if (previous == null || distanceMeters(previous, p) > meters) {
      last = p;
      yield p;
    }
  }
}
```

`await for` chờ lần lượt từng sự kiện của `source`. Biến `last` nằm trong thân hàm nên **mỗi lần listen có trạng thái riêng**; đặt nó ngoài hàm và dùng `where` thì hai người nghe giẫm lên nhau. Khoảng cách tính bằng `distanceMeters` (equirectangular xấp xỉ, đủ cho quãng ngắn). Test kiểm bằng `expectLater(stream, emitsInOrder([p0, p3, p5, emitsDone]))`: với ngưỡng 50 m, ba trong sáu điểm đi qua.

Phía gọi, và chu kỳ sống của subscription ở Hình 3:

### main.dart · 05-async-stream/bin/main.dart (dòng 43 đến 67)

```dart
  final feed = DriverLocationFeed(route);
  final done = Completer<void>();
  late final StreamSubscription<Position> subscription;
  var received = 0;
  subscription = feed.positions(every: const Duration(milliseconds: 50)).listen(
    (p) async {
      received++;
      print('Vị trí #$received: $p');
      if (received == 5) {
        await subscription.cancel();
        print('Đã huỷ subscription, không nhận thêm.');
        done.complete();
      }
    },
  );
  await done.future;

  print('--- 3b. await for với take(3): tự huỷ khi đủ ---');
  final latitudes = feed
      .positions(every: const Duration(milliseconds: 30))
      .map((p) => p.lat.toStringAsFixed(4))
      .take(3);
  await for (final lat in latitudes) {
    print('lat $lat');
  }
```

Đoạn đầu dùng `listen` rồi `cancel()` ở vị trí thứ 5. Đoạn sau nối `map` và `take(3)` (cùng họ với `where`) rồi `await for`: khi `take` đủ 3, nó huỷ subscription của nguồn rồi đóng stream nên `await for` kết thúc. `break`, `return` hay exception bên trong `await for` cũng huỷ subscription (có test `break`). Nguyên tắc: **mọi `listen` phải có chỗ `cancel`**, trừ khi stream tự kết thúc.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 3. Chu kỳ sống của subscription: listen khởi động nguồn, năm sự kiện vị trí đến, cancel dừng timer; các vị trí sau đó không bao giờ tới."><defs><marker id="fl05-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs>
<rect x="30" y="30" width="170" height="64" rx="10" class="f-pk-soft s-pk"/><text x="46" y="60" class="t-b">listen()</text><text x="46" y="82" class="t-small">onListen: bật timer</text>
<path d="M115 94 L115 150" class="f-none s-pk" stroke-width="2"/>
<path d="M60 150 L840 150" class="f-none s-line" stroke-width="2" marker-end="url(#fl05-2-a)"/>
<circle cx="230" cy="150" r="10" class="f-phys s-phys"/><text x="222" y="130" class="t-mono">#1</text>
<circle cx="320" cy="150" r="10" class="f-phys s-phys"/><text x="312" y="130" class="t-mono">#2</text>
<circle cx="410" cy="150" r="10" class="f-phys s-phys"/><text x="402" y="130" class="t-mono">#3</text>
<circle cx="500" cy="150" r="10" class="f-phys s-phys"/><text x="492" y="130" class="t-mono">#4</text>
<circle cx="590" cy="150" r="10" class="f-phys s-phys"/><text x="582" y="130" class="t-mono">#5</text>
<rect x="640" y="30" width="200" height="64" rx="10" class="f-warn-soft s-warn"/><text x="656" y="60" class="t-b">cancel()</text><text x="656" y="82" class="t-small">onCancel: huỷ timer</text>
<path d="M650 94 L650 150" class="f-none s-warn" stroke-width="2"/>
<circle cx="710" cy="150" r="10" class="f-none s-muted" stroke-dasharray="4 3"/><circle cx="770" cy="150" r="10" class="f-none s-muted" stroke-dasharray="4 3"/><circle cx="830" cy="150" r="10" class="f-none s-muted" stroke-dasharray="4 3"/>
<text x="860" y="198" class="t-small" text-anchor="end">#6, #7, #8 không bao giờ tới</text>
<rect x="30" y="230" width="270" height="76" rx="10" class="f-phys-soft s-phys"/><text x="46" y="260" class="t-b">Single-subscription</text><text x="46" y="284" class="t-small">1 người nghe, có pause/buffer</text>
<rect x="320" y="230" width="270" height="76" rx="10" class="f-phys-soft s-phys"/><text x="336" y="260" class="t-b">Broadcast</text><text x="336" y="284" class="t-small">nhiều người, nghe muộn là lỡ</text>
<rect x="610" y="230" width="240" height="76" rx="10" class="f-warn-soft s-warn"/><text x="626" y="260" class="t-b">Quên cancel</text><text x="626" y="284" class="t-small">timer chạy mãi, rò rỉ</text>
</svg>
</div>
<figcaption>Hình 3. Subscribe, nhận sự kiện, cancel: sau cancel nguồn phải dừng hẳn, không chỉ ngừng được nghe.</figcaption>
</figure>

Test huỷ kiểm hai điều: không nhận thêm sự kiện, và **không còn timer nào** chạy ngầm:

### driver_location_test.dart · 05-async-stream/test/driver_location_test.dart (dòng 97 đến 115)

```dart
    test('cancel dừng timer và không nhận thêm vị trí', () {
      fakeAsync((async) {
        final feed = DriverLocationFeed(route);
        final seen = <Position>[];
        final subscription = feed
            .positions(every: const Duration(seconds: 1))
            .listen(seen.add);

        async.elapse(const Duration(seconds: 3));
        expect(seen, hasLength(3));
        expect(async.pendingTimers, isNotEmpty);

        subscription.cancel();
        expect(async.pendingTimers, isEmpty); // không rò rỉ timer

        async.elapse(const Duration(seconds: 10));
        expect(seen, hasLength(3));
      });
    });
```

Dòng `pendingTimers` rỗng là bằng chứng nguồn đã dừng; chỉ đếm sự kiện thì chưa đủ.

## Isolate.run: việc nặng sang luồng khác

Có việc tốn CPU thật sự, như parse JSON vài MB hay xử lý tuyến đường dài. `await` không giúp gì vì không có gì để chờ: CPU đang bận. Lúc này mới cần luồng thứ hai: `Isolate.run` chạy một hàm trong **isolate mới** rồi trả kết quả qua Future.

### heavy.dart · 05-async-stream/lib/heavy.dart

```dart
import 'dart:async';
import 'dart:isolate';

/// Tổng 1..n bằng vòng lặp (cố ý tốn CPU để minh hoạ).
int sumUpTo(int n) {
  if (n < 0) throw ArgumentError.value(n, 'n', 'không được âm');
  var total = 0;
  for (var i = 1; i <= n; i++) {
    total += i;
  }
  return total;
}

/// Chạy [sumUpTo] ở isolate khác. Closure và dữ liệu nó bắt được (ở đây là n)
/// được copy sang isolate mới; kết quả trả về qua Isolate.exit nên được
/// chuyển sang isolate gọi mà không copy. Isolate mới không chia sẻ bộ nhớ
/// với isolate gọi, nên vòng lặp nặng không chặn event loop của UI.
Future<int> sumInIsolate(int n) => Isolate.run(() => sumUpTo(n));

/// Đếm số lần timer 20 ms kịp chạy trong lúc [work] thực hiện.
/// Nhiều tick nghĩa là event loop của isolate gọi vẫn rảnh.
Future<({int result, int ticks})> measureTicks(
  Future<int> Function() work,
) async {
  var ticks = 0;
  final timer = Timer.periodic(
    const Duration(milliseconds: 20),
    (_) => ticks++,
  );
  try {
    final result = await work();
    return (result: result, ticks: ticks);
  } finally {
    timer.cancel();
  }
}
```

Điều cần nhớ:

- Isolate mới **không chia sẻ bộ nhớ** với isolate gọi. Closure và dữ liệu nó bắt được **copy** sang isolate mới; kết quả trả về qua `Isolate.exit` nên được chuyển sang isolate gọi **không copy**. Không có data race, nhưng gửi đối tượng lớn vào có giá, và closure có thể vô tình bắt nhiều trạng thái hơn ta tưởng. Nên chỉ bắt giá trị đơn giản (số, chuỗi, danh sách), không bắt đối tượng giữ tài nguyên UI như `BuildContext`.
- Lỗi ném trong isolate được **ném lại ở phía gọi**: `sumInIsolate(-1)` nhận `ArgumentError` như lời gọi thường.
- Tạo isolate có chi phí cố định cỡ mili giây, nên đừng dùng cho việc nhỏ.

Khi chạy `bin/main.dart`, `measureTicks` đếm số lần timer 20 ms kịp chạy. Với 600 triệu vòng lặp ngay trên main isolate, timer gần như không chạy lần nào (trong app Flutter, đó là màn hình đứng hình); với `Isolate.run` timer vẫn chạy đều. Số lần phụ thuộc máy, nên test chỉ kiểm tính đúng của `sumInIsolate`, không so thời gian. Trên iOS/Android, `compute()` của Flutter tương đương `Isolate.run(() => fn(message))`; trên web nó chạy trên cùng luồng. Bài 41 dùng nó để parse JSON lớn và đo bằng DevTools.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với Future và Stream</span>
<p><strong>1. Quên <code>await</code>.</strong> Dấu hiệu: log in <code>Instance of 'Future&lt;int&gt;'</code>, test kiểm tra trước khi dữ liệu về, lỗi biến mất vào zone gốc. Sửa: thêm <code>await</code>; cố ý không chờ thì bọc <code>unawaited(...)</code>, và bật lint <code>unawaited_futures</code>.</p>
<p><strong>2. Quên <code>cancel</code> subscription.</strong> Dấu hiệu: màn hình đã đóng vẫn nhận vị trí, lỗi <code>setState</code> sau khi widget bị huỷ, pin tụt vì timer chạy mãi. Sửa: lưu <code>StreamSubscription</code> và <code>cancel</code> khi xong (ở bài 10 là trong <code>dispose</code>), hoặc dùng <code>await for</code> kèm <code>take</code>; test bằng <code>pendingTimers</code>.</p>
<p><strong>3. Dùng broadcast khi không cần.</strong> Dấu hiệu: thêm <code>asBroadcastStream()</code> chỉ để hết <code>StateError</code> "already been listened to", rồi người nghe muộn lặng lẽ mất sự kiện đầu. Sửa: giữ single-subscription và tìm xem vì sao lại nghe hai lần.</p>
<p><strong>4. <code>catch (e)</code> hoặc <code>on Exception</code> nuốt lỗi.</strong> <code>ArgumentError</code> là <code>Error</code>, không phải <code>Exception</code>, nên <code>on Exception</code> bỏ lọt nó; <code>catch (_) {}</code> rỗng thì nuốt tất cả và có thể trả cước 0 đồng như chuyến miễn phí. Sửa: bắt đúng kiểu (<code>on QuoteException</code>) và xử lý có nghĩa, không thì để lỗi đi tiếp.</p>
<p><strong>5. <code>async</code> trong constructor.</strong> Constructor không thể <code>async</code>, nên người ta gọi hàm bất đồng bộ không <code>await</code> trong thân nó. Dấu hiệu: đối tượng được dùng khi dữ liệu chưa sẵn sàng, lỗi lúc có lúc không. Sửa: hàm static trả Future (<code>static Future&lt;Foo&gt; create()</code>) hoặc bước <code>load()</code> mà người gọi phải <code>await</code>.</p>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/05-async-stream
dart pub get
dart analyze
dart test
dart run bin/main.dart
```

Hoặc chạy cả bài 01 đến 06 bằng `node examples/toi-noi-flutter/verify-01-06.mjs`. Đã kiểm với Dart 3.13.3 (Flutter 3.47.4): `dart analyze` không issue, `dart test` 34 test xanh, `dart run` xong trong vài giây. Mức: **Kiểm chứng tự động**, không cần thiết bị.

| Nhóm test | Chứng minh điều gì |
| --- | --- |
| `calculateFareVnd` | Biên 0, 2, 2,001, 100 km; ngoài miền hoặc NaN ném `ArgumentError` |
| `fake_async` | 25.500 sau đúng 300 ms ảo; `QuoteException` qua Future; `TimeoutException` sau 100 ms ảo, việc phía service vẫn chạy tiếp |
| `Future.wait` / `Future.any` | Giữ thứ tự đầu vào; 300 ms ảo cho 3 quãng đường; mặc định chờ hết rồi mới báo lỗi, `eagerError` báo ngay; nguồn nhanh nhất thắng |
| `onlyMoved`, `positions` | `emitsInOrder` đúng điểm; `StateError` khi nghe lần hai; broadcast, người nghe muộn bỏ lỡ; `cancel` và `await for` + `break` không để lại timer; `pause`/`resume` |
| event loop | `['sync', 'microtask', 'event']` và thứ tự đủ năm cách xếp việc; async chạy đồng bộ tới `await` đầu |
| `Isolate.run` | `sumInIsolate` đúng, lỗi được ném lại |

**Chưa làm**: mạng thật và retry (bài 19, 20), GPS và WebSocket thật (bài 35, 36), isolate sống lâu với `SendPort`, và so thời gian thật giữa main isolate với `Isolate.run`.

## Bài tập

1. `fetchFaresInParallel` cho `[2, 5, 10]` tốn 300 ms ảo. Nếu viết vòng `for` và `await` từng lời gọi, tốn bao lâu, vì sao?
2. Vì sao test `cancel` kiểm `async.pendingTimers` rỗng thay vì chỉ kiểm số sự kiện không tăng?
3. Có nên bọc `fetchFareVnd` trong `Isolate.run` để app bớt giật không?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Khoảng 900 ms ảo: mỗi <code>await</code> chờ xong rồi mới gọi lời gọi kế, nên ba độ trễ cộng dồn. <code>Future.wait</code> khởi động cả ba ngay nên chỉ tốn bằng độ trễ dài nhất.</p>
<p><strong>2.</strong> Số sự kiện đứng yên vẫn đúng khi timer còn chạy ngầm mà controller bỏ qua sự kiện, tức vẫn rò rỉ. <code>pendingTimers</code> rỗng chứng minh <code>onCancel</code> thực sự dừng nguồn.</p>
<p><strong>3.</strong> Không. Lời gọi báo giá chủ yếu chờ I/O, không tốn CPU, và <code>await</code> đã không chặn luồng. Isolate chỉ thêm chi phí tạo isolate và gửi dữ liệu; nó dành cho việc tốn CPU.</p>
</details>

Bài trước: [Collections, generics và extension](/articles/dart-collections-generics-extension-danh-sach-chuyen). Bài tiếp: [Package, pubspec, lint và test](/articles/dart-package-pubspec-lint-test-toi-noi-core).

## Tài liệu gốc

- [Asynchronous programming: futures, async, await](https://dart.dev/libraries/async/async-await) (dart.dev).
- [Asynchronous programming: streams](https://dart.dev/libraries/async/using-streams) (dart.dev).
- [Concurrency in Dart](https://dart.dev/language/concurrency) (dart.dev).
- [Future](https://api.dart.dev/dart-async/Future-class.html), [StreamController](https://api.dart.dev/dart-async/StreamController-class.html) và [Isolate.run](https://api.dart.dev/dart-isolate/Isolate/run.html) (api.dart.dev).
- [Concurrency and isolates](https://docs.flutter.dev/perf/isolates) (docs.flutter.dev).
- [fake_async](https://pub.dev/packages/fake_async) (pub.dev).
