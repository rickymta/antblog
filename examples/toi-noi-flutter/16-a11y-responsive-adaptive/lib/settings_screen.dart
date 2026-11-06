import 'package:flutter/material.dart';

/// Màn hình cài đặt tối thiểu, đủ để có một công tắc.
///
/// `SwitchListTile.adaptive` đọc `ThemeData.platform` để vẽ công tắc theo kiểu
/// iOS/macOS hoặc Material; bên dưới vẫn là `Switch` của Material (không có
/// `CupertinoSwitch`). Công tắc chưa nối với chức năng nào.
class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  bool _saveData = false;

  @override
  Widget build(BuildContext context) {
    return ListView(
      children: [
        SwitchListTile.adaptive(
          title: const Text('Tiết kiệm dữ liệu'),
          subtitle: const Text(
            'Giảm ảnh và cập nhật nền khi dùng mạng di động',
          ),
          value: _saveData,
          onChanged: (value) => setState(() => _saveData = value),
        ),
      ],
    );
  }
}
