import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';

void main() {
  // flutter_secure_storage và shared_preferences gọi plugin: cần binding.
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const ProviderScope(child: ToiNoiApp()));
}
