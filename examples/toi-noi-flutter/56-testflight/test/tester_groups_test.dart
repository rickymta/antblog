import 'package:test/test.dart';
import 'package:toi_noi_56/toi_noi_56.dart';

Map<String, AscRole> people(int n, [AscRole role = AscRole.developer]) => {
  for (var i = 0; i < n; i++) 'nv$i@toinoi.example': role,
};

Set<String> emails(int n) => {
  for (var i = 0; i < n; i++) 'kh$i@toinoi.example',
};

List<String> codes(TesterGroups g) => [for (final i in g.validate()) i.code];

void main() {
  group('kế hoạch của Tới Nơi', () {
    test('app tài xế: 5 nội bộ, 12 tài xế mời bằng email, không có issue', () {
      final plan = TesterGroups.driverBeta();
      expect(plan.internalCount, 5);
      expect(plan.externalCount, 12);
      expect(plan.validate(), isEmpty);
    });

    test('app tài xế mời từng người, không dùng public link', () {
      final external = TesterGroups.driverBeta().groups.last;
      expect(external.kind, GroupKind.external);
      expect(external.publicLink, isNull);
      expect(
        external.externalEmails,
        everyElement(endsWith('@toinoi.example')),
      );
    });

    test('app khách: public link lọc theo tiêu chí, không có email nào', () {
      final plan = TesterGroups.customerBeta();
      final link = plan.groups.last.publicLink!;
      expect(link.mode, LinkMode.filterByCriteria);
      expect(link.criteria, isNotEmpty);
      expect(link.testerLimit, 100);
      expect(plan.externalCount, 0);
      expect(plan.validate(), isEmpty);
    });

    test('chỉ nhóm external cần Beta App Review', () {
      final plan = TesterGroups.driverBeta();
      expect([for (final g in plan.groups) g.needsBetaReview], [false, true]);
    });
  });

  group('giới hạn tester', () {
    test('internal: đúng 100 người qua, 101 người vượt', () {
      final ok = TesterGroups([
        TesterGroup.internal(name: 'A', members: people(100)),
      ]);
      expect(ok.validate(), isEmpty);
      final over = TesterGroups([
        TesterGroup.internal(name: 'A', members: people(101)),
      ]);
      expect(codes(over), ['internal-over-limit']);
    });

    test('internal: một người ở hai nhóm chỉ tính một lần', () {
      final plan = TesterGroups([
        TesterGroup.internal(name: 'A', members: people(60)),
        TesterGroup.internal(name: 'B', members: people(60)),
      ]);
      expect(plan.internalCount, 60);
      expect(plan.validate(), isEmpty);
    });

    test('internal: hai nhóm khác người cộng lại vượt 100', () {
      final plan = TesterGroups([
        TesterGroup.internal(
          name: 'A',
          members: {
            for (var i = 0; i < 60; i++) 'a$i@toinoi.example': AscRole.admin,
          },
        ),
        TesterGroup.internal(
          name: 'B',
          members: {
            for (var i = 0; i < 60; i++) 'b$i@toinoi.example': AscRole.admin,
          },
        ),
      ]);
      expect(codes(plan), ['internal-over-limit']);
    });

    test('external: đúng 10.000 qua, 10.001 vượt', () {
      final internal = TesterGroup.internal(name: 'In', members: people(1));
      final ok = TesterGroups([
        internal,
        TesterGroup.external(name: 'Ex', externalEmails: emails(10000)),
      ]);
      expect(ok.validate(), isEmpty);
      final over = TesterGroups([
        internal,
        TesterGroup.external(name: 'Ex', externalEmails: emails(10001)),
      ]);
      expect(codes(over), ['external-over-limit']);
    });
  });

  group('vai trò internal', () {
    test('năm vai trò đầu được, vai trò khác không', () {
      for (final role in AscRole.values) {
        final plan = TesterGroups([
          TesterGroup.internal(name: 'A', members: {'x@toinoi.example': role}),
        ]);
        expect(
          codes(plan),
          role == AscRole.other ? ['internal-role-ineligible'] : isEmpty,
          reason: role.name,
        );
      }
    });
  });

  group('quy tắc nhóm external', () {
    test('external mà chưa có nhóm internal nào là sai', () {
      final plan = TesterGroups([
        TesterGroup.external(name: 'Ex', externalEmails: emails(1)),
      ]);
      expect(codes(plan), ['external-without-internal']);
    });

    test('chỉ có nhóm internal thì hợp lệ', () {
      final plan = TesterGroups([
        TesterGroup.internal(name: 'In', members: people(1)),
      ]);
      expect(plan.validate(), isEmpty);
    });

    test('external không có ai và không có link: cảnh báo group-empty', () {
      final plan = TesterGroups([
        TesterGroup.internal(name: 'In', members: people(1)),
        const TesterGroup.external(name: 'Ex'),
      ]);
      expect(codes(plan), ['group-empty']);
    });

    test('tên nhóm trùng bị báo', () {
      final plan = TesterGroups([
        TesterGroup.internal(name: 'Nhóm', members: people(1)),
        TesterGroup.external(name: 'Nhóm', externalEmails: emails(1)),
      ]);
      expect(codes(plan), ['duplicate-group-name']);
    });

    test('email sai dạng bị báo, ở cả hai loại nhóm', () {
      const plan = TesterGroups([
        TesterGroup.internal(
          name: 'In',
          members: {'khong-co-a-cong': AscRole.admin},
        ),
        TesterGroup.external(
          name: 'Ex',
          externalEmails: {'thieu@mien', 'co khoang@trang.example'},
        ),
      ]);
      expect(
        codes(plan).where((c) => c == 'tester-email-invalid'),
        hasLength(3),
      );
    });
  });

  group('public link', () {
    TesterGroups withLink(PublicLink link) => TesterGroups([
      TesterGroup.internal(name: 'In', members: people(1)),
      TesterGroup.external(name: 'Ex', publicLink: link),
    ]);

    test('giới hạn từ 1 đến 10.000 là hợp lệ, ngoài đó là lỗi', () {
      for (final limit in [1, 100, 10000]) {
        expect(
          withLink(PublicLink.openToAnyone(testerLimit: limit)).validate(),
          isEmpty,
          reason: '$limit',
        );
      }
      for (final limit in [0, -5, 10001]) {
        expect(codes(withLink(PublicLink.openToAnyone(testerLimit: limit))), [
          'public-link-limit-range',
        ], reason: '$limit');
      }
    });

    test('không đặt giới hạn cũng được', () {
      expect(withLink(const PublicLink.openToAnyone()).validate(), isEmpty);
    });

    test('link lọc theo tiêu chí phải có ít nhất một tiêu chí', () {
      expect(codes(withLink(const PublicLink.filtered(criteria: []))), [
        'public-link-no-criteria',
      ]);
      expect(
        withLink(const PublicLink.filtered(criteria: ['iPhone, iOS 17.0+']))
            .validate(),
        isEmpty,
      );
    });
  });
}
