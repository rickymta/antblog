import 'package:go_router/go_router.dart';

import 'quote_screen.dart';
import 'trip_screen.dart';

/// Hai route của bài: báo giá ở `/`, chuyến ở `/trip/:id`.
GoRouter createRouter({String initialLocation = '/'}) => GoRouter(
  initialLocation: initialLocation,
  routes: [
    GoRoute(path: '/', builder: (context, state) => const QuoteScreen()),
    GoRoute(
      path: '/trip/:id',
      builder: (context, state) => TripScreen(id: state.pathParameters['id']!),
    ),
  ],
);
