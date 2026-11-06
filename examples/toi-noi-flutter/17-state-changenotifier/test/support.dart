import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_17/app.dart';
import 'package:toi_noi_17/quote_draft.dart';
import 'package:toi_noi_17/sample_trips.dart';

/// Giờ cố định để test không phụ thuộc đồng hồ thật.
DateTime fixedClock() => DateTime.utc(2026, 11, 6, 8, 30);

/// Dựng app đầy đủ với bốn chuyến mẫu.
Future<void> pumpApp(WidgetTester tester) {
  return tester.pumpWidget(
    ToiNoiApp(initialTrips: sampleTrips(), clock: fixedClock),
  );
}

/// Widget thăm dò: đếm số lần `build` rồi gọi [read] để lấy chữ hiển thị.
class Probe extends StatelessWidget {
  const Probe({super.key, required this.onBuild, required this.read});

  final VoidCallback onBuild;
  final String Function(BuildContext context) read;

  @override
  Widget build(BuildContext context) {
    onBuild();
    return Text(read(context), textDirection: TextDirection.ltr);
  }
}

/// `hasListeners` là `@protected`, nên test muốn hỏi "còn ai nghe không" phải
/// đi qua một lớp con.
class ObservableDraft extends QuoteDraft {
  bool get isListened => hasListeners;
}
