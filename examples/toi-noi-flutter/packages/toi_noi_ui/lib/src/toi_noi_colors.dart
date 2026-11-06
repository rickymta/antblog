import 'package:flutter/material.dart';

/// Màu ngữ nghĩa của Tới Nơi mà `ColorScheme` của Material 3 không có:
/// thành công, cảnh báo và nguy hiểm.
///
/// Bộ `danger` đặt đúng bằng `error`, `onError`, `errorContainer`,
/// `onErrorContainer` của `ColorScheme` mà [ToiNoiTheme] sinh ra, để app
/// không có hai màu đỏ gần nhau.
///
/// Mỗi vai trò có bốn màu theo mẫu của Material: màu chính, màu chữ/icon đặt
/// trên màu chính (`on...`), màu nền nhạt (`...Container`) và màu chữ đặt
/// trên nền nhạt (`on...Container`). Mọi cặp `on...` và nền đều đạt tỉ lệ
/// tương phản WCAG AA 4.5:1 ở cả bản sáng và bản tối (có test kiểm).
///
/// Đăng ký qua `ThemeData.extensions`, đọc bằng [ToiNoiColors.of].
@immutable
class ToiNoiColors extends ThemeExtension<ToiNoiColors> {
  /// Tạo bộ màu; thường dùng [ToiNoiColors.light] hoặc [ToiNoiColors.dark].
  const ToiNoiColors({
    required this.success,
    required this.onSuccess,
    required this.successContainer,
    required this.onSuccessContainer,
    required this.warning,
    required this.onWarning,
    required this.warningContainer,
    required this.onWarningContainer,
    required this.danger,
    required this.onDanger,
    required this.dangerContainer,
    required this.onDangerContainer,
  });

  /// Bộ màu cho giao diện sáng.
  static const ToiNoiColors light = ToiNoiColors(
    success: Color(0xFF1B6E3C),
    onSuccess: Color(0xFFFFFFFF),
    successContainer: Color(0xFFC8F0D3),
    onSuccessContainer: Color(0xFF0E5A2E),
    warning: Color(0xFF8A5A00),
    onWarning: Color(0xFFFFFFFF),
    warningContainer: Color(0xFFFFE3A6),
    onWarningContainer: Color(0xFF5C3D00),
    danger: Color(0xFFBA1A1A),
    onDanger: Color(0xFFFFFFFF),
    dangerContainer: Color(0xFFFFDAD6),
    onDangerContainer: Color(0xFF93000A),
  );

  /// Bộ màu cho giao diện tối: màu chính sáng hơn, nền nhạt đậm hơn.
  static const ToiNoiColors dark = ToiNoiColors(
    success: Color(0xFF7FD99A),
    onSuccess: Color(0xFF00391A),
    successContainer: Color(0xFF0F5132),
    onSuccessContainer: Color(0xFFB7F0C5),
    warning: Color(0xFFF2C05C),
    onWarning: Color(0xFF3E2800),
    warningContainer: Color(0xFF5C3D00),
    onWarningContainer: Color(0xFFFFE0A0),
    danger: Color(0xFFFFB4AB),
    onDanger: Color(0xFF690005),
    dangerContainer: Color(0xFF93000A),
    onDangerContainer: Color(0xFFFFDAD6),
  );

  /// Màu thành công, ví dụ chuyến đã hoàn thành.
  final Color success;

  /// Chữ và icon đặt trên [success].
  final Color onSuccess;

  /// Nền nhạt cho trạng thái thành công.
  final Color successContainer;

  /// Chữ và icon đặt trên [successContainer].
  final Color onSuccessContainer;

  /// Màu cảnh báo, ví dụ đang chờ tài xế.
  final Color warning;

  /// Chữ và icon đặt trên [warning].
  final Color onWarning;

  /// Nền nhạt cho trạng thái cảnh báo.
  final Color warningContainer;

  /// Chữ và icon đặt trên [warningContainer].
  final Color onWarningContainer;

  /// Màu nguy hiểm, ví dụ chuyến đã huỷ.
  final Color danger;

  /// Chữ và icon đặt trên [danger].
  final Color onDanger;

