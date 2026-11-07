import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_38/buffer_store.dart';
import 'package:toi_noi_38/location_buffer.dart';
import 'package:toi_noi_38/location_sample.dart';
import 'package:toi_noi_38/location_uploader.dart';

import 'test_helpers.dart';

void main() {
  late FakeSink sink;
  late LocationBuffer buffer;
  late MemoryBufferStore store;
  late LocationUploader uploader;
  var clock = t0;

  setUp(() {
    sink = FakeSink();
    buffer = LocationBuffer(capacity: 100);
    store = MemoryBufferStore();
    clock = t0;
    uploader = LocationUploader(
      sink: sink,
      buffer: buffer,
      store: store,
      batchSize: 20,
      now: () => clock,
    );
  });

  test('gửi theo lô: 45 mẫu với batchSize 20 ra 20, 20, 5', () async {
    for (var i = 0; i < 45; i++) {
      buffer.add(sample(i));
    }
    final result = await uploader.flush();
    expect(sink.batches.map((b) => b.length), [20, 20, 5]);
    expect(result.sent, 45);
    expect(result.pending, 0);
    expect(result.failed, isFalse);
    expect(buffer.isEmpty, isTrue);
  });

  test('mất mạng: mẫu ở lại bộ đệm và được lưu xuống kho', () async {
    sink.failing = true;
    final result = await uploader.record(sample(0));
    expect(result.failed, isTrue);
    expect(result.pending, 1);
    expect(await store.load(), [sample(0)]);
  });

  test('có mạng lại: gửi đủ và đúng thứ tự cũ tới mới', () async {
    sink.failing = true;
    await uploader.record(sample(0));
    await uploader.record(sample(1));
    sink.failing = false;
    final result = await uploader.record(sample(2));
    expect(sink.sent, [sample(0), sample(1), sample(2)]);
    expect(result.failed, isFalse);
    expect(await store.load(), isEmpty);
  });

  test('lô lỗi giữa chừng: các lô đã gửi bị xoá, lô lỗi còn nguyên', () async {
    for (var i = 0; i < 30; i++) {
      buffer.add(sample(i));
    }
    // Lô đầu thành công rồi sink hỏng ở lô hai.
    final flaky = _FailAfter(sink, okCalls: 1);
    final result = await LocationUploader(
      sink: flaky,
      buffer: buffer,
      batchSize: 20,
      now: () => clock,
    ).flush();
    expect(result.sent, 20);
    expect(result.pending, 10);
    expect(result.failed, isTrue);
    expect(buffer.peek(1), [sample(20)]);
  });

  test('mẫu quá hạn bị bỏ trước khi gửi, không bao giờ lên server', () async {
    sink.failing = true;
    await uploader.record(sample(0)); // chụp lúc t0
    sink.failing = false;
    clock = t0.add(const Duration(minutes: 11));
    final result = await uploader.flush();
    expect(sink.batches, isEmpty);
    expect(result.sent, 0);
    expect(buffer.isEmpty, isTrue);
  });

  test(
    'hai flush chồng nhau dùng chung một lần gửi, không gửi trùng',
    () async {
      buffer.add(sample(0));
      final a = uploader.flush();
      final b = uploader.flush();
      expect(identical(a, b), isTrue);
      await Future.wait([a, b]);
      expect(sink.sent, [sample(0)]);
    },
  );

  test('discardPending xoá cả bộ đệm lẫn kho', () async {
    sink.failing = true;
    await uploader.record(sample(0));
    await uploader.discardPending();
    expect(buffer.isEmpty, isTrue);
    expect(await store.load(), isEmpty);
  });

  test('mẫu đến lúc đang gửi đẩy mẫu cũ ra khỏi bộ đệm đầy: không mất mẫu nào chưa gửi', () async {
    final tight = LocationBuffer(capacity: 3)
      ..add(sample(0))
      ..add(sample(1))
      ..add(sample(2));
    late LocationUploader racing;
    final racingSink = _OnFirstSend(sink, () {
      // Trong lúc lô [0, 1, 2] đang gửi, mẫu 3 đến và đẩy mẫu 0 ra khỏi bộ đệm.
      racing.buffer.add(sample(3));
    });
    racing = LocationUploader(
      sink: racingSink,
      buffer: tight,
      now: () => clock,
    );

    final result = await racing.flush();

    expect(sink.batches.map((b) => b.length), [3, 1]);
    expect(sink.sent, [sample(0), sample(1), sample(2), sample(3)]);
    expect(result.pending, 0, reason: 'mẫu 3 phải được gửi, không bị xoá nhầm');
  });

  test('onFailed chỉ được gọi khi gửi lỗi, kèm số mẫu còn chờ', () async {
    final failures = <FlushResult>[];
    final watched = LocationUploader(
      sink: sink,
      buffer: buffer,
      onFailed: failures.add,
      now: () => clock,
    );
    await watched.record(sample(0));
    expect(failures, isEmpty);

    sink.failing = true;
    await watched.record(sample(1));
    expect(failures, hasLength(1));
    expect(failures.single.pending, 1);
  });

  test('lỗi ghi kho không làm hỏng việc gửi', () async {
    final broken = LocationUploader(
      sink: sink,
      buffer: buffer,
      store: _ThrowingStore(),
      now: () => clock,
    );
    final result = await broken.record(sample(0));
    expect(result.failed, isFalse);
    expect(sink.sent, [sample(0)]);
  });
}

final class _OnFirstSend implements LocationSink {
  _OnFirstSend(this.inner, this.onFirst);
  final FakeSink inner;
  final void Function() onFirst;
  bool _first = true;

  @override
  Future<void> send(List<LocationSample> batch) async {
    await inner.send(batch);
    if (_first) {
      _first = false;
      onFirst();
    }
  }
}

final class _FailAfter implements LocationSink {
  _FailAfter(this.inner, {required this.okCalls});
  final FakeSink inner;
  int okCalls;

  @override
  Future<void> send(batch) async {
    if (okCalls-- <= 0) throw Exception('hỏng');
    await inner.send(batch);
  }
}

final class _ThrowingStore implements BufferStore {
  @override
  Future<List<Never>> load() async => const [];

  @override
  Future<void> save(samples) async => throw Exception('đĩa đầy');
}
