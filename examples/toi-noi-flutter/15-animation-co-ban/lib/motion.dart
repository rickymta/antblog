import 'package:flutter/widgets.dart';

/// Thời lượng, đường cong và quy tắc "giảm chuyển động" dùng chung cho mọi
/// animation của lab.
///
/// Mọi widget đi qua [ToiNoiMotion.duration] thay vì gõ `Duration` trực tiếp:
/// khi người dùng bật cài đặt giảm chuyển động của hệ điều hành, tất cả cùng
/// chuyển sang `Duration.zero` ở một chỗ.
abstract final class ToiNoiMotion {
  /// 150 ms: thay đổi nhỏ, ví dụ hiện một dòng chữ phụ.
  static const Duration quick = Duration(milliseconds: 150);

  /// 300 ms: thay đổi thường, ví dụ đổi nội dung hay đổi màu thẻ.
  static const Duration standard = Duration(milliseconds: 300);

  /// 600 ms: chuyển động cần được nhìn thấy, ví dụ vòng tiến độ.
  static const Duration slow = Duration(milliseconds: 600);

  /// Đường cong cho phần tử đang đổi trạng thái tại chỗ.
  static const Curve standardCurve = Curves.easeInOut;

  /// Đường cong cho phần tử đi vào: nhanh lúc đầu, chậm dần khi tới đích.
  static const Curve enterCurve = Curves.easeOutCubic;

  /// Người dùng có đang yêu cầu giảm chuyển động không.
  ///
  /// Đọc `MediaQuery.disableAnimationsOf` nên widget gọi hàm này được dựng lại
  /// khi cài đặt đổi. Chỉ phản ánh cờ `disableAnimations` của Flutter; cờ
  /// `reduceMotion` riêng của iOS chưa được đọc (xem README của lab).
  static bool reduced(BuildContext context) =>
      MediaQuery.disableAnimationsOf(context);

  /// Trả [normal], hoặc `Duration.zero` khi [reduced].
  static Duration duration(BuildContext context, Duration normal) =>
      reduced(context) ? Duration.zero : normal;
}
