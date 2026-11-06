import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'environment_screen.dart';
import 'providers.dart';

/// Vỏ ứng dụng. Mọi bản không phải `prod` có dải ruy-băng tên môi trường để
/// tester không nhầm bản staging với bản thật.
class ToiNoiApp extends ConsumerWidget {
  const ToiNoiApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(appConfigProvider);
    return MaterialApp(
      title: config.isProd ? 'Tới Nơi' : 'Tới Nơi ${config.flavor.name}',
      theme: ToiNoiTheme.light(),
      darkTheme: ToiNoiTheme.dark(),
      builder: (context, child) => config.isProd
          ? child!
          : Banner(
              message: config.flavor.name.toUpperCase(),
              location: BannerLocation.topStart,
              child: child,
            ),
      home: const EnvironmentScreen(),
    );
  }
}
