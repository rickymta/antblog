import 'package:flutter/widgets.dart';

/// Bộ đếm số lần `build` theo nhãn, dùng để biến "có vẻ chậm" thành một con
/// số so sánh được trước và sau khi sửa.
///
/// Đếm số lần build chứ không đo mili giây: số lần build không phụ thuộc máy,
/// nên test kiểm được. Thời gian thật phải đo bằng DevTools ở profile mode.
abstract final class BuildLog {
  static final Map<String, int> _counts = {};

  /// Ghi một lần build của [label].
  static void hit(String label) => _counts[label] = of(label) + 1;

  /// Số lần [label] đã build kể từ lần [reset] gần nhất.
  static int of(String label) => _counts[label] ?? 0;

  /// Xoá mọi bộ đếm. Gọi trong `setUp` để các test độc lập với nhau.
  static void reset() => _counts.clear();
}

/// Widget đo: mỗi lần `build` chạy là một lần [BuildLog.hit].
///
/// Đặt nó vào chỗ cần theo dõi, ví dụ `const BuildProbe('footer')`.
class BuildProbe extends StatelessWidget {
  /// Tạo widget đo cho [label].
  const BuildProbe(this.label, {super.key});

  /// Tên dùng trong [BuildLog].
  final String label;

  @override
  Widget build(BuildContext context) {
    BuildLog.hit(label);
    return const SizedBox.shrink();
  }
}

/// Ghi lại kiểu widget của mọi element được rebuild, qua
/// `debugOnRebuildDirtyWidget`.
///
/// Callback chỉ chạy ở debug build (nằm trong `assert`) và là một biến toàn
/// cục duy nhất, nên luôn gọi [stop] khi xong để test sau không bị ghi lẫn.
class RebuildRecorder {
  RebuildRecorder._();

  /// Bắt đầu ghi. Mỗi lần element build (kể cả lần dựng đầu) thêm một phần tử.
  factory RebuildRecorder.start() {
    final recorder = RebuildRecorder._();
    debugOnRebuildDirtyWidget = (element, _) {
      recorder.types.add(element.widget.runtimeType.toString());
    };
    return recorder;
  }

  /// Tên kiểu widget theo thứ tự rebuild.
  final List<String> types = [];

  /// Số lần widget kiểu [type] được build.
  int count(String type) => types.where((t) => t == type).length;

  /// Xoá những gì đã ghi, giữ nguyên việc ghi tiếp.
  void clear() => types.clear();

  /// Dừng ghi và gỡ callback.
  void stop() => debugOnRebuildDirtyWidget = null;
}
