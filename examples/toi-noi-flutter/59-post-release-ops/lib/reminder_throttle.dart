import 'app_version.dart';

/// Nơi nhớ lần nhắc cuối. Bản thật dùng `shared_preferences` (bài 22).
abstract interface class ReminderStore {
  Future<({DateTime shownAt, AppVersion version})?> read();
  Future<void> write(DateTime shownAt, AppVersion version);
}

/// Giới hạn tần suất nhắc "nên cập nhật". Chỉ áp cho mức recommend; mức force
/// không đi qua đây.
///
/// Nhắc lại khi: chưa từng nhắc, đã qua [interval], hoặc server nâng
/// `recommended` lên bản khác với bản đã nhắc.
final class ReminderThrottle {
  ReminderThrottle({
    required this._store,
    required this._clock,
    this.interval = const Duration(days: 3),
  });

  final ReminderStore _store;
  final DateTime Function() _clock;
  final Duration interval;

  Future<bool> shouldRemind(AppVersion recommended) async {
    final ({DateTime shownAt, AppVersion version})? last;
    try {
      last = await _store.read();
    } on Object {
      // Không đọc được bộ nhớ: im lặng còn hơn nhắc ở mỗi lần mở app.
      return false;
    }
    if (last == null || last.version != recommended) return true;
    final now = _clock();
    // Đồng hồ máy lùi về trước lần nhắc cuối: coi như đã hết hạn thay vì
    // im lặng vô thời hạn.
    if (now.isBefore(last.shownAt)) return true;
    return now.difference(last.shownAt) >= interval;
  }

  Future<void> markShown(AppVersion recommended) async {
    try {
      await _store.write(_clock(), recommended);
    } on Object {
      // Lỗi ghi chỉ làm lần sau có thể nhắc sớm; không được làm hỏng luồng mở app.
    }
  }
}
