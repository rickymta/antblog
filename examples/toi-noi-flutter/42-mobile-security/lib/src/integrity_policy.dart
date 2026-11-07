/// Tín hiệu về tính toàn vẹn của thiết bị, do plugin hoặc code native cung cấp.
///
/// Đây là **gợi ý**: kẻ tấn công có công cụ che nó, còn người dùng bình
/// thường đôi khi bị báo nhầm. Lab không chứa bộ phát hiện nào, chỉ mô hình
/// hoá cách app dùng tín hiệu.
enum IntegritySignal { rootOrJailbreakHint, debuggerAttached, hookingHint }

/// Việc app có thể làm, xếp theo độ nhạy cảm.
enum AppAction {
  viewTripHistory(sensitive: false),
  createTrip(sensitive: false),
  changePhoneNumber(sensitive: true),
  changePayoutAccount(sensitive: true);

  const AppAction({required this.sensitive});

  final bool sensitive;
}

/// Quyết định của app, từ nhẹ tới nặng. Không có "chặn hẳn": tín hiệu sai là
/// chuyện thường, và chặn là cách biến báo nhầm thành mất khách.
enum Decision { allow, allowAndReport, requireReauthentication }

/// Chính sách phản ứng với tín hiệu.
///
/// - Việc nhạy cảm luôn đòi xác thực lại, có tín hiệu hay không: không có tín
///   hiệu không chứng minh máy sạch.
/// - Việc thường chỉ gửi tín hiệu lên server để chấm rủi ro; server mới là nơi
///   quyết định cuối cùng.
Decision decide(AppAction action, Set<IntegritySignal> signals) {
  if (action.sensitive) return Decision.requireReauthentication;
  return signals.isEmpty ? Decision.allow : Decision.allowAndReport;
}
