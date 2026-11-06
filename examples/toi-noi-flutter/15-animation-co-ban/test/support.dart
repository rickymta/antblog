import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Bọc [home] trong `MaterialApp` có theme Tới Nơi.
///
/// [reduceMotion] ghi đè `MediaQueryData.disableAnimations` dưới
/// `MaterialApp`. Cách này chỉ tác động vào code đọc `MediaQuery` (helper
/// `ToiNoiMotion`); `AnimationController` của framework đọc cờ nền tảng nên
/// không bị ảnh hưởng, nhờ vậy test chứng minh đúng phần việc của helper.
Widget harness(Widget home, {bool reduceMotion = false}) {
  return MaterialApp(
    theme: ToiNoiTheme.light(),
    builder: (context, child) => MediaQuery(
      data: MediaQuery.of(context).copyWith(disableAnimations: reduceMotion),
      child: child!,
    ),
    home: home,
  );
}

/// Widget giữ một giá trị `T` để test đổi giá trị rồi `pump`, thay vì dựng lại
/// cả `MaterialApp` bằng `pumpWidget` lần hai.
class Host<T> extends StatefulWidget {
  const Host({super.key, required this.initial, required this.builder});

  final T initial;
  final Widget Function(BuildContext context, T value) builder;

  static HostState<T> of<T>(WidgetTester tester) =>
      tester.state<HostState<T>>(find.byType(Host<T>));

  @override
  State<Host<T>> createState() => HostState<T>();
}

class HostState<T> extends State<Host<T>> {
  late T _value = widget.initial;

  T get value => _value;

  set value(T next) => setState(() => _value = next);

  @override
  Widget build(BuildContext context) => widget.builder(context, _value);
}
