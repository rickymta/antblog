import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'router.dart';

void main() {
  runApp(ProviderScope(child: ToiNoiApp(router: createRouter())));
}
