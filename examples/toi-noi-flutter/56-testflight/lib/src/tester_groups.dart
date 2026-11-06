/// Vai trò trong tài khoản App Store Connect. Theo trang TestFlight của Apple,
/// tester nội bộ phải giữ một trong năm vai trò đầu; mọi vai trò khác là [other].
enum AscRole {
  accountHolder,
  admin,
  appManager,
  developer,
  marketing,
  other;

  bool get canTestInternally => this != other;
}

enum GroupKind { internal, external }

enum LinkMode { openToAnyone, filterByCriteria }

/// Liên kết mời công khai của một nhóm external.
class PublicLink {
  const PublicLink.openToAnyone({this.testerLimit})
    : mode = LinkMode.openToAnyone,
      criteria = const [];

  /// Lọc theo thiết bị và phiên bản OS, ví dụ `iPhone, iOS 17.0 trở lên`.
  const PublicLink.filtered({required this.criteria, this.testerLimit})
    : mode = LinkMode.filterByCriteria;

  final LinkMode mode;
  final List<String> criteria;

  /// Số tester tối đa vào bằng link; `null` là không đặt giới hạn.
  final int? testerLimit;
}

class TesterGroup {
  /// Nhóm internal: người dùng App Store Connect, kèm vai trò.
  const TesterGroup.internal({
    required this.name,
    required Map<String, AscRole> members,
  }) : kind = GroupKind.internal,
       internalMembers = members,
       externalEmails = const {},
       publicLink = null;

  /// Nhóm external: mời bằng email và/hoặc public link.
  const TesterGroup.external({
    required this.name,
    this.externalEmails = const {},
    this.publicLink,
  }) : kind = GroupKind.external,
       internalMembers = const {};

  final String name;
  final GroupKind kind;
  final Map<String, AscRole> internalMembers;
  final Set<String> externalEmails;
  final PublicLink? publicLink;

  /// Chỉ nhóm external cần Beta App Review (bản đầu của mỗi version).
  bool get needsBetaReview => kind == GroupKind.external;

  Iterable<String> get emails =>
      kind == GroupKind.internal ? internalMembers.keys : externalEmails;
}

class GroupIssue {
  const GroupIssue(this.code, this.group, this.message);

  final String code;
  final String group;
  final String message;

  @override
  String toString() => '$code [$group]: $message';
}

/// Kế hoạch nhóm tester của MỘT app trên App Store Connect (app khách và app
/// tài xế là hai app, mỗi app có kế hoạch riêng).
class TesterGroups {
  const TesterGroups(this.groups);

  /// Tới 100 người dùng App Store Connect làm tester nội bộ (kiểm tại 11/2026).
  static const internalLimit = 100;

  /// Tới 10.000 tester external mỗi app (kiểm tại 11/2026).
  static const externalLimit = 10000;

  final List<TesterGroup> groups;

  Set<String> _emailsOf(GroupKind kind) => {
    for (final g in groups)
      if (g.kind == kind) ...g.emails,
  };

  /// Số người khác nhau trong mọi nhóm internal.
  int get internalCount => _emailsOf(GroupKind.internal).length;

  /// Số người khác nhau được mời bằng email vào nhóm external. Người vào bằng
  /// public link hiện ẩn danh và không nằm trong con số này.
  int get externalCount => _emailsOf(GroupKind.external).length;

