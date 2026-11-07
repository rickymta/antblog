/// Tám nhóm kiểm soát của OWASP MASVS v2 (mas.owasp.org/MASVS), với tên nhóm
/// theo trang chính thức.
enum MasvsGroup {
  storage('STORAGE', 'Storage'),
  crypto('CRYPTO', 'Cryptography'),
  auth('AUTH', 'Authentication and Authorization'),
  network('NETWORK', 'Network Communication'),
  platform('PLATFORM', 'Platform Interaction'),
  code('CODE', 'Code Quality'),
  resilience(
    'RESILIENCE',
    'Resilience Against Reverse Engineering and Tampering',
  ),
  privacy('PRIVACY', 'Privacy');

  const MasvsGroup(this.key, this.name);

  final String key;
  final String name;
}

/// Mức làm được của Tới Nơi cho một control.
enum Coverage { done, partial, notDone }

/// Một dòng của checklist: ID control, ý chính (tiếng Việt, diễn đạt lại),
/// mức đã làm, bài liên quan và ghi chú còn thiếu gì.
final class MasvsItem {
  const MasvsItem(
    this.group,
    this.number,
    this.summary,
    this.coverage,
    this.lessons,
    this.note,
  );

  final MasvsGroup group;
  final int number;
  final String summary;
  final Coverage coverage;
  final List<int> lessons;
  final String note;

  /// ID theo MASVS v2, ví dụ `MASVS-STORAGE-1`.
  String get id => 'MASVS-${group.key}-$number';
}

