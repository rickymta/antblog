import 'package:flutter_riverpod/misc.dart';
import 'package:toi_noi_32/data/fare_table_repository.dart';
import 'package:toi_noi_32/presentation/quote_providers.dart';

/// Composition root: nơi duy nhất nối presentation với thực thi của data.
final List<Override> productionOverrides = [
  quoteRepositoryProvider.overrideWithValue(const FareTableRepository()),
];
