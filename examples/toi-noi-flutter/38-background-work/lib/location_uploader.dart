import 'buffer_store.dart';
import 'location_buffer.dart';
import 'location_sample.dart';

/// Đích nhận vị trí (API của Tới Nơi ở bản thật). Ném [Exception] khi gửi
/// không được, ví dụ mất mạng hoặc server trả 5xx.
abstract interface class LocationSink {
  Future<void> send(List<LocationSample> batch);
}

/// Kết quả một lần [LocationUploader.flush].
final class FlushResult {
  const FlushResult({
    required this.sent,
    required this.pending,
    required this.failed,
  });

  final int sent;
  final int pending;
  final bool failed;

  @override
  String toString() =>
      'FlushResult(sent: $sent, pending: $pending, failed: $failed)';
}

/// Đẩy bộ đệm lên [sink] theo lô và chỉ xoá mẫu sau khi gửi thành công.
final class LocationUploader {
  LocationUploader({
    required this.sink,
    required this.buffer,
    this.store,
    this.batchSize = 20,
    this.onFailed,
    DateTime Function()? now,
  }) : _now = now ?? (() => DateTime.now().toUtc()),
       assert(batchSize > 0, 'batchSize phải dương');

  final LocationSink sink;
  final LocationBuffer buffer;
  final BufferStore? store;
  final int batchSize;

  /// Gọi khi một lần flush kết thúc vì sink lỗi, để app hẹn task nền gửi bù.
  final void Function(FlushResult result)? onFailed;
  final DateTime Function() _now;

  Future<FlushResult>? _inFlight;

  /// Thêm một mẫu rồi thử gửi ngay.
  Future<FlushResult> record(LocationSample sample) {
    buffer.add(sample);
    return flush();
  }

  /// Gửi lần lượt từng lô cũ nhất cho tới khi hết hoặc có lỗi.
  ///
  /// Hai lời gọi chồng nhau dùng chung một lần gửi, nếu không cùng một mẫu
  /// sẽ lên server hai lần.
  Future<FlushResult> flush() =>
      _inFlight ??= _flush().whenComplete(() => _inFlight = null);

  Future<FlushResult> _flush() async {
    buffer.purgeExpired(_now());
    var sent = 0;
    var failed = false;
    while (!buffer.isEmpty) {
      final batch = buffer.peek(batchSize);
      try {
        await sink.send(batch);
      } on Exception {
        failed = true;
        break;
      }
      buffer.removeSent(batch);
      sent += batch.length;
    }
    await _persist();
    final result = FlushResult(
      sent: sent,
      pending: buffer.length,
      failed: failed,
    );
    if (failed) onFailed?.call(result);
    return result;
  }

  /// Dọn bộ đệm khi chuyến kết thúc: vị trí của một chuyến đã xong không còn
  /// lý do để nằm lại trên máy.
  Future<void> discardPending() async {
    buffer.clear();
    await _persist();
  }

  Future<void> _persist() async {
    try {
      await store?.save(buffer.items);
    } on Exception {
      // Kho chỉ là bản sao dự phòng; bộ đệm trong bộ nhớ vẫn đúng.
    }
  }
}
