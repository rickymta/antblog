import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_21/presentation/failure_text.dart';

void main() {
  final failures = <TripFailure>[
    const Unreachable(UnreachableReason.noConnection),
    const Unreachable(UnreachableReason.tooSlow),
    const Unreachable(UnreachableReason.insecure),
    const ServiceFault(supportCode: 'fake-000009'),
    for (final kind in RejectionKind.values) Rejected(kind),
    const Unreadable(),
  ];

  test('nút Thử lại theo đúng quyết định của domain, với mọi loại lỗi', () {
    for (final failure in failures) {
      expect(
        describeFailure(failure).canRetry,
        failure.canRetryByUser,
        reason: '$failure',
      );
    }
  });

  test(
    'mỗi loại lỗi có một câu riêng, tiếng Việt, không lộ chi tiết kỹ thuật',
    () {
      final messages = [for (final f in failures) describeFailure(f).message];
      expect(messages.toSet(), hasLength(failures.length));
      for (final message in messages) {
        expect(message, isNot(contains('Dio')));
        expect(message, isNot(contains('HTTP')));
        expect(message, isNot(contains('500')));
      }
    },
  );

  test('mã hỗ trợ đi theo lỗi', () {
    expect(
      describeFailure(const ServiceFault(supportCode: 'fake-000009'))
          .supportCode,
      'fake-000009',
    );
    expect(
      describeFailure(const Unreachable(UnreachableReason.tooSlow)).supportCode,
      isNull,
    );
  });

  test('lỗi lập trình không có nút Thử lại', () {
    final info = describeFailure(StateError('hỏng'));
    expect(info.canRetry, isFalse);
    expect(info.message, 'Đã có lỗi không mong muốn.');
  });
}
