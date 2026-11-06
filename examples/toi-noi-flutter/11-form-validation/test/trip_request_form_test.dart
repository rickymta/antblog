import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_11/request_screen.dart';
import 'package:toi_noi_11/trip_request_data.dart';
import 'package:toi_noi_11/trip_request_form.dart';
import 'package:toi_noi_11/validators.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

const _pickupLabel = 'Điểm đón';
const _destinationLabel = 'Điểm đến';
const _distanceLabel = 'Khoảng cách (km)';

// TextFormField dựng TextField bên trong; tìm theo nhãn rồi lấy TextField.
Finder _field(String label) => find.widgetWithText(TextFormField, label);

TextField _textField(WidgetTester tester, String label) =>
    tester.widget<TextField>(
      find.descendant(of: _field(label), matching: find.byType(TextField)),
    );

FocusNode _focusOf(WidgetTester tester, String label) =>
    _textField(tester, label).focusNode!;

TextEditingController _controllerOf(WidgetTester tester, String label) =>
    _textField(tester, label).controller!;

Future<void> _pumpForm(
  WidgetTester tester,
  ValueChanged<TripRequestData> onSubmit,
) {
  return tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: SingleChildScrollView(child: TripRequestForm(onSubmit: onSubmit)),
      ),
    ),
  );
}

Future<void> _fill(
  WidgetTester tester, {
  String pickup = '12 Lê Lợi',
  String destination = 'Sân bay Tân Sơn Nhất',
  String distance = '5',
}) async {
  await tester.enterText(_field(_pickupLabel), pickup);
  await tester.enterText(_field(_destinationLabel), destination);
  await tester.enterText(_field(_distanceLabel), distance);
}

Future<void> _tapSubmit(WidgetTester tester) async {
  await tester.tap(find.text('Tìm tài xế'));
  await tester.pump();
}

