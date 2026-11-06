import 'package:flutter_test/flutter_test.dart';
import 'package:intl/intl.dart';

// File riêng để không có test nào khác nạp dữ liệu ngày tháng trước: mỗi file
// test chạy trong một isolate của nó.
void main() {
  test('DateFormat với locale chưa nạp dữ liệu ném LocaleDataException', () {
    // LocaleDataException nằm trong src của intl, không export, nên kiểm
    // bằng tên kiểu và nội dung thông báo.
    expect(
      () => DateFormat.yMd('vi').format(DateTime.utc(2026, 11, 6)),
      throwsA(
        predicate<Object>(
          (e) =>
              e.runtimeType.toString() == 'LocaleDataException' &&
              e.toString().contains('initializeDateFormatting'),
        ),
      ),
    );
  });
}
