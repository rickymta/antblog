import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'providers.dart';

/// `noProviderRetry` tắt cơ chế thử lại mặc định của Riverpod cho mọi
/// provider. Việc thử lại đã có ở `RetryInterceptor`, nơi biết request nào lặp
/// lại được an toàn; để hai tầng cùng thử thì số request nhân với nhau.
void main() =>
    runApp(ProviderScope(retry: noProviderRetry, child: const ToiNoiApp()));
