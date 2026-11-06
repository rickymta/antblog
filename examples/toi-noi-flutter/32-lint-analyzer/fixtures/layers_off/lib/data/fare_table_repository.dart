import '../domain/quote_repository.dart';

class FareTableRepository implements QuoteRepository {
  @override
  Future<int> quoteVnd(double distanceKm) async => 25500;
}
