import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'accessible.dart';

/// Màn hình báo giá: chọn dịch vụ, kéo quãng đường, xem cước, đặt chuyến.
///
/// Nút đặt chuyến ghim ở đáy và nằm trong [SafeArea] để không chui xuống
/// vạch home indicator hay thanh điều hướng cử chỉ. Nội dung giới hạn bề rộng
/// 560 để trên màn hình rộng không bị kéo giãn.
class QuoteScreen extends StatefulWidget {
  const QuoteScreen({super.key, required this.onBook});

  final void Function(ServiceKind kind, double km) onBook;

  @override
  State<QuoteScreen> createState() => _QuoteScreenState();
}

class _QuoteScreenState extends State<QuoteScreen> {
  static const double _minKm = 1;
  static const double _maxKm = 20;

  ServiceKind _kind = ServiceKind.ride;
  double _km = 5;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    final fare = calculateFare(_km);
    return Align(
      alignment: Alignment.topCenter,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 560),
        child: Column(
          children: [
            Expanded(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(ToiNoiSpacing.md),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Chọn dịch vụ', style: textTheme.titleMedium),
                    const SizedBox(height: ToiNoiSpacing.sm),
                    Wrap(
                      spacing: ToiNoiSpacing.sm,
                      runSpacing: ToiNoiSpacing.sm,
                      children: [
                        for (final kind in ServiceKind.values)
                          ChoiceChip(
                            label: Text(kind.label),
                            selected: kind == _kind,
                            onSelected: (_) => setState(() => _kind = kind),
                          ),
                      ],
                    ),
                    const SizedBox(height: ToiNoiSpacing.lg),
                    // Chữ hiển thị giá trị hiện tại; semantics của nó tắt đi vì
                    // nút trượt bên dưới đã tự đọc "nhãn, giá trị".
                    ExcludeSemantics(
                      child: Text(
                        'Quãng đường: ${formatKm(_km)}',
                        style: textTheme.titleMedium,
                      ),
                    ),
                    // MergeSemantics gộp nhãn "Quãng đường" vào đúng nút
                    // trượt, thay vì để hai nút riêng. Slider tự có thao tác
                    // tăng/giảm; ta thêm câu đọc giá trị bằng tiếng Việt.
                    MergeSemantics(
                      child: Semantics(
                        label: 'Quãng đường',
                        child: Slider(
                          value: _km,
                          min: _minKm,
                          max: _maxKm,
                          divisions: (_maxKm - _minKm).round(),
                          semanticFormatterCallback: formatKm,
                          onChanged: (value) => setState(() => _km = value),
                        ),
                      ),
                    ),
                    const SizedBox(height: ToiNoiSpacing.md),
                    Card(
                      child: Padding(
                        padding: const EdgeInsets.all(ToiNoiSpacing.md),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text('Cước dự kiến', style: textTheme.labelLarge),
                            const SizedBox(height: ToiNoiSpacing.xs),
                            AccessibleMoney(
                              fare,
                              style: textTheme.headlineMedium,
                            ),
                          ],
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            SafeArea(
              top: false,
              child: Padding(
                padding: const EdgeInsets.all(ToiNoiSpacing.md),
                child: SizedBox(
                  width: double.infinity,
                  child: FilledButton(
                    onPressed: () => widget.onBook(_kind, _km),
                    child: const Text('Đặt chuyến'),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
