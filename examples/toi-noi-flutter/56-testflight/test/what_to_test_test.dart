import 'package:test/test.dart';
import 'package:toi_noi_56/toi_noi_56.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

void main() {
  test('cước trong kịch bản lấy từ calculateFare: 5 km là 25.500 VND', () {
    final fare = calculateFare(5).toString();
    expect(fare, '25.500 VND');
    for (final role in AppRole.values) {
      expect(
        buildWhatToTest(role: role, buildName: '1.4.0', buildNumber: '43'),
        contains(fare),
        reason: role.name,
      );
    }
  });

  test('dòng đầu nói rõ bản và build đang test', () {
    final text = buildWhatToTest(
      role: AppRole.customer,
      buildName: '1.4.0',
      buildNumber: '43',
    );
    expect(text.split('\n').first, 'Bản 1.4.0 (build 43).');
  });

  test('app khách và app tài xế có việc khác nhau', () {
    final customer = buildWhatToTest(
      role: AppRole.customer,
      buildName: '1.4.0',
      buildNumber: '43',
    );
    final driver = buildWhatToTest(
      role: AppRole.driver,
      buildName: '1.4.0',
      buildNumber: '43',
    );
    expect(customer, contains('đặt một chuyến'));
    expect(driver, contains('app dẫn đường'));
    expect(customer, isNot(contains('app dẫn đường')));
    expect(driver, isNot(contains('đặt một chuyến')));
  });

  test('lỗi đã biết chỉ xuất hiện khi có', () {
    const known = ['Ảnh đại diện tài xế chưa tải được'];
    final withKnown = buildWhatToTest(
      role: AppRole.driver,
      buildName: '1.4.0',
      buildNumber: '43',
      knownIssues: known,
    );
    expect(withKnown, contains('Lỗi đã biết'));
    expect(withKnown, contains('- ${known.single}'));
    expect(
      buildWhatToTest(
        role: AppRole.driver,
        buildName: '1.4.0',
        buildNumber: '43',
      ),
      isNot(contains('Lỗi đã biết')),
    );
  });

  test('luôn nhắc cách báo lỗi và đánh số các bước từ 1', () {
    final text = buildWhatToTest(
      role: AppRole.customer,
      buildName: '1.4.0',
      buildNumber: '43',
    );
    expect(text, contains('1. '));
    expect(text, contains('3. '));
    expect(text, isNot(contains('4. ')));
    expect(text, endsWith('kèm ảnh và mô tả.'));
  });

  test('tên bản hoặc build rỗng ném ArgumentError', () {
    for (final (name, build) in [
      ('', '43'),
      ('  ', '43'),
      ('1.4.0', ''),
      ('1.4.0', ' '),
    ]) {
      expect(
        () => buildWhatToTest(
          role: AppRole.customer,
          buildName: name,
          buildNumber: build,
        ),
        throwsArgumentError,
        reason: '"$name" "$build"',
      );
    }
  });
}
