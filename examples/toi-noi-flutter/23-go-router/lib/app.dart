import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'router.dart';

class ToiNoiApp extends ConsumerWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: 'Tới Nơi',
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      routerConfig: ref.watch(routerProvider),
    );
  }
}
