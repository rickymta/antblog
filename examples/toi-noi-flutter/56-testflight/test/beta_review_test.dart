import 'package:test/test.dart';
import 'package:toi_noi_56/toi_noi_56.dart';

BetaSubmission sub(
  DateTime at, {
  String version = '1.4.0',
  String build = '40',
  BuildStatus status = BuildStatus.testing,
}) => BetaSubmission(
  version: version,
  build: build,
  submittedAt: at,
  status: status,
);

void main() {
  final now = DateTime.utc(2026, 11, 7, 12);

  group('expectedReview', () {
    test('nhóm internal không cần review dù là bản đầu', () {
      expect(
        expectedReview(
          kind: GroupKind.internal,
          version: '1.4.0',
          versionsAlreadySubmitted: {},
        ),
        ReviewExpectation.notRequired,
      );
    });

    test('nhóm external: bản đầu của version cần review đầy đủ', () {
      expect(
        expectedReview(
          kind: GroupKind.external,
          version: '1.4.0',
          versionsAlreadySubmitted: {'1.3.0'},
        ),
        ReviewExpectation.full,
      );
    });

    test('nhóm external: bản sau của cùng version chỉ là "có thể" nhẹ hơn', () {
      expect(
        expectedReview(
          kind: GroupKind.external,
          version: '1.4.0',
          versionsAlreadySubmitted: {'1.4.0'},
        ),
        ReviewExpectation.mayBeLighter,
      );
    });
  });

  group('checkBetaSubmission: sáu build trong 24 giờ', () {
    test('không có lịch sử thì được gửi', () {
      expect(
        checkBetaSubmission(now: now, version: '1.4.0', history: []).isAllowed,
        isTrue,
      );
    });

    List<BetaSubmission> history(
      int n, {
      Duration spacing = const Duration(hours: 1),
    }) => [
      for (var i = 0; i < n; i++)
        sub(
          now.subtract(spacing * (i + 1)),
          version: '1.${3 - i}.0',
          build: '${30 + i}',
        ),
    ];

    test('năm lần trong 24 giờ qua vẫn được gửi lần thứ sáu', () {
      expect(
        checkBetaSubmission(
          now: now,
          version: '1.4.0',
          history: history(5),
        ).isAllowed,
        isTrue,
      );
    });

    test('sáu lần trong 24 giờ qua thì chặn, và nói khi nào thử lại', () {
      final h = history(6); // cũ nhất cách 6 giờ
      final d = checkBetaSubmission(now: now, version: '1.4.0', history: h);
      expect(d.isAllowed, isFalse);
      expect(
        d.retryAt,
        now.subtract(const Duration(hours: 6)).add(const Duration(hours: 24)),
      );
      expect(d.reasons.single, contains('6'));
    });

    test('mốc đúng 24 giờ: lần gửi cũ không còn được tính', () {
      final h = [
        sub(
          now.subtract(const Duration(hours: 24)),
        ), // đúng biên, ra khỏi cửa sổ
        for (var i = 1; i <= 5; i++) sub(now.subtract(Duration(hours: i))),
      ];
      expect(
        checkBetaSubmission(now: now, version: '1.4.0', history: h).isAllowed,
        isTrue,
      );
    });

    test('một giây trước biên thì vẫn tính và chặn', () {
      final h = [
        sub(
          now
              .subtract(const Duration(hours: 24))
              .add(const Duration(seconds: 1)),
        ),
        for (var i = 1; i <= 5; i++) sub(now.subtract(Duration(hours: i))),
      ];
      final d = checkBetaSubmission(now: now, version: '1.4.0', history: h);
      expect(d.isAllowed, isFalse);
      expect(d.retryAt, now.add(const Duration(seconds: 1)));
    });

    test('nhiều hơn sáu: thời điểm thử lại phải đủ để còn dưới giới hạn', () {
      // 8 lần, cách nhau 1 giờ (từ 1 đến 8 giờ trước). Muốn còn 5 lần trong
      // cửa sổ, ba lần cũ nhất phải ra khỏi đó; lần ra sau cùng là lần cách
      // 6 giờ, nên mốc thử lại là 6 giờ trước cộng 24 giờ.
      final d = checkBetaSubmission(
        now: now,
        version: '1.4.0',
        history: history(8),
      );
      expect(
        d.retryAt,
        now.subtract(const Duration(hours: 6)).add(const Duration(hours: 24)),
      );
      final retry = d.retryAt!;
      final h = history(8);
      expect(
        checkBetaSubmission(now: retry, version: '1.4.0', history: h).isAllowed,
        isTrue,
        reason: 'đúng mốc thì gửi được',
      );
      expect(
        checkBetaSubmission(
          now: retry.subtract(const Duration(seconds: 1)),
          version: '1.4.0',
          history: h,
        ).isAllowed,
        isFalse,
        reason: 'sớm một giây thì chưa',
      );
    });
  });

  group('checkBetaSubmission: một build mỗi version đang review', () {
    for (final status in [
      BuildStatus.waitingForReview,
      BuildStatus.inBetaReview,
    ]) {
      test('chặn khi cùng version đang ${status.label}', () {
        final d = checkBetaSubmission(
          now: now,
          version: '1.4.0',
          history: [sub(now.subtract(const Duration(days: 2)), status: status)],
        );
        expect(d.isAllowed, isFalse);
        expect(
          d.retryAt,
          isNull,
          reason: 'phải chờ review, không phải chờ giờ',
        );
        expect(d.reasons.single, contains('1.4.0'));
      });
    }

    test('version khác đang review thì không chặn', () {
      final d = checkBetaSubmission(
        now: now,
        version: '1.5.0',
        history: [
          sub(
            now.subtract(const Duration(hours: 2)),
            status: BuildStatus.inBetaReview,
          ),
        ],
      );
      expect(d.isAllowed, isTrue);
    });

    test('cùng version nhưng build cũ đã được duyệt thì không chặn', () {
      final d = checkBetaSubmission(
        now: now,
        version: '1.4.0',
        history: [sub(now.subtract(const Duration(hours: 2)))],
      );
      expect(d.isAllowed, isTrue);
    });

    test('vi phạm cả hai quy tắc thì báo cả hai lý do', () {
      final h = [
        for (var i = 1; i <= 5; i++) sub(now.subtract(Duration(hours: i))),
        sub(
          now.subtract(const Duration(hours: 6)),
          status: BuildStatus.waitingForReview,
        ),
      ];
      final d = checkBetaSubmission(now: now, version: '1.4.0', history: h);
      expect(d.reasons, hasLength(2));
      expect(d.retryAt, isNotNull);
    });
  });

  group('BuildExpiry (90 ngày)', () {
    final uploaded = DateTime.utc(2026, 8, 9, 8);

    test('hạn là đúng 90 ngày sau mốc bắt đầu', () {
      expect(
        BuildExpiry.expiresAt(uploaded),
        uploaded.add(const Duration(days: 90)),
      );
      expect(BuildExpiry.expiresAt(uploaded), DateTime.utc(2026, 11, 7, 8));
    });

    test('ngày tải lên còn đủ 90 ngày', () {
      expect(BuildExpiry.daysLeft(uploaded, uploaded), 90);
    });

    test('làm tròn lên: còn 1 giờ vẫn là còn 1 ngày', () {
      final now = DateTime.utc(2026, 11, 7, 7);
      expect(BuildExpiry.daysLeft(uploaded, now), 1);
      expect(BuildExpiry.isExpired(uploaded, now), isFalse);
    });

    test('còn dưới một giờ vẫn chưa hết hạn: tính là 1 ngày', () {
      for (final left in [
        const Duration(minutes: 30),
        const Duration(seconds: 1),
      ]) {
        final now = BuildExpiry.expiresAt(uploaded).subtract(left);
        expect(BuildExpiry.daysLeft(uploaded, now), 1, reason: '$left');
        expect(BuildExpiry.isExpired(uploaded, now), isFalse, reason: '$left');
      }
    });

    test('đúng thời điểm hết hạn và sau đó là hết: 0 ngày', () {
      expect(BuildExpiry.daysLeft(uploaded, DateTime.utc(2026, 11, 7, 8)), 0);
      expect(BuildExpiry.daysLeft(uploaded, DateTime.utc(2027, 1, 1)), 0);
      expect(
        BuildExpiry.isExpired(uploaded, DateTime.utc(2026, 11, 7, 8)),
        isTrue,
      );
    });

    test('sắp hết hạn: còn từ 1 đến 14 ngày, mặc định', () {
      bool soon(DateTime now) => BuildExpiry.isExpiringSoon(uploaded, now);
      expect(
        soon(uploaded.add(const Duration(days: 75))),
        isFalse,
        reason: 'còn 15 ngày: chưa',
      );
      expect(
        soon(uploaded.add(const Duration(days: 76))),
        isTrue,
        reason: 'còn 14',
      );
      expect(
        soon(uploaded.add(const Duration(days: 89))),
        isTrue,
        reason: 'còn 1',
      );
      expect(
        soon(uploaded.add(const Duration(days: 90))),
        isFalse,
        reason: 'đã hết',
      );
      expect(soon(uploaded.add(const Duration(days: 10))), isFalse);
    });

    test('ngưỡng cảnh báo tuỳ chỉnh', () {
      final now = uploaded.add(const Duration(days: 60)); // còn 30 ngày
      expect(BuildExpiry.isExpiringSoon(uploaded, now), isFalse);
      expect(BuildExpiry.isExpiringSoon(uploaded, now, warnDays: 30), isTrue);
    });
  });
}
