import 'package:flutter/material.dart';

/// Cha ép chặt 300x300 (`SizedBox` truyền constraints chặt cho con).
/// Con đòi 100x100 nhưng không được nghe, trừ khi có `Align` nới lỏng.
class TightParentDemo extends StatelessWidget {
  const TightParentDemo({super.key, required this.loosen});

  /// `true`: chen `Align` giữa cha và con để con được tự chọn kích thước.
  final bool loosen;

  static const boxKey = Key('demo-box');

  @override
  Widget build(BuildContext context) {
    const box = ColoredBox(
      key: boxKey,
      color: Colors.teal,
      child: SizedBox(width: 100, height: 100),
    );
    return SizedBox(
      width: 300,
      height: 300,
      child: loosen ? const Align(child: box) : box,
    );
  }
}

/// Hàng 360 px: một ô cố định 60 px, hai ô `Expanded` flex 1 và flex 2.
class FlexRatioRow extends StatelessWidget {
  const FlexRatioRow({super.key});

  static const fixedKey = Key('flex-fixed');
  static const oneKey = Key('flex-1');
  static const twoKey = Key('flex-2');

  @override
  Widget build(BuildContext context) {
    return const SizedBox(
      width: 360,
      height: 48,
      child: Row(
        children: [
          SizedBox(key: fixedKey, width: 60, height: 48),
          Expanded(flex: 1, child: SizedBox(key: oneKey, height: 48)),
          Expanded(flex: 2, child: SizedBox(key: twoKey, height: 48)),
        ],
      ),
    );
  }
}

/// Hàng 300 px, hai con cùng flex 1 và cùng đòi rộng 50 px: `Flexible` (loose)
/// để con giữ 50 px, `Expanded` (tight) ép con phình ra đúng phần được cấp.
class FlexFitRow extends StatelessWidget {
  const FlexFitRow({super.key});

  static const flexibleKey = Key('fit-flexible');
  static const expandedKey = Key('fit-expanded');

  @override
  Widget build(BuildContext context) {
    return const SizedBox(
      width: 300,
      height: 48,
      child: Row(
        children: [
          Flexible(child: SizedBox(key: flexibleKey, width: 50, height: 48)),
          Expanded(child: SizedBox(key: expandedKey, width: 50, height: 48)),
        ],
      ),
    );
  }
}
