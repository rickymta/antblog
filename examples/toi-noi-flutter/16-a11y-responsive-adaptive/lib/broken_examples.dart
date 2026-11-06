import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Ba cách làm sai có chủ đích, mỗi cái vi phạm một guideline khác nhau.
/// Test dùng chúng để chứng minh guideline thật sự bắt được lỗi.

/// Icon chạm được nhưng bé (20) và không có nhãn: vi phạm cả
/// `androidTapTargetGuideline`, `iOSTapTargetGuideline` và
/// `labeledTapTargetGuideline`.
class UnlabeledTinyIcon extends StatelessWidget {
  const UnlabeledTinyIcon({super.key, required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) =>
      GestureDetector(onTap: onTap, child: const Icon(Icons.close, size: 20));
}

/// Vuông 44: đạt chuẩn iOS (44) nhưng không đạt chuẩn Android (48).
/// Có nhãn nên `labeledTapTargetGuideline` vẫn qua.
class Square44Button extends StatelessWidget {
  const Square44Button({super.key, required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: 44,
    child: Semantics(
      button: true,
      label: 'Đóng',
      child: InkWell(onTap: onTap, child: const Icon(Icons.close)),
    ),
  );
}

/// Bản sửa: `IconButton` có `tooltip` (cũng là nhãn semantics) và vùng chạm
/// mặc định 48.
class LabeledCloseButton extends StatelessWidget {
  const LabeledCloseButton({super.key, required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => IconButton(
    tooltip: 'Đóng',
    onPressed: onTap,
    icon: const Icon(Icons.close),
  );
}

/// `IconButton` quên `tooltip`: vùng chạm đủ lớn nhưng không có nhãn.
class IconButtonWithoutTooltip extends StatelessWidget {
  const IconButtonWithoutTooltip({super.key, required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) =>
      IconButton(onPressed: onTap, icon: const Icon(Icons.close));
}

/// Chữ xám nhạt trên nền trắng: tỉ lệ tương phản khoảng 1,9, thấp hơn nhiều
/// so với mức 4,5 của WCAG AA cho chữ thường.
class LowContrastNote extends StatelessWidget {
  const LowContrastNote({super.key});

  @override
  Widget build(BuildContext context) => const ColoredBox(
    color: Colors.white,
    child: Padding(
      padding: EdgeInsets.all(ToiNoiSpacing.md),
      child: Text(
        'Tài xế sẽ gọi khi tới điểm đón',
        style: TextStyle(color: Color(0xFFBBBBBB), fontSize: 16),
      ),
    ),
  );
}

/// Thẻ chuyến cao cố định 72: vừa ở cỡ chữ 1,0, tràn khi chữ phóng to.
class FixedHeightTripTile extends StatelessWidget {
  const FixedHeightTripTile({super.key, required this.trip});

  final Trip trip;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    return SizedBox(
      height: 72,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(trip.serviceKind.label, style: textTheme.titleMedium),
          Text(trip.id),
          MoneyText(trip.fare),
        ],
      ),
    );
  }
}

/// Nút ghim ở đáy mà quên `SafeArea`: nằm sát mép dưới vật lý của màn hình.
class UnsafeFooter extends StatelessWidget {
  const UnsafeFooter({super.key});

  @override
  Widget build(BuildContext context) => Scaffold(
    body: Column(
      children: [
        const Expanded(child: SizedBox.expand()),
        Padding(
          padding: const EdgeInsets.all(ToiNoiSpacing.md),
          child: FilledButton(
            onPressed: () {},
            child: const Text('Đặt chuyến'),
          ),
        ),
      ],
    ),
  );
}

/// Bản sửa của [UnsafeFooter]: cùng nút, bọc trong `SafeArea`.
class SafeFooter extends StatelessWidget {
  const SafeFooter({super.key});

  @override
  Widget build(BuildContext context) => Scaffold(
    body: Column(
      children: [
        const Expanded(child: SizedBox.expand()),
        SafeArea(
          top: false,
          child: Padding(
            padding: const EdgeInsets.all(ToiNoiSpacing.md),
            child: FilledButton(
              onPressed: () {},
              child: const Text('Đặt chuyến'),
            ),
          ),
        ),
      ],
    ),
  );
}
