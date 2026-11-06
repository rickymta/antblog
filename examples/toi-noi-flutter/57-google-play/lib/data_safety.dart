/// Bản nháp khai báo Data safety, đối chiếu với quyền trong AndroidManifest.
///
/// Bảng "quyền kéo theo loại dữ liệu" là quy tắc của lab, không phải bảng của
/// Google. Người khai chịu trách nhiệm khai đầy đủ và đúng (trang Data safety
/// của Play Console, kiểm tại 11/2026); lab chỉ bắt thiếu sót hiển nhiên.
library;

import 'manifest_permissions.dart';

/// Hai loại dữ liệu vị trí trong form (tên trong form: Approximate location,
/// Precise location). Các loại khác không suy ra được từ quyền manifest.
enum DataType { approximateLocation, preciseLocation }

/// Mục đích, theo danh sách trong form.
enum DataPurpose {
  appFunctionality,
  analytics,
  developerCommunications,
  advertisingOrMarketing,
  fraudPreventionSecurityCompliance,
  personalization,
  accountManagement,
}

final class DataDeclaration {
  const DataDeclaration({
    required this.type,
    required this.purposes,
    this.collected = true,
    this.shared = false,
    this.optional = true,
  });

  final DataType type;

  /// "Collected" theo Google: dữ liệu rời khỏi máy người dùng.
  final bool collected;
  final bool shared;

  /// Người dùng có thể tắt việc thu thập này không.
  final bool optional;
  final Set<DataPurpose> purposes;
}

enum Severity { error, warning }

enum FindingCode {
  missingDeclaration,
  noPurpose,
  noPrivacyPolicy,
  privacyPolicyNotHttps,
  declaredWithoutPermission,
  notEncryptedInTransit,
  noDeletionPath,
}

final class Finding {
  const Finding(this.code, this.severity, this.message);
  final FindingCode code;
  final Severity severity;
  final String message;

  @override
  String toString() => '${severity.name}: ${code.name}: $message';
}

/// Loại dữ liệu mà các quyền trong manifest buộc phải được khai.
Set<DataType> requiredTypesFor(Set<String> permissions) => {
  if (permissions.contains(coarseLocation)) DataType.approximateLocation,
  if (permissions.contains(fineLocation)) ...{
    DataType.preciseLocation,
    // Người dùng có thể chỉ cấp vị trí gần đúng dù app khai FINE (tài liệu
    // Android về quyền vị trí), nên khi có FINE ta khai cả hai loại.
    DataType.approximateLocation,
  },
};

final class DataSafetyDraft {
  const DataSafetyDraft({
    required this.declarations,
    required this.encryptedInTransit,
    required this.deletionRequestSupported,
    required this.privacyPolicyUrl,
  });

  final List<DataDeclaration> declarations;
  final bool encryptedInTransit;
  final bool deletionRequestSupported;
  final String? privacyPolicyUrl;

  Set<DataType> get declaredTypes => {for (final d in declarations) d.type};

  /// Đối chiếu bản nháp với quyền của manifest [xml].
  List<Finding> check(String xml) {
    final permissions = readPermissions(xml);
    final required = requiredTypesFor(permissions);
    final declared = declaredTypes;
    return [
      for (final type in DataType.values)
        if (required.contains(type) && !declared.contains(type))
          Finding(
            FindingCode.missingDeclaration,
            Severity.error,
            'manifest có quyền kéo theo ${type.name} nhưng chưa khai',
          ),
      for (final d in declarations)
        if (d.collected && d.purposes.isEmpty)
          Finding(
            FindingCode.noPurpose,
            Severity.error,
            '${d.type.name} được thu thập nhưng chưa nêu mục đích',
          ),
      for (final type in declared)
        if (!required.contains(type))
          Finding(
            FindingCode.declaredWithoutPermission,
            Severity.warning,
            '${type.name} đã khai nhưng manifest không có quyền tương ứng: '
            'đúng nếu dữ liệu đến từ nguồn khác (ví dụ vị trí gần đúng '
            'suy ra từ IP phía server), còn không thì khai thừa',
          ),
      ..._policyFindings(),
      if (!encryptedInTransit)
        const Finding(
          FindingCode.notEncryptedInTransit,
          Severity.warning,
          'dữ liệu không mã hoá khi truyền: cần lý do rõ ràng',
        ),
      if (!deletionRequestSupported)
        const Finding(
          FindingCode.noDeletionPath,
          Severity.warning,
          'chưa có cách để người dùng yêu cầu xoá dữ liệu',
        ),
    ];
  }

  Iterable<Finding> _policyFindings() sync* {
    final url = privacyPolicyUrl?.trim() ?? '';
    if (url.isEmpty) {
      yield const Finding(
        FindingCode.noPrivacyPolicy,
        Severity.error,
        'form Data safety đòi liên kết chính sách quyền riêng tư',
      );
    } else if (!url.startsWith('https://')) {
      yield Finding(
        FindingCode.privacyPolicyNotHttps,
        Severity.error,
        'liên kết chính sách phải là https: $url',
      );
    }
  }

  /// Không còn lỗi chặn; cảnh báo vẫn có thể còn.
  bool isCompleteFor(String xml) =>
      check(xml).every((f) => f.severity != Severity.error);
}

/// Bản nháp của app tài xế Tới Nơi (tên miền hư cấu).
const DataSafetyDraft driverDraft = DataSafetyDraft(
  declarations: [
    DataDeclaration(
      type: DataType.preciseLocation,
      purposes: {DataPurpose.appFunctionality},
      optional: false,
    ),
    DataDeclaration(
      type: DataType.approximateLocation,
      purposes: {DataPurpose.appFunctionality},
      optional: false,
    ),
  ],
  encryptedInTransit: true,
  deletionRequestSupported: true,
  privacyPolicyUrl: 'https://toinoi.example/privacy',
);
