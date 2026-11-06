import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'composition.dart';
import 'presentation/app.dart';

void main() => runApp(
  ProviderScope(
    retry: noProviderRetry,
    overrides: [productionTripRepository()],
    child: const ToiNoiApp(),
  ),
);
