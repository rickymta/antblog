import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'clock.dart';
import 'confirm_trip_screen.dart';
import 'quote.dart';

/// Tên route theo kiểu Navigator 1.0: chỉ là chuỗi.
abstract final class AppRoutes {
  static const quote = '/';
  static const history = '/history';
  static const confirm = '/confirm';
}

/// Dựng route cho tên có tham số. Tên không nhận ra thì trả `null` để
/// `Navigator` chuyển sang `onUnknownRoute`.
///
/// `arguments` là `Object?`: compiler không biết `/confirm` cần `Quote`, nên
/// kiểm tra phải làm lúc chạy.
Route<dynamic>? onGenerateAppRoute(
  RouteSettings settings, {
  Clock clock = systemClock,
}) {
  if (settings.name != AppRoutes.confirm) return null;
  final arguments = settings.arguments;
  if (arguments is! Quote) {
    // Cùng tên route thì mọi nhánh phải cùng kiểu kết quả `Trip`, kể cả
    // trang lỗi: `pushNamed<Trip>` ép kiểu route lúc chạy.
    return MaterialPageRoute<Trip>(
      settings: settings,
      builder: (_) => RouteErrorScreen(
        message:
            '${settings.name} cần một Quote, nhận ${arguments.runtimeType}',
      ),
    );
  }
  return MaterialPageRoute<Trip>(
    settings: settings,
    builder: (_) => ConfirmTripScreen(quote: arguments, clock: clock),
  );
}

/// Màn hình báo lỗi cho route sai tham số hoặc không tồn tại.
class RouteErrorScreen extends StatelessWidget {
  const RouteErrorScreen({super.key, required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Không mở được màn hình')),
      body: Center(child: Text(message)),
    );
  }
}

/// Chỗ giữ chỗ cho lịch sử chuyến (bài 09 làm danh sách thật).
class HistoryPlaceholderScreen extends StatelessWidget {
  const HistoryPlaceholderScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Lịch sử chuyến')),
      body: const Center(child: Text('Chưa có chuyến nào')),
    );
  }
}