  List<GroupIssue> validate() {
    final issues = <GroupIssue>[];
    final names = <String>{};
    for (final g in groups) {
      if (!names.add(g.name)) {
        issues.add(
          GroupIssue('duplicate-group-name', g.name, 'Tên nhóm bị trùng.'),
        );
      }
      for (final email in g.emails) {
        if (!_email.hasMatch(email)) {
          issues.add(
            GroupIssue(
              'tester-email-invalid',
              g.name,
              '"$email" không phải địa chỉ email.',
            ),
          );
        }
      }
      if (g.kind == GroupKind.internal) {
        for (final e in g.internalMembers.entries) {
          if (!e.value.canTestInternally) {
            issues.add(
              GroupIssue(
                'internal-role-ineligible',
                g.name,
                '${e.key} có vai trò không được làm tester nội bộ. Đổi vai '
                    'trò trong Users and Access hoặc mời làm tester external.',
              ),
            );
          }
        }
      } else {
        _checkExternal(g, issues);
      }
    }
    if (internalCount > internalLimit) {
      issues.add(
        GroupIssue(
          'internal-over-limit',
          'internal',
          '$internalCount tester nội bộ, giới hạn $internalLimit.',
        ),
      );
    }
    if (externalCount > externalLimit) {
      issues.add(
        GroupIssue(
          'external-over-limit',
          'external',
          '$externalCount tester external, giới hạn $externalLimit.',
        ),
      );
    }
    final hasInternal = groups.any((g) => g.kind == GroupKind.internal);
    final hasExternal = groups.any((g) => g.kind == GroupKind.external);
    if (hasExternal && !hasInternal) {
      issues.add(
        const GroupIssue(
          'external-without-internal',
          'external',
          'Apple đòi tạo nhóm internal trước khi tạo nhóm external.',
        ),
      );
    }
    return issues;
  }

  void _checkExternal(TesterGroup g, List<GroupIssue> issues) {
    final link = g.publicLink;
    if (g.externalEmails.isEmpty && link == null) {
      issues.add(
        GroupIssue(
          'group-empty',
          g.name,
          'Nhóm external không có tester nào và không có public link.',
        ),
      );
    }
    if (link == null) return;
    final limit = link.testerLimit;
    if (limit != null && (limit < 1 || limit > externalLimit)) {
      issues.add(
        GroupIssue(
          'public-link-limit-range',
          g.name,
          'Giới hạn của public link phải từ 1 đến $externalLimit, nhận $limit.',
        ),
      );
    }
    if (link.mode == LinkMode.filterByCriteria && link.criteria.isEmpty) {
      issues.add(
        GroupIssue(
          'public-link-no-criteria',
          g.name,
          'Link lọc theo tiêu chí cần ít nhất một tiêu chí.',
        ),
      );
    }
  }

  static final _email = RegExp(r'^[^@\s]+@[^@\s]+\.[^@\s]+$');

  /// App tài xế: nhóm nội bộ và mười hai tài xế thử nghiệm mời từng người
  /// bằng email, vì ta cần biết ai là ai để gắn với cuốc thử. Số người là
  /// minh hoạ, địa chỉ dùng tên miền giả `toinoi.example`.
  factory TesterGroups.driverBeta() => TesterGroups([
    _internalGroup(),
    TesterGroup.external(
      name: 'Tài xế thử nghiệm',
      externalEmails: {
        for (var i = 1; i <= 12; i++)
          'taixe${i.toString().padLeft(2, '0')}@toinoi.example',
      },
    ),
  ]);

  /// App khách: nhóm nội bộ và một public link lọc theo thiết bị. Người vào
  /// bằng link hiện ẩn danh, chấp nhận được với khách thử.
  factory TesterGroups.customerBeta() => TesterGroups([
    _internalGroup(),
    const TesterGroup.external(
      name: 'Khách thử nghiệm',
      publicLink: PublicLink.filtered(
        criteria: ['iPhone, iOS 17.0 trở lên'],
        testerLimit: 100,
      ),
    ),
  ]);

  static TesterGroup _internalGroup() => const TesterGroup.internal(
    name: 'Nội bộ Tới Nơi',
    members: {
      'dev.an@toinoi.example': AscRole.developer,
      'dev.binh@toinoi.example': AscRole.developer,
      'pm.chi@toinoi.example': AscRole.appManager,
      'qa.dung@toinoi.example': AscRole.developer,
      'mkt.giang@toinoi.example': AscRole.marketing,
    },
  );
}