void main() {
  group('validate hiện lỗi đúng', () {
    testWidgets('chưa chạm gì thì chưa có lỗi nào', (tester) async {
      await _pumpForm(tester, (_) {});
      expect(find.text('Nhập điểm đón.'), findsNothing);
      expect(find.text('Nhập điểm đến.'), findsNothing);
      expect(find.text('Nhập khoảng cách (km).'), findsNothing);
    });

    testWidgets('bấm gửi form rỗng: 3 lỗi, không gọi callback', (tester) async {
      var calls = 0;
      await _pumpForm(tester, (_) => calls++);

      await _tapSubmit(tester);

      expect(find.text('Nhập điểm đón.'), findsOneWidget);
      expect(find.text('Nhập điểm đến.'), findsOneWidget);
      expect(find.text('Nhập khoảng cách (km).'), findsOneWidget);
      expect(calls, 0);
    });

    testWidgets('focus nhảy tới ô lỗi đầu tiên', (tester) async {
      await _pumpForm(tester, (_) {});
      await _fill(tester, pickup: '12 Lê Lợi', destination: '', distance: 'x');

      await _tapSubmit(tester);

      expect(_focusOf(tester, _destinationLabel).hasPrimaryFocus, isTrue);
    });

    testWidgets('khoảng cách ngoài miền hiện câu có FarePolicy.maxKm', (
      tester,
    ) async {
      await _pumpForm(tester, (_) {});
      await _fill(tester, distance: '100,5');

      await _tapSubmit(tester);

      expect(
        find.text('Khoảng cách phải từ 0 đến ${FarePolicy.maxKm} km.'),
        findsOneWidget,
      );
    });

    testWidgets('điểm đến trùng điểm đón hiện lỗi ở ô điểm đến', (
      tester,
    ) async {
      var calls = 0;
      await _pumpForm(tester, (_) => calls++);
      await _fill(tester, pickup: '12 Lê Lợi', destination: ' 12  LÊ LỢI ');

      await _tapSubmit(tester);

      expect(find.text('Điểm đến trùng điểm đón.'), findsOneWidget);
      expect(calls, 0);
    });

    testWidgets('sửa ô đang lỗi thì lỗi biến mất, không cần bấm gửi lại', (
      tester,
    ) async {
      await _pumpForm(tester, (_) {});
      await _tapSubmit(tester);
      expect(find.text('Nhập điểm đón.'), findsOneWidget);

      await tester.enterText(_field(_pickupLabel), '12 Lê Lợi');
      await tester.pump();

      expect(find.text('Nhập điểm đón.'), findsNothing);
      // Các ô chưa sửa vẫn giữ lỗi.
      expect(find.text('Nhập điểm đến.'), findsOneWidget);
    });

    testWidgets('lỗi chéo ô: sửa điểm đón làm lỗi trùng ở điểm đến biến mất', (
      tester,
    ) async {
      await _pumpForm(tester, (_) {});
      await _fill(tester, pickup: '12 Lê Lợi', destination: '12 Lê Lợi');
      await _tapSubmit(tester);
      expect(find.text('Điểm đến trùng điểm đón.'), findsOneWidget);

      // Mỗi lần một ô đổi, Form dựng lại mọi ô; ô điểm đến đang có lỗi và
      // người dùng đã gõ vào nó nên tự kiểm lại với điểm đón mới.
      await tester.enterText(_field(_pickupLabel), '45 Nguyễn Huệ');
      await tester.pump();

      expect(find.text('Điểm đến trùng điểm đón.'), findsNothing);
    });

    testWidgets('chiều ngược lại: ô đang hợp lệ không tự báo lỗi chéo', (
      tester,
    ) async {
      var calls = 0;
      await _pumpForm(tester, (_) => calls++);
      await _fill(tester, pickup: '12 Lê Lợi', destination: 'Bến Thành');
      await _tapSubmit(tester);
      expect(calls, 1);

      // onUserInteractionIfError chỉ kiểm lại ô đang có lỗi, nên đổi điểm đón
      // thành trùng điểm đến chưa hiện lỗi cho tới khi bấm gửi.
      await tester.enterText(_field(_pickupLabel), 'Bến Thành');
      await tester.pump();
      expect(find.text('Điểm đến trùng điểm đón.'), findsNothing);

      await _tapSubmit(tester);
      expect(find.text('Điểm đến trùng điểm đón.'), findsOneWidget);
      expect(calls, 1);
    });

    testWidgets(
      'sửa hết lỗi rồi gõ sai lại: chưa báo lỗi cho tới lần gửi sau',
      (tester) async {
        var calls = 0;
        await _pumpForm(tester, (_) => calls++);
        await _tapSubmit(tester);
        expect(find.text('Nhập điểm đón.'), findsOneWidget);

        // Sửa đúng: lỗi biến mất, ô không còn lỗi.
        await tester.enterText(_field(_pickupLabel), '12 Lê Lợi');
        await tester.pump();
        expect(find.text('Nhập điểm đón.'), findsNothing);

        // Gõ sai lại: ô không có lỗi nên onUserInteractionIfError không kiểm.
        await tester.enterText(_field(_pickupLabel), 'a');
        await tester.pump();
        expect(find.textContaining('ít nhất'), findsNothing);
        await tester.enterText(_field(_pickupLabel), '');
        await tester.pump();
        expect(find.text('Nhập điểm đón.'), findsNothing);

        // Lần gửi sau mới bắt được.
        await _tapSubmit(tester);
        expect(find.text('Nhập điểm đón.'), findsOneWidget);
        expect(calls, 0);
      },
    );

    testWidgets('bộ lọc bàn phím chặn chữ cái ở ô khoảng cách', (tester) async {
      await _pumpForm(tester, (_) {});

      await tester.enterText(_field(_distanceLabel), '5a,5b');

      expect(_controllerOf(tester, _distanceLabel).text, '5,5');
    });

    testWidgets('dán chuỗi 1,2,3 vượt bộ lọc nhưng validator vẫn chặn', (
      tester,
    ) async {
      await _pumpForm(tester, (_) {});
      await _fill(tester, distance: '1,2,3');

      await _tapSubmit(tester);

      expect(_controllerOf(tester, _distanceLabel).text, '1,2,3');
      expect(find.text('Nhập số, ví dụ 5 hoặc 5,5.'), findsOneWidget);
    });
  });

  group('so sánh AutovalidateMode', () {
    Future<void> pumpField(WidgetTester tester, AutovalidateMode mode) {
      return tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Form(
              child: TextFormField(
                autovalidateMode: mode,
                validator: validatePickup,
              ),
            ),
          ),
        ),
      );
    }

    testWidgets('onUserInteraction báo lỗi ngay ở ký tự đầu tiên', (
      tester,
    ) async {
      await pumpField(tester, AutovalidateMode.onUserInteraction);
      await tester.enterText(find.byType(TextFormField), 'a');
      await tester.pump();
      expect(find.textContaining('ít nhất'), findsOneWidget);
    });

    testWidgets('onUserInteractionIfError chờ tới khi đã có lỗi mới kiểm lại', (
      tester,
    ) async {
      await pumpField(tester, AutovalidateMode.onUserInteractionIfError);
      await tester.enterText(find.byType(TextFormField), 'a');
      await tester.pump();
      expect(find.textContaining('ít nhất'), findsNothing);
    });

    testWidgets(
      'đặt ở cấp Form: gõ một ô là mọi ô bị kiểm, kể cả ô chưa chạm',
      (tester) async {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: Form(
                autovalidateMode: AutovalidateMode.onUserInteraction,
                child: Column(
                  children: [
                    TextFormField(
                      key: const Key('a'),
                      validator: (_) => 'lỗi a',
                    ),
                    TextFormField(
                      key: const Key('b'),
                      validator: (_) => 'lỗi b',
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
        expect(find.text('lỗi b'), findsNothing);

        await tester.enterText(find.byKey(const Key('a')), 'x');
        await tester.pump();

        expect(find.text('lỗi a'), findsOneWidget);
        expect(find.text('lỗi b'), findsOneWidget);
      },
    );

    testWidgets('disabled không bao giờ tự kiểm', (tester) async {
      await pumpField(tester, AutovalidateMode.disabled);
      await tester.enterText(find.byType(TextFormField), 'a');
      await tester.pump();
      expect(find.textContaining('ít nhất'), findsNothing);
    });
  });

  group('TextInputAction và focus', () {
    testWidgets('hai ô đầu dùng next, ô cuối dùng done', (tester) async {
      await _pumpForm(tester, (_) {});
      expect(
        _textField(tester, _pickupLabel).textInputAction,
        TextInputAction.next,
      );
      expect(
        _textField(tester, _destinationLabel).textInputAction,
        TextInputAction.next,
      );
      expect(
        _textField(tester, _distanceLabel).textInputAction,
        TextInputAction.done,
      );
    });

    testWidgets('next chuyển focus điểm đón -> điểm đến -> khoảng cách', (
      tester,
    ) async {
      await _pumpForm(tester, (_) {});

      await tester.tap(_field(_pickupLabel));
      await tester.pump();
      expect(_focusOf(tester, _pickupLabel).hasPrimaryFocus, isTrue);

      await tester.testTextInput.receiveAction(TextInputAction.next);
      await tester.pump();
      expect(_focusOf(tester, _destinationLabel).hasPrimaryFocus, isTrue);

      await tester.testTextInput.receiveAction(TextInputAction.next);
      await tester.pump();
      expect(_focusOf(tester, _distanceLabel).hasPrimaryFocus, isTrue);
    });

    testWidgets('next không kiểm hay gửi form', (tester) async {
      var calls = 0;
      await _pumpForm(tester, (_) => calls++);

      await tester.tap(_field(_pickupLabel));
      await tester.pump();
      await tester.testTextInput.receiveAction(TextInputAction.next);
      await tester.pump();

      expect(calls, 0);
      expect(find.text('Nhập điểm đón.'), findsNothing);
    });

    testWidgets('done ở ô cuối gửi form', (tester) async {
      final received = <TripRequestData>[];
      await _pumpForm(tester, received.add);
      await _fill(tester);

      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();

      expect(received, hasLength(1));
    });

    testWidgets('done khi dữ liệu sai thì hiện lỗi và không gọi callback', (
      tester,
    ) async {
      var calls = 0;
      await _pumpForm(tester, (_) => calls++);
      await tester.enterText(_field(_distanceLabel), '5');
      await tester.pump();

      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();

      expect(calls, 0);
      expect(find.text('Nhập điểm đón.'), findsOneWidget);
      // Return ở ô cuối đưa focus về ô lỗi đầu tiên, không phải ô vừa bấm.
      expect(_focusOf(tester, _pickupLabel).hasPrimaryFocus, isTrue);
    });
  });

  group('submit gọi callback với dữ liệu đã chuẩn hoá', () {
    testWidgets('gọn khoảng trắng và đổi dấu phẩy thành số thực', (
      tester,
    ) async {
      final received = <TripRequestData>[];
      await _pumpForm(tester, received.add);
      await _fill(
        tester,
        pickup: '  12   Lê  Lợi ',
        destination: ' Sân bay   Tân Sơn Nhất  ',
        distance: ' 2,001 ',
      );

      await _tapSubmit(tester);

      expect(received, [
        const TripRequestData(
          pickup: '12 Lê Lợi',
          destination: 'Sân bay Tân Sơn Nhất',
          distanceKm: 2.001,
        ),
      ]);
      expect(received.single.estimatedFare, calculateFare(2.001));
      expect(received.single.estimatedFare.vnd, 12005);
    });

    testWidgets('dấu chấm và dấu phẩy cho cùng kết quả', (tester) async {
      final received = <TripRequestData>[];
      await _pumpForm(tester, received.add);

      await _fill(tester, distance: '5,5');
      await _tapSubmit(tester);
      await _fill(tester, distance: '5.5');
      await _tapSubmit(tester);

      expect(received, hasLength(2));
      expect(received[0], received[1]);
      expect(received[0].distanceKm, 5.5);
    });

    testWidgets('biên 100 km được chấp nhận, cước 453.000 VND', (tester) async {
      final received = <TripRequestData>[];
      await _pumpForm(tester, received.add);
      await _fill(tester, distance: '100');

      await _tapSubmit(tester);

      expect(received.single.estimatedFare.vnd, 453000);
    });

    testWidgets('gửi hợp lệ thì hạ bàn phím (không ô nào còn focus)', (
      tester,
    ) async {
      await _pumpForm(tester, (_) {});
      await _fill(tester);
      await tester.tap(_field(_distanceLabel));
      await tester.pump();
      expect(_focusOf(tester, _distanceLabel).hasPrimaryFocus, isTrue);

      await _tapSubmit(tester);

      expect(_focusOf(tester, _distanceLabel).hasFocus, isFalse);
    });

    testWidgets('màn hình hiện cước đúng: 5 km là 25.500 VND', (tester) async {
      await tester.pumpWidget(const MaterialApp(home: RequestScreen()));
      await _fill(tester, pickup: '12 Lê Lợi', destination: 'Bến Thành');

      await _tapSubmit(tester);

      expect(find.textContaining('25.500 VND'), findsOneWidget);
    });
  });

  group('dispose', () {
    testWidgets('gỡ form khỏi cây thì controller và FocusNode bị dispose', (
      tester,
    ) async {
      await _pumpForm(tester, (_) {});
      final controllers = [
        for (final label in [_pickupLabel, _destinationLabel, _distanceLabel])
          _controllerOf(tester, label),
      ];
      final focusNodes = [
        for (final label in [_pickupLabel, _destinationLabel, _distanceLabel])
          _focusOf(tester, label),
      ];
      // Còn sống: thêm listener được.
      for (final c in [...controllers, ...focusNodes]) {
        c.addListener(() {});
      }

      await tester.pumpWidget(const SizedBox.shrink());

      // ChangeNotifier đã dispose thì addListener ném FlutterError (khi bật assert).
      for (final c in controllers) {
        expect(() => c.addListener(() {}), throwsFlutterError);
      }
      for (final f in focusNodes) {
        expect(() => f.addListener(() {}), throwsFlutterError);
      }
    });
  });

  group('bàn phím và resizeToAvoidBottomInset', () {
    const keyboard = 300.0;
    const screenHeight = 600.0;

    Future<void> setUpScreen(
      WidgetTester tester, {
      required bool resize,
    }) async {
      tester.view.devicePixelRatio = 1;
      tester.view.physicalSize = const Size(400, screenHeight);
      addTearDown(tester.view.reset);
      await tester.pumpWidget(
        MaterialApp(home: RequestScreen(resizeToAvoidBottomInset: resize)),
      );
    }

    Future<void> showKeyboard(WidgetTester tester) async {
      tester.view.viewInsets = const FakeViewPadding(bottom: keyboard);
      addTearDown(tester.view.resetViewInsets);
      await tester.pump();
    }

    double bodyInset(WidgetTester tester) =>
        MediaQuery.viewInsetsOf(tester.element(find.byType(TripRequestForm)))
            .bottom;

    double maxScroll(WidgetTester tester) => tester
        .state<ScrollableState>(find.byType(Scrollable).first)
        .position
        .maxScrollExtent;

    testWidgets('chưa có bàn phím: form vừa màn hình, không cần cuộn', (
      tester,
    ) async {
      await setUpScreen(tester, resize: true);
      expect(maxScroll(tester), 0);
      expect(
        tester.getRect(find.text('Tìm tài xế')).bottom,
        lessThan(screenHeight),
      );
    });

    testWidgets(
      'mặc định: Scaffold lấy phần bàn phím ra khỏi body, form cuộn',
      (tester) async {
        await setUpScreen(tester, resize: true);

        await showKeyboard(tester);

        expect(bodyInset(tester), 0);
        expect(maxScroll(tester), greaterThan(0));

        // Cuộn xuống thì nút gửi lên trên mép bàn phím.
        await tester.ensureVisible(find.text('Tìm tài xế'));
        await tester.pump();
        expect(
          tester.getRect(find.text('Tìm tài xế')).bottom,
          lessThanOrEqualTo(screenHeight - keyboard),
        );
      },
    );

    testWidgets(
      'resize false: body vẫn nhận bàn phím, không cuộn, nút bị che',
      (tester) async {
        await setUpScreen(tester, resize: false);

        await showKeyboard(tester);

        expect(bodyInset(tester), keyboard);
        expect(maxScroll(tester), 0);
        expect(
          tester.getRect(find.text('Tìm tài xế')).bottom,
          greaterThan(screenHeight - keyboard),
        );
      },
    );

    testWidgets('resize true: ô đang gõ được cuộn lên trên bàn phím', (
      tester,
    ) async {
      await setUpScreen(tester, resize: true);
      await tester.tap(_field(_distanceLabel));
      await tester.pump();

      await showKeyboard(tester);
      await tester.pumpAndSettle();

      expect(
        tester.getRect(_field(_distanceLabel)).bottom,
        lessThanOrEqualTo(screenHeight - keyboard),
      );
    });
  });

  group('cạm bẫy khác', () {
    testWidgets('initialValue cùng controller bị assert chặn', (tester) async {
      final controller = TextEditingController();
      addTearDown(controller.dispose);
      expect(
        () => TextFormField(initialValue: 'abc', controller: controller),
        throwsAssertionError,
      );
    });

    testWidgets('form không cuộn gặp bàn phím thì tràn RenderFlex', (
      tester,
    ) async {
      tester.view.devicePixelRatio = 1;
      tester.view.physicalSize = const Size(400, 500);
      tester.view.viewInsets = const FakeViewPadding(bottom: 300);
      addTearDown(tester.view.reset);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: TripRequestForm(onSubmit: (_) {})),
        ),
      );

      final error = tester.takeException();
      expect(error, isA<FlutterError>());
      expect(error.toString(), contains('overflowed'));
    });
  });

  group('cạm bẫy: Form trong danh sách dựng lười', () {
    testWidgets('ô nằm ngoài vùng dựng không được validate()', (tester) async {
      final formKey = GlobalKey<FormState>();
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Form(
              key: formKey,
              child: ListView(
                children: [
                  const SizedBox(height: 3000),
                  TextFormField(validator: (_) => 'luôn lỗi'),
                ],
              ),
            ),
          ),
        ),
      );

      // Ô lỗi chưa được dựng nên Form không biết nó tồn tại.
      expect(find.byType(TextFormField), findsNothing);
      expect(formKey.currentState!.validate(), isTrue);
    });

    testWidgets('cùng form đó trong SingleChildScrollView thì bắt được lỗi', (
      tester,
    ) async {
      final formKey = GlobalKey<FormState>();
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: Form(
                key: formKey,
                child: Column(
                  children: [
                    const SizedBox(height: 3000),
                    TextFormField(validator: (_) => 'luôn lỗi'),
                  ],
                ),
              ),
            ),
          ),
        ),
      );

      expect(formKey.currentState!.validate(), isFalse);
    });
  });
}
