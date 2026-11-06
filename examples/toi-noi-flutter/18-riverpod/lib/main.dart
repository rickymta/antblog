import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';

void main() {
  // ProviderScope giữ state của mọi provider; phải nằm trên MaterialApp.
  runApp(const ProviderScope(child: ToiNoiApp()));
}
