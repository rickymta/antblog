import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_17/fare_readers.dart';
import 'package:toi_noi_17/quote_draft.dart';
import 'package:toi_noi_17/quote_scope.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'support.dart';

void main() {
  group('QuoteScope.of / maybeOf / read', () {
    testWidgets('thiếu scope: maybeOf trả null, of và read ném FlutterError', (
      tester,
    ) async {
      QuoteDraft? maybe;
      late BuildContext captured;

      await tester.pumpWidget(
        Builder(
          builder: (context) {
            captured = context;
            maybe = QuoteScope.maybeOf(context);
            return const SizedBox();
          },
        ),
      );

      expect(maybe, isNull);
      expect(
        () => QuoteScope.of(captured),
        throwsA(
          isA<FlutterError>().having(
            (e) => e.message,
            'message',
            contains('Không tìm thấy QuoteScope'),
          ),
        ),
      );
      expect(() => QuoteScope.read(captured), throwsFlutterError);
    });

    testWidgets('of đăng ký phụ thuộc, read thì không', (tester) async {
      final draft = QuoteDraft();
      addTearDown(draft.dispose);
      var ofBuilds = 0;
      var readBuilds = 0;

      await tester.pumpWidget(
        QuoteScope(
          draft: draft,
          child: Column(
            children: [
              Probe(
                onBuild: () => ofBuilds++,
                read: (c) => '${QuoteScope.of(c).service.name}-of',
              ),
              Probe(
                onBuild: () => readBuilds++,
                read: (c) => '${QuoteScope.read(c).service.name}-read',
              ),
            ],
          ),
        ),
      );
      expect((ofBuilds, readBuilds), (1, 1));

      draft.selectService(ServiceKind.food);
      await tester.pump();

      expect(ofBuilds, 2);
      expect(readBuilds, 1);
      expect(find.text('food-of'), findsOneWidget);
      // Chữ của widget `read` vẫn là dữ liệu cũ vì nó không được dựng lại.
      expect(find.text('ride-read'), findsOneWidget);
    });

    testWidgets(
      'nhiều thay đổi trước một khung hình: dependent dựng lại một lần',
      (tester) async {
        final draft = QuoteDraft();
        addTearDown(draft.dispose);
        var builds = 0;

        await tester.pumpWidget(
          QuoteScope(
            draft: draft,
            child: Probe(
              onBuild: () => builds++,
              read: (c) => QuoteScope.of(c).fare.toString(),
            ),
          ),
        );

        draft
          ..selectService(ServiceKind.parcel)
          ..setKm(3)
          ..setKm(10);
        await tester.pump();

        expect(builds, 2); // 1 lần đầu + 1 lần cho cả ba thay đổi
        expect(find.text('12.000 VND'), findsNothing);
        expect(find.text(calculateFare(10).toString()), findsOneWidget);
      },
    );

    testWidgets('gỡ scope khỏi cây thì draft không còn bị scope nghe', (
      tester,
    ) async {
      final draft = ObservableDraft();
      addTearDown(draft.dispose);

      await tester.pumpWidget(
        QuoteScope(draft: draft, child: const SizedBox()),
      );
      expect(draft.isListened, isTrue);

      await tester.pumpWidget(const SizedBox());
      expect(draft.isListened, isFalse);
    });

    testWidgets('đổi sang draft khác: of thấy ngay, read giữ bản cũ', (
      tester,
    ) async {
      final first = QuoteDraft(km: 5);
      final second = QuoteDraft(km: 10);
      addTearDown(first.dispose);
      addTearDown(second.dispose);
      late StateSetter rebuild;
      var current = first;
      // Cùng một đối tượng ở mọi lần dựng: chỉ dependent mới bị đánh thức.
      const body = Column(children: [FareFromScope(), FareFromValue()]);

      await tester.pumpWidget(
        MaterialApp(
          home: StatefulBuilder(
            builder: (context, setState) {
              rebuild = setState;
              return QuoteScope(draft: current, child: body);
            },
          ),
        ),
      );
      expect(find.text('25.500 VND'), findsNWidgets(2));

      rebuild(() => current = second);
      await tester.pump();

      // `of`: dựng lại vì danh tính notifier đổi.
      expect(find.text(calculateFare(10).toString()), findsOneWidget);
      // `read`: vẫn nghe fareListenable của draft cũ.
      expect(find.text('25.500 VND'), findsOneWidget);
    });
  });

  group('vị trí đặt scope', () {
    Widget app({required bool aboveMaterialApp, required QuoteDraft draft}) {
      final home = Builder(
        builder: (context) => TextButton(
          onPressed: () => Navigator.of(context).push(
            MaterialPageRoute<void>(
              builder: (routeContext) =>
                  Text(QuoteScope.of(routeContext).fare.toString()),
            ),
          ),
          child: const Text('mở'),
        ),
      );
      return aboveMaterialApp
          ? QuoteScope(
              draft: draft,
              child: MaterialApp(home: home),
            )
          : MaterialApp(
              home: QuoteScope(draft: draft, child: home),
            );
    }

    testWidgets('scope trên MaterialApp: route được đẩy thấy scope', (
      tester,
    ) async {
      final draft = QuoteDraft();
      addTearDown(draft.dispose);
      await tester.pumpWidget(app(aboveMaterialApp: true, draft: draft));

      await tester.tap(find.text('mở'));
      await tester.pumpAndSettle();

      expect(find.text('25.500 VND'), findsOneWidget);
    });

    testWidgets(
      'scope trong home: route được đẩy không thấy, ném FlutterError',
      (tester) async {
        final draft = QuoteDraft();
        addTearDown(draft.dispose);
        await tester.pumpWidget(app(aboveMaterialApp: false, draft: draft));

        await tester.tap(find.text('mở'));
        await tester.pumpAndSettle();

        expect(tester.takeException(), isFlutterError);
      },
    );
  });

  group('chỗ gọi và vòng đời', () {
    testWidgets(
      'of trong initState: FlutterError (chế độ debug); read thì được',
      (tester) async {
        final draft = QuoteDraft();
        addTearDown(draft.dispose);

        await tester.pumpWidget(
          QuoteScope(draft: draft, child: const _InitStateReader(useOf: true)),
        );
        expect(
          tester.takeException(),
          isA<FlutterError>().having(
            (e) => e.toString(),
            'message',
            contains('before _InitStateReaderState.initState() completed'),
          ),
        );

        await tester.pumpWidget(
          QuoteScope(draft: draft, child: const _InitStateReader(useOf: false)),
        );
        expect(tester.takeException(), isNull);
        expect(find.text('ride'), findsOneWidget);
      },
    );

    testWidgets(
      'tạo QuoteDraft ngay trong build: mỗi lần dựng lại là một draft mới',
      (tester) async {
        final created = <QuoteDraft>[];
        addTearDown(() {
          for (final d in created) {
            d.dispose();
          }
        });
        late StateSetter rebuild;

        await tester.pumpWidget(
          StatefulBuilder(
            builder: (context, setState) {
              rebuild = setState;
              final draft = QuoteDraft(); // sai: tạo trong build
              created.add(draft);
              return QuoteScope(
                draft: draft,
                child: Probe(
                  onBuild: () {},
                  read: (c) => QuoteScope.of(c).service.name,
                ),
              );
            },
          ),
        );
        created.single.selectService(ServiceKind.food);
        await tester.pump();
        expect(find.text('food'), findsOneWidget);

        rebuild(() {});
        await tester.pump();

        // Lựa chọn của người dùng biến mất vì đã có một draft khác.
        expect(created, hasLength(2));
        expect(find.text('ride'), findsOneWidget);
      },
    );
  });
}

class _InitStateReader extends StatefulWidget {
  const _InitStateReader({required this.useOf});

  final bool useOf;

  @override
  State<_InitStateReader> createState() => _InitStateReaderState();
}

class _InitStateReaderState extends State<_InitStateReader> {
  late final String _service;

  @override
  void initState() {
    super.initState();
    final draft = widget.useOf
        ? QuoteScope.of(context)
        : QuoteScope.read(context);
    _service = draft.service.name;
  }

  @override
  Widget build(BuildContext context) =>
      Text(_service, textDirection: TextDirection.ltr);
}
