import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Màn hình báo giá: hiện quãng đường và cước tính từ [calculateFare].
///
/// `StatelessWidget`: mọi thứ nó hiển thị suy ra từ [distanceKm], nên không
/// cần `State`. Muốn báo giá khác thì parent dựng một `QuoteScreen` mới.
class QuoteScreen extends StatelessWidget {
  const QuoteScreen({super.key, required this.distanceKm});

  /// Quãng đường cần báo giá, đơn vị km.
  final double distanceKm;

  /// Quãng đường nằm trong miền bảng giá chấp nhận (0..100 km). Viết dạng
  /// "điều kiện hợp lệ" để NaN cũng bị loại.
  bool get _isInRange => distanceKm >= 0 && distanceKm <= FarePolicy.maxKm;

  @override
  Widget build(BuildContext context) {
    final textTheme = Theme.of(context).textTheme;
    final colorScheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(title: const Text('Tới Nơi')),
      body: Padding(
        key: const ValueKey('quote-body'),
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.directions_car, size: 48),
            const SizedBox(height: 8),
            Text(
              'Quãng đường: ${formatKm(distanceKm)} km',
              style: textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            if (_isInRange)
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: colorScheme.primaryContainer,
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Text(
                  calculateFare(distanceKm).toString(),
                  style: textTheme.headlineMedium,
                ),
              )
            else
              Text(
                'Quãng đường phải từ 0 đến ${FarePolicy.maxKm} km',
                style: textTheme.bodyMedium?.copyWith(color: colorScheme.error),
              ),
            const SizedBox(height: 8),
            const Text('Cước minh hoạ, không phải bảng giá thực tế'),
          ],
        ),
      ),
    );
  }
}

/// Hiển thị km theo kiểu Việt Nam: số nguyên bỏ phần thập phân, số lẻ dùng
/// dấu phẩy (`5` thành "5", `2.001` thành "2,001"). NaN và vô cực giữ
/// nguyên chữ của Dart.
///
/// Hàm này gọn nhưng chưa theo locale; bài về `intl` sẽ thay nó.
String formatKm(double km) {
  if (km.isFinite && km == km.truncateToDouble()) {
    return km.toInt().toString();
  }
  return km.toString().replaceAll('.', ',');
}
