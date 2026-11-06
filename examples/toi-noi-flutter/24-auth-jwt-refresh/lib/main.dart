import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';

/// Chạy với server giả đã bật xác thực:
/// `dart run bin/server.dart --with-auth` trong `tools/fake_api`.
void main() => runApp(const ProviderScope(child: ToiNoiApp()));
