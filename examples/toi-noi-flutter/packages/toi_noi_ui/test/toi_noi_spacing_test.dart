import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

void main() {
  test('các mức khoảng cách tăng dần và là bội của 4', () {
    const steps = [
      ToiNoiSpacing.xs,
      ToiNoiSpacing.sm,
      ToiNoiSpacing.md,
      ToiNoiSpacing.lg,
      ToiNoiSpacing.xl,
    ];
    expect(steps, [4, 8, 16, 24, 32]);
    for (var i = 1; i < steps.length; i++) {
      expect(steps[i], greaterThan(steps[i - 1]));
      expect(steps[i] % 4, 0);
    }
  });
}
