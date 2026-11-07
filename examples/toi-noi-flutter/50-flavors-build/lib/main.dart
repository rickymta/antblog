import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show appFlavor;

import 'flavor_screen.dart';

void main() {
  // `String.fromEnvironment` chỉ đảm bảo hoạt động khi gọi bằng const.
  const config = String.fromEnvironment('FLAVOR');
  runApp(
    const MaterialApp(
      title: 'Tới Nơi',
      home: FlavorScreen(native: appFlavor, config: config),
    ),
  );
}
