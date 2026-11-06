import 'package:test/test.dart';
import 'package:toi_noi_56/toi_noi_56.dart';

void main() {
  test('đúng mười hai trạng thái trong trang App build statuses của Apple', () {
    expect(BuildStatus.values.map((s) => s.label), [
      'Invalid Binary',
      'Missing Compliance',
      'Waiting for Export Compliance Review',
      'In Compliance Review',
      'Ready to Submit',
      'Waiting for Review',
      'In Beta Review',
      'Not Available for Testing',
      'Expired',
      'Rejected',
      'Ready to Test',
      'Testing',
    ]);
  });

  test('màu theo chỉ báo của Apple: đỏ, vàng, xanh', () {
    Set<String> labels(StatusColor c) => {
      for (final s in BuildStatus.values)
        if (s.color == c) s.label,
    };
    expect(labels(StatusColor.red), {
      'Invalid Binary',
      'Not Available for Testing',
      'Expired',
      'Rejected',
    });
    expect(labels(StatusColor.green), {'Ready to Test', 'Testing'});
    expect(labels(StatusColor.yellow), hasLength(6));
  });

  test('mỗi trạng thái có lời khuyên, và nhãn không trùng nhau', () {
    for (final s in BuildStatus.values) {
      expect(s.advice.trim(), isNotEmpty, reason: s.label);
    }
    expect(
      BuildStatus.values.map((s) => s.label).toSet(),
      hasLength(BuildStatus.values.length),
    );
  });

  test('trạng thái chết: build không dùng lại được', () {
    expect(BuildStatus.values.where((s) => s.isDead), [
      BuildStatus.invalidBinary,
      BuildStatus.notAvailableForTesting,
      BuildStatus.expired,
      BuildStatus.rejected,
    ]);
  });

  test(
    'parse đọc đúng tên Apple, không phân biệt hoa thường và khoảng trắng',
    () {
      for (final s in BuildStatus.values) {
        expect(BuildStatus.parse(s.label), s);
      }
      expect(
        BuildStatus.parse('  missing compliance '),
        BuildStatus.missingCompliance,
      );
      expect(BuildStatus.parse('IN BETA REVIEW'), BuildStatus.inBetaReview);
    },
  );

  test('parse ném FormatException với tên lạ, kể cả tên không có trong trang Apple', () {
    for (final bad in [
      '',
      'Processing',
      'Missing  Compliance',
      'Ready',
      'Testing!',
    ]) {
      expect(
        () => BuildStatus.parse(bad),
        throwsA(isA<FormatException>()),
        reason: '"$bad"',
      );
    }
  });

  test('Missing Compliance chỉ cách làm: trả lời câu hỏi hoặc khai khoá Info.plist', () {
    final advice = BuildStatus.missingCompliance.advice;
    expect(advice, contains('ITSAppUsesNonExemptEncryption'));
  });
}
