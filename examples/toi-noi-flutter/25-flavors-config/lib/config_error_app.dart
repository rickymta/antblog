import 'package:flutter/material.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

/// Hiện khi cấu hình sai. Ném lỗi trước `runApp` chỉ cho màn hình trắng trên
/// máy thật; màn hình này nói rõ khoá nào sai và cách chạy lại.
class ConfigErrorApp extends StatelessWidget {
  const ConfigErrorApp({required this.problems, super.key});

  final List<String> problems;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Tới Nơi: cấu hình lỗi',
      theme: ToiNoiTheme.light(),
      home: Scaffold(
        appBar: AppBar(title: const Text('Cấu hình không hợp lệ')),
        body: SafeArea(
          child: ListView(
            padding: const EdgeInsets.all(ToiNoiSpacing.md),
            children: [
              const Text(
                'Chạy lại với --dart-define-from-file=config/<dev|staging|prod>.json',
              ),
              const SizedBox(height: ToiNoiSpacing.md),
              for (final problem in problems)
                ListTile(
                  dense: true,
                  contentPadding: EdgeInsets.zero,
                  leading: const Icon(Icons.error_outline),
                  title: Text(problem),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
