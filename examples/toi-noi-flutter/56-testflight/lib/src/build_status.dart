/// Màu chỉ báo của Apple: đỏ là cần hành động hoặc đã hết, vàng là một quy
/// trình đang chạy, xanh là đang được test.
enum StatusColor { red, yellow, green }

/// Trạng thái build trong TestFlight, đúng tên Apple dùng trong trang
/// "App build statuses" (kiểm tại 11/2026). Trạng thái xử lý (Processing) không
/// nằm trong trang đó nên không có ở đây.
enum BuildStatus {
  invalidBinary(
    'Invalid Binary',
    StatusColor.red,
    'Apple nhận được build nhưng nó không đạt yêu cầu tải lên. Đọc email lỗi, '
        'sửa, tải lại với build number mới.',
  ),
  missingCompliance(
    'Missing Compliance',
    StatusColor.yellow,
    'Thiếu thông tin export compliance. Trả lời câu hỏi ở mục Manage, hoặc '
        'khai ITSAppUsesNonExemptEncryption trong Info.plist cho các bản sau.',
  ),
  waitingForExportComplianceReview(
    'Waiting for Export Compliance Review',
    StatusColor.yellow,
    'Tài liệu compliance đã nộp và đang chờ duyệt. Không cần làm gì.',
  ),
  inComplianceReview(
    'In Compliance Review',
    StatusColor.yellow,
    'Tài liệu compliance đang được duyệt. Không cần làm gì.',
  ),
  readyToSubmit(
    'Ready to Submit',
    StatusColor.yellow,
    'Gán cho nhóm internal để test ngay, hoặc gửi TestFlight App Review để '
        'test external.',
  ),
  waitingForReview(
    'Waiting for Review',
    StatusColor.yellow,
    'Đã gửi Beta App Review. Chưa test external được cho tới khi được duyệt.',
  ),
  inBetaReview(
    'In Beta Review',
    StatusColor.yellow,
    'TestFlight App Review đang duyệt build.',
  ),
  notAvailableForTesting(
    'Not Available for Testing',
    StatusColor.red,
    'Provisioning profile thiếu application identifier. Tạo lại profile App '
        'Store cho đúng bundle ID rồi build lại.',
  ),
  expired(
    'Expired',
    StatusColor.red,
    'Build quá cửa sổ 90 ngày. Tải lên build mới.',
  ),
  rejected(
    'Rejected',
    StatusColor.red,
    'TestFlight App Review từ chối và build không dùng lại được. Đọc lý do ở '
        'mục App Review, sửa, tải build mới.',
  ),
  readyToTest(
    'Ready to Test',
    StatusColor.green,
    'Tester internal và external test được nhưng chưa thêm tester nào. Thêm '
        'nhóm vào build.',
  ),
  testing(
    'Testing',
    StatusColor.green,
    'Ít nhất một nhóm hoặc tester đang test build này.',
  );

  const BuildStatus(this.label, this.color, this.advice);

  /// Tên hiển thị trong App Store Connect.
  final String label;
  final StatusColor color;

  /// Việc tiếp theo, bằng tiếng Việt.
  final String advice;

  /// Trạng thái kết thúc: build không thể dùng lại.
  bool get isDead => switch (this) {
    invalidBinary || notAvailableForTesting || expired || rejected => true,
    _ => false,
  };

  /// Đọc tên Apple (không phân biệt hoa thường, bỏ khoảng trắng thừa).
  /// Ném [FormatException] nếu không phải một trong mười hai trạng thái.
  static BuildStatus parse(String text) {
    final wanted = text.trim().toLowerCase();
    for (final s in values) {
      if (s.label.toLowerCase() == wanted) return s;
    }
    throw FormatException('Trạng thái build lạ: "$text"');
  }
}
