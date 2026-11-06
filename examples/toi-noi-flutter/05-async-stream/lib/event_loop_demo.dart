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
