import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/misc.dart' show Override;

import 'app.dart';

/// Dựng widget gốc. `main()` gọi nó rồi `runApp`; test (kể cả test Patrol, vốn
/// không được gọi `runApp`) truyền [overrides] rồi `pumpWidget` cùng widget này.
Widget buildApp({List<Override> overrides = const []}) =>
    ProviderScope(overrides: overrides, child: const ToiNoiApp());
