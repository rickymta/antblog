import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'providers.dart';

/// Cài đặt: chế độ sáng/tối (`shared_preferences`) và phiên đăng nhập
/// (`flutter_secure_storage`).
class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final mode = ref.watch(themeModeProvider).value ?? ThemeMode.system;
    final signedIn = ref.watch(sessionProvider).value ?? false;
    return Scaffold(
      appBar: AppBar(title: const Text('Cài đặt')),
      body: ListView(
        padding: const EdgeInsets.all(ToiNoiSpacing.md),
        children: [
          Text('Giao diện', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: ToiNoiSpacing.sm),
          SegmentedButton<ThemeMode>(
            segments: const [
              ButtonSegment(value: ThemeMode.system, label: Text('Hệ thống')),
              ButtonSegment(value: ThemeMode.light, label: Text('Sáng')),
              ButtonSegment(value: ThemeMode.dark, label: Text('Tối')),
            ],
            selected: {mode},
            onSelectionChanged: (selection) =>
                ref.read(themeModeProvider.notifier).select(selection.first),
          ),
          const SizedBox(height: ToiNoiSpacing.lg),
          Text(
            'Phiên đăng nhập',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: ToiNoiSpacing.sm),
          Text(signedIn ? 'Đã lưu token trên máy' : 'Chưa đăng nhập'),
          const SizedBox(height: ToiNoiSpacing.sm),
          if (signedIn)
            OutlinedButton(
              onPressed: () => ref.read(sessionProvider.notifier).signOut(),
              child: const Text('Đăng xuất'),
            )
          else
            FilledButton(
              onPressed: () => ref.read(sessionProvider.notifier).signInDemo(),
              child: const Text('Đăng nhập thử'),
            ),
        ],
      ),
    );
  }
}
