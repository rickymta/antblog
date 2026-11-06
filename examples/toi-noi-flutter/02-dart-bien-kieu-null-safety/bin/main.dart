import 'package:toi_noi_02/fare.dart';
import 'package:toi_noi_02/quote.dart';

void main() {
  const distanceKm = 5.0; // const: biết từ lúc biên dịch
  final accepted = Quote.fromDistance(distanceKm, driverName: 'Lê Minh Châu');
  print('Đã có tài xế : ${accepted.describe()}');

  var quote = Quote.fromDistance(2.3); // var: suy ra kiểu Quote
  print('Chưa có tài xế: ${quote.describe()}');

  // Gán lại biến sang bản sao mới; đối tượng cũ không đổi.
  quote = quote.withDriver('Trần Văn An');
  // quote = 5; // không biên dịch: var đã chốt kiểu Quote
  print('Sau khi nhận  : ${quote.describe()}');

  print('5 / 2 = ${5 / 2}, 5 ~/ 2 = ${5 ~/ 2}');
  print(formatVnd(calculateFareVnd(100)));
}