/// 24 control của MASVS v2, theo trạng thái của Tới Nơi sau bài 42.
const masvsChecklist = <MasvsItem>[
  MasvsItem(
    MasvsGroup.storage,
    1,
    'Lưu dữ liệu nhạy cảm an toàn',
    Coverage.partial,
    [22, 24, 42],
    'Token qua flutter_secure_storage, tắt backup Android trong native-config. Chưa kiểm trên thiết bị.',
  ),
  MasvsItem(
    MasvsGroup.storage,
    2,
    'Không để dữ liệu nhạy cảm rò ra ngoài',
    Coverage.partial,
    [19, 42],
    'Redactor và SafeLogInterceptor. Chưa: ảnh chụp ở app switcher, clipboard, crash report.',
  ),
  MasvsItem(
    MasvsGroup.crypto,
    1,
    'Dùng mật mã hiện hành, đúng cách',
    Coverage.partial,
    [42],
    'App không tự cài thuật toán; SHA-256 chỉ để tính ghim. Chưa rà soát toàn bộ.',
  ),
  MasvsItem(
    MasvsGroup.crypto,
    2,
    'Quản lý khoá đúng cách',
    Coverage.partial,
    [22],
    'Khoá do Keychain/Keystore giữ. Chưa có khoá gắn sinh trắc học hay quy trình xoay khoá.',
  ),
  MasvsItem(
    MasvsGroup.auth,
    1,
    'Giao thức xác thực và phân quyền an toàn',
    Coverage.done,
    [24],
    'OTP, JWT ngắn hạn, refresh token xoay vòng; server cưỡng chế phân quyền.',
  ),
  MasvsItem(
    MasvsGroup.auth,
    2,
    'Xác thực cục bộ an toàn',
    Coverage.notDone,
    [],
    'Chưa có sinh trắc học hay PIN cục bộ.',
  ),
  MasvsItem(
    MasvsGroup.auth,
    3,
    'Thao tác nhạy cảm cần xác thực thêm',
    Coverage.partial,
    [42],
    'AppAction.sensitive đòi xác thực lại; mới là mô hình, chưa có màn hình.',
  ),
  MasvsItem(
    MasvsGroup.network,
    1,
    'Mọi lưu lượng mạng đi theo thực hành hiện hành',
    Coverage.done,
    [19, 20, 25],
    'HTTPS cho staging và prod; dev dùng localhost.',
  ),
  MasvsItem(
    MasvsGroup.network,
    2,
    'Ghim danh tính cho endpoint do ta quản lý',
    Coverage.partial,
    [42],
    'Ghim SPKI với hai ghim, thử trên localhost. Chưa có hash thật, chưa rotation thật.',
  ),
  MasvsItem(
    MasvsGroup.platform,
    1,
    'Dùng IPC an toàn',
    Coverage.partial,
    [23],
    'Deep link qua go_router có guard. Chưa rà intent filter và component exported.',
  ),
  MasvsItem(MasvsGroup.platform, 2, 'Dùng WebView an toàn', Coverage.notDone, [
    39,
  ], 'WebView thanh toán ở bài 39 cần giới hạn URL và JavaScript.'),
  MasvsItem(
    MasvsGroup.platform,
    3,
    'Giao diện không làm lộ dữ liệu',
    Coverage.notDone,
    [37],
    'Chưa che màn hình ở app switcher; nội dung thông báo push cần kiểm.',
  ),
  MasvsItem(
    MasvsGroup.code,
    1,
    'Yêu cầu phiên bản hệ điều hành đủ mới',
    Coverage.notDone,
    [],
    'Chưa chốt phiên bản OS tối thiểu (iOS deployment target, Android minSdk).',
  ),
  MasvsItem(MasvsGroup.code, 2, 'Có cơ chế ép cập nhật app', Coverage.notDone, [
    59,
  ], 'Ép cập nhật thuộc bài 59.'),
  MasvsItem(
    MasvsGroup.code,
    3,
    'Chỉ dùng thành phần không có lỗ hổng đã biết',
    Coverage.partial,
    [63],
    'dart pub outdated, bài 63 về nâng cấp. Chưa có quét lỗ hổng tự động.',
  ),
  MasvsItem(
    MasvsGroup.code,
    4,
    'Kiểm tra và làm sạch mọi đầu vào không tin cậy',
    Coverage.done,
    [11, 19],
    'Form validation và DTO kiểm kiểu.',
  ),
  MasvsItem(
    MasvsGroup.resilience,
    1,
    'Kiểm tra tính toàn vẹn của nền tảng',
    Coverage.partial,
    [42],
    'Mô hình IntegritySignal. Chưa có bộ phát hiện, chưa Play Integrity hay App Attest.',
  ),
  MasvsItem(
    MasvsGroup.resilience,
    2,
    'Chống chỉnh sửa app',
    Coverage.notDone,
    [],
    'Chưa làm.',
  ),
  MasvsItem(
    MasvsGroup.resilience,
    3,
    'Chống phân tích tĩnh',
    Coverage.partial,
    [42],
    '--obfuscate chỉ đổi tên; chưa áp vào build phát hành (Phần 7).',
  ),
  MasvsItem(
    MasvsGroup.resilience,
    4,
    'Chống phân tích động',
    Coverage.notDone,
    [],
    'Chưa làm.',
  ),
  MasvsItem(
    MasvsGroup.privacy,
    1,
    'Giảm tối đa dữ liệu và quyền truy cập',
    Coverage.partial,
    [35, 38],
    'Quyền vị trí xin đúng lúc, nền chỉ khi có chuyến (theo khung bài 35, 38).',
  ),
  MasvsItem(
    MasvsGroup.privacy,
    2,
    'Ngăn việc nhận diện người dùng',
    Coverage.partial,
    [42],
    'Redactor làm tròn toạ độ trong log. Chưa có định danh giả danh.',
  ),
  MasvsItem(
    MasvsGroup.privacy,
    3,
    'Minh bạch về thu thập và sử dụng dữ liệu',
    Coverage.notDone,
    [58],
    'Nhãn quyền riêng tư khi phát hành (bài 58).',
  ),
  MasvsItem(
    MasvsGroup.privacy,
    4,
    'Người dùng kiểm soát dữ liệu của mình',
    Coverage.notDone,
    [],
    'Chưa có xoá tài khoản và dữ liệu.',
  ),
];

/// Số control theo mức, để đối chiếu với bảng trong bài.
Map<Coverage, int> coverageCounts([
  Iterable<MasvsItem> items = masvsChecklist,
]) {
  final counts = {for (final c in Coverage.values) c: 0};
  for (final item in items) {
    counts[item.coverage] = counts[item.coverage]! + 1;
  }
  return counts;
}