  /// Nền nhạt cho trạng thái nguy hiểm.
  final Color dangerContainer;

  /// Chữ và icon đặt trên [dangerContainer].
  final Color onDangerContainer;

  /// Lấy bộ màu từ theme của [context].
  ///
  /// Nếu `ThemeData` không đăng ký [ToiNoiColors] (ví dụ app dùng
  /// `ThemeData()` trơn), trả về [light] hoặc [dark] theo
  /// `ThemeData.brightness` thay vì ném lỗi.
  static ToiNoiColors of(BuildContext context) {
    final theme = Theme.of(context);
    return theme.extension<ToiNoiColors>() ??
        (theme.brightness == Brightness.dark ? dark : light);
  }

  @override
  ToiNoiColors copyWith({
    Color? success,
    Color? onSuccess,
    Color? successContainer,
    Color? onSuccessContainer,
    Color? warning,
    Color? onWarning,
    Color? warningContainer,
    Color? onWarningContainer,
    Color? danger,
    Color? onDanger,
    Color? dangerContainer,
    Color? onDangerContainer,
  }) {
    return ToiNoiColors(
      success: success ?? this.success,
      onSuccess: onSuccess ?? this.onSuccess,
      successContainer: successContainer ?? this.successContainer,
      onSuccessContainer: onSuccessContainer ?? this.onSuccessContainer,
      warning: warning ?? this.warning,
      onWarning: onWarning ?? this.onWarning,
      warningContainer: warningContainer ?? this.warningContainer,
      onWarningContainer: onWarningContainer ?? this.onWarningContainer,
      danger: danger ?? this.danger,
      onDanger: onDanger ?? this.onDanger,
      dangerContainer: dangerContainer ?? this.dangerContainer,
      onDangerContainer: onDangerContainer ?? this.onDangerContainer,
    );
  }

  /// Nội suy từng màu. Khi [other] không phải [ToiNoiColors] trả về chính nó,
  /// đúng quy ước của `ThemeExtension.lerp`.
  @override
  ToiNoiColors lerp(ThemeExtension<ToiNoiColors>? other, double t) {
    if (other is! ToiNoiColors) {
      return this;
    }
    return ToiNoiColors(
      success: Color.lerp(success, other.success, t)!,
      onSuccess: Color.lerp(onSuccess, other.onSuccess, t)!,
      successContainer: Color.lerp(
        successContainer,
        other.successContainer,
        t,
      )!,
      onSuccessContainer: Color.lerp(
        onSuccessContainer,
        other.onSuccessContainer,
        t,
      )!,
      warning: Color.lerp(warning, other.warning, t)!,
      onWarning: Color.lerp(onWarning, other.onWarning, t)!,
      warningContainer: Color.lerp(
        warningContainer,
        other.warningContainer,
        t,
      )!,
      onWarningContainer: Color.lerp(
        onWarningContainer,
        other.onWarningContainer,
        t,
      )!,
      danger: Color.lerp(danger, other.danger, t)!,
      onDanger: Color.lerp(onDanger, other.onDanger, t)!,
      dangerContainer: Color.lerp(dangerContainer, other.dangerContainer, t)!,
      onDangerContainer: Color.lerp(
        onDangerContainer,
        other.onDangerContainer,
        t,
      )!,
    );
  }

  @override
  bool operator ==(Object other) =>
      other is ToiNoiColors &&
      other.success == success &&
      other.onSuccess == onSuccess &&
      other.successContainer == successContainer &&
      other.onSuccessContainer == onSuccessContainer &&
      other.warning == warning &&
      other.onWarning == onWarning &&
      other.warningContainer == warningContainer &&
      other.onWarningContainer == onWarningContainer &&
      other.danger == danger &&
      other.onDanger == onDanger &&
      other.dangerContainer == dangerContainer &&
      other.onDangerContainer == onDangerContainer;

  @override
  int get hashCode => Object.hash(
    success,
    onSuccess,
    successContainer,
    onSuccessContainer,
    warning,
    onWarning,
    warningContainer,
    onWarningContainer,
    danger,
    onDanger,
    dangerContainer,
    onDangerContainer,
  );
}
