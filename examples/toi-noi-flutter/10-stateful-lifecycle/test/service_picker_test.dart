import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_10/service_picker.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Đếm số lần `build` theo tên. `const Probe('x')` là cùng một instance qua
/// các lần cha dựng lại, nên framework bỏ qua nó.
final Map<String, int> _builds = {};

class Probe extends StatelessWidget {
  const Probe(this.name, {super.key, this.tag});

  final String name;

  /// Giá trị đổi theo từng lần cha dựng lại, khiến instance không còn là const.
  final Object? tag;

  @override
  Widget build(BuildContext context) {
    _builds[name] = (_builds[name] ?? 0) + 1;
    return Text('probe $name');
  }
}

/// Widget cha có state riêng để ép nó dựng lại mà không đụng ServicePicker.
class _Parent extends StatefulWidget {
  const _Parent({required this.initial});

  final ServiceKind initial;

  @override
  State<_Parent> createState() => _ParentState();
}

class _ParentState extends State<_Parent> {
  var _initial = ServiceKind.ride;
  var _rebuilds = 0;

  @override
  void initState() {
    super.initState();
    _initial = widget.initial;
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        ServicePicker(
          initial: _initial,
          summaryBuilder: (context, selected) {
            _builds['summary'] = (_builds['summary'] ?? 0) + 1;
            return Text('tóm tắt: ${selected.name}');
          },
        ),
        const Probe('const'),
        Probe('fresh', tag: _rebuilds),
        TextButton(
          onPressed: () => setState(() => _rebuilds++),
          child: const Text('dựng lại cha'),
        ),
        TextButton(
          onPressed: () => setState(() => _initial = ServiceKind.parcel),
          child: const Text('đổi initial'),
        ),
      ],
    );
  }
}

Widget _app(Widget child) => MaterialApp(home: Scaffold(body: child));

void main() {
  setUp(_builds.clear);

  group('ServicePicker', () {
    testWidgets('hiện ba dịch vụ và mặc định chọn Đặt xe', (tester) async {
      await tester.pumpWidget(_app(const ServicePicker()));

      for (final kind in ServiceKind.values) {
        expect(find.text(kind.label), findsOneWidget);
      }
      expect(find.text('Dịch vụ: Đặt xe'), findsOneWidget);
    });

    testWidgets('chạm một dịch vụ đổi lựa chọn và gọi onChanged', (
      tester,
    ) async {
      final picked = <ServiceKind>[];
      await tester.pumpWidget(_app(ServicePicker(onChanged: picked.add)));

      await tester.tap(find.text('Giao đồ ăn'));
      await tester.pump();

      expect(find.text('Dịch vụ: Giao đồ ăn'), findsOneWidget);
      expect(picked, [ServiceKind.food]);
    });

    testWidgets('chạm lại dịch vụ đang chọn không gọi onChanged', (
      tester,
    ) async {
      final picked = <ServiceKind>[];
      await tester.pumpWidget(_app(ServicePicker(onChanged: picked.add)));

      await tester.tap(find.text('Đặt xe'));
      await tester.pump();

      expect(picked, isEmpty);
      expect(find.text('Dịch vụ: Đặt xe'), findsOneWidget);
    });

    testWidgets('initial quyết định lựa chọn ban đầu', (tester) async {
      await tester.pumpWidget(
        _app(const ServicePicker(initial: ServiceKind.parcel)),
      );

      expect(find.text('Dịch vụ: Giao bưu kiện'), findsOneWidget);
    });

    testWidgets('vừa khít màn hình hẹp 320 px, không tràn', (tester) async {
      tester.view.physicalSize = const Size(320, 640);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);

      await tester.pumpWidget(_app(const ServicePicker()));

      expect(tester.takeException(), isNull);
    });

    group('phạm vi rebuild', () {
      testWidgets('setState chỉ dựng lại cây con của chính state đó', (
        tester,
      ) async {
        await tester.pumpWidget(
          _app(
            Column(
              children: [
                ServicePicker(
                  summaryBuilder: (context, selected) {
                    _builds['summary'] = (_builds['summary'] ?? 0) + 1;
                    return Text(selected.name);
                  },
                ),
                const Probe('anh em'),
              ],
            ),
          ),
        );
        expect(_builds, {'summary': 1, 'anh em': 1});

        await tester.tap(find.text('Giao đồ ăn'));
        await tester.pump();

        // build của state chạy lại; widget anh em và cha không bị đụng tới.
        expect(_builds, {'summary': 2, 'anh em': 1});
      });

      testWidgets('cha dựng lại thì picker dựng lại, const thì không', (
        tester,
      ) async {
        await tester.pumpWidget(_app(const _Parent(initial: ServiceKind.ride)));
        expect(_builds, {'summary': 1, 'const': 1, 'fresh': 1});

        await tester.tap(find.text('dựng lại cha'));
        await tester.pump();

        expect(_builds['summary'], 2);
        expect(_builds['fresh'], 2);
        expect(_builds['const'], 1);
      });

      testWidgets('đổi initial ở cha không đổi lựa chọn đã có', (tester) async {
        await tester.pumpWidget(_app(const _Parent(initial: ServiceKind.ride)));
        await tester.tap(find.text('Giao đồ ăn'));
        await tester.pump();

        await tester.tap(find.text('đổi initial'));
        await tester.pump();

        // Picker đã được dựng lại (summary chạy thêm) nhưng initState không
        // chạy lần nữa nên _selected vẫn là food.
        expect(find.text('tóm tắt: food'), findsOneWidget);
      });
    });
  });
}
