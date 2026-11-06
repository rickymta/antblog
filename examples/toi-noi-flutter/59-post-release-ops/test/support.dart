import 'dart:async';

import 'package:toi_noi_59/app_version.dart';
import 'package:toi_noi_59/feature_flags.dart';
import 'package:toi_noi_59/reminder_throttle.dart';
import 'package:toi_noi_59/update_gate.dart';
import 'package:toi_noi_59/url_opener.dart';

/// Nguồn chính sách giả: trả giá trị, ném lỗi, hoặc không bao giờ xong.
final class FakePolicySource implements PolicySource {
  FakePolicySource.value(this._value) : _error = null, _hang = false;
  FakePolicySource.throwing(this._error) : _value = null, _hang = false;
  FakePolicySource.hanging() : _value = null, _error = null, _hang = true;

  /// Trả lời khi test gọi [complete]; dùng để dựng "route được push lúc đang chờ".
  FakePolicySource.manual() : _value = null, _error = null, _hang = true;

  final Object? _value;
  final Object? _error;
  final bool _hang;
  int calls = 0;
  final Completer<Object?> _manual = Completer<Object?>();

  void complete(Object? value) => _manual.complete(value);

  @override
  Future<Object?> fetchRaw() {
    calls++;
    if (_hang) return _manual.future;
    final error = _error;
    if (error != null) return Future<Object?>.error(error);
    return Future<Object?>.value(_value);
  }
}

final class MemoryReminderStore implements ReminderStore {
  ({DateTime shownAt, AppVersion version})? last;
  bool failReads = false;
  bool failWrites = false;

  @override
  Future<({DateTime shownAt, AppVersion version})?> read() async {
    if (failReads) throw StateError('đọc lỗi');
    return last;
  }

  @override
  Future<void> write(DateTime shownAt, AppVersion version) async {
    if (failWrites) throw StateError('ghi lỗi');
    last = (shownAt: shownAt, version: version);
  }
}

/// Đồng hồ chỉnh tay.
final class FakeClock {
  FakeClock(this.now);
  DateTime now;
  DateTime call() => now;
  void advance(Duration d) => now = now.add(d);
}

/// Opener giả: ghi lại link đã mở.
final class FakeOpener implements UrlOpener {
  final List<Uri> opened = [];
  bool result = true;
  Object? error;

  @override
  Future<bool> open(Uri uri) async {
    opened.add(uri);
    final e = error;
    if (e != null) throw e;
    return result;
  }
}

final class FakeFlagSource implements RemoteFlagSource {
  FakeFlagSource([this.values = const {}]);
  Map<String, Object?> values;
  Object? error;
  int calls = 0;

  @override
  Future<Map<String, Object?>> fetch() async {
    calls++;
    final e = error;
    if (e != null) throw e;
    return values;
  }
}

final class MemoryFlagCache implements FlagCache {
  Map<String, Object?>? stored;
  bool failReads = false;
  bool failWrites = false;

  @override
  Future<Map<String, Object?>?> read() async {
    if (failReads) throw StateError('đọc lỗi');
    return stored;
  }

  @override
  Future<void> write(Map<String, Object?> values) async {
    if (failWrites) throw StateError('ghi lỗi');
    stored = Map.of(values);
  }
}
