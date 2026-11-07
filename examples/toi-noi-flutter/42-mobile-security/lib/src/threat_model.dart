import 'masvs_checklist.dart';

/// Hai app của Tới Nơi.
enum AppKind { customer, driver }

/// Kẻ tấn công hoặc tình huống ta phòng thủ.
enum Attacker {
  /// Người nhặt được hoặc mượn máy đã mở khoá.
  lostDevice,

  /// App khác (độc hại) cài trên cùng máy.
  otherApp,

  /// Người ngồi giữa đường mạng: Wi-Fi công cộng, proxy có chứng chỉ cài thêm.
  networkObserver,

  /// Người dùng root/jailbreak máy hoặc sửa app để gian lận.
  tamperer,

  /// Người đọc log, bản sao lưu hoặc báo cáo lỗi (kể cả nhân viên hỗ trợ).
  logReader,
}

/// Biện pháp thực thi ở đâu: app chỉ làm khó, server mới là chỗ kiểm cuối.
enum EnforcedOn { client, server }

/// Một dòng của mô hình đe doạ: ai muốn lấy gì và ta chặn bằng gì.
final class Threat {
  const Threat({
    required this.apps,
    required this.asset,
    required this.attacker,
    required this.control,
    required this.masvs,
    required this.enforcedOn,
  });

  final Set<AppKind> apps;
  final String asset;
  final Attacker attacker;
  final String control;

  /// ID control MASVS mà biện pháp này thuộc về.
  final String masvs;
  final EnforcedOn enforcedOn;
}

/// Mô hình đe doạ của app khách và app tài xế (số liệu và tình huống minh hoạ).
const threatModel = <Threat>[
  Threat(
    apps: {AppKind.customer, AppKind.driver},
    asset: 'Access và refresh token',
    attacker: Attacker.lostDevice,
    control: 'Lưu trong Keychain/Keystore, token ngắn hạn, refresh xoay vòng',
    masvs: 'MASVS-STORAGE-1',
    enforcedOn: EnforcedOn.client,
  ),
  Threat(
    apps: {AppKind.customer, AppKind.driver},
    asset: 'Access và refresh token',
    attacker: Attacker.otherApp,
    control: 'Không ghi token vào SharedPreferences; tắt backup',
    masvs: 'MASVS-STORAGE-1',
    enforcedOn: EnforcedOn.client,
  ),
  Threat(
    apps: {AppKind.customer, AppKind.driver},
    asset: 'Phiên đăng nhập',
    attacker: Attacker.networkObserver,
    control: 'HTTPS; ghim SPKI cho API của Tới Nơi',
    masvs: 'MASVS-NETWORK-2',
    enforcedOn: EnforcedOn.client,
  ),
  Threat(
    apps: {AppKind.customer},
    asset: 'Số điện thoại và địa chỉ đón',
    attacker: Attacker.logReader,
    control: 'Redactor trước khi ghi log; không ghi body request',
    masvs: 'MASVS-STORAGE-2',
    enforcedOn: EnforcedOn.client,
  ),
  Threat(
    apps: {AppKind.driver},
    asset: 'Vị trí chính xác của tài xế',
    attacker: Attacker.logReader,
    control: 'Làm tròn toạ độ trong log; xoá bộ đệm sau khi gửi',
    masvs: 'MASVS-PRIVACY-2',
    enforcedOn: EnforcedOn.client,
  ),
  Threat(
    apps: {AppKind.driver},
    asset: 'Tài khoản nhận tiền của tài xế',
    attacker: Attacker.lostDevice,
    control: 'Đổi tài khoản nhận tiền đòi xác thực lại; server kiểm lần nữa',
    masvs: 'MASVS-AUTH-3',
    enforcedOn: EnforcedOn.server,
  ),
  Threat(
    apps: {AppKind.driver},
    asset: 'Dữ liệu vị trí gửi lên (chống giả GPS)',
    attacker: Attacker.tamperer,
    control: 'Tín hiệu root/hook gửi lên server để chấm rủi ro, không tự chặn',
    masvs: 'MASVS-RESILIENCE-1',
    enforcedOn: EnforcedOn.server,
  ),
  Threat(
    apps: {AppKind.customer, AppKind.driver},
    asset: 'Logic cước và API nội bộ',
    attacker: Attacker.tamperer,
    control: 'Obfuscation làm chậm người đọc; cước tính lại ở server',
    masvs: 'MASVS-RESILIENCE-3',
    enforcedOn: EnforcedOn.server,
  ),
  Threat(
    apps: {AppKind.customer, AppKind.driver},
    asset: 'Khoá API của bên thứ ba',
    attacker: Attacker.tamperer,
    control: 'Không đưa vào app; server giữ, app gọi qua BFF',
    masvs: 'MASVS-STORAGE-1',
    enforcedOn: EnforcedOn.server,
  ),
];

/// ID control mà [threat] tham chiếu có nằm trong checklist không.
bool referencesKnownControl(Threat threat) =>
    masvsChecklist.any((item) => item.id == threat.masvs);
