import 'package:flutter_test/flutter_test.dart';

import 'support/files.dart';

/// Sáu ID theo curriculum mục 3.2, gõ lại ở đây để hợp đồng `flavors.json` không
/// tự xác nhận chính nó.
const section32 = {
  'customer': {
    'dev': 'vn.toinoi.customer.dev',
    'staging': 'vn.toinoi.customer.staging',
    'prod': 'vn.toinoi.customer',
  },
  'driver': {
    'dev': 'vn.toinoi.driver.dev',
    'staging': 'vn.toinoi.driver.staging',
    'prod': 'vn.toinoi.driver',
  },
};

void main() {
  final contract = readJson('flavors.json');
  final contractApps = contract['apps'] as Map<String, dynamic>;

  Map<String, dynamic> section(String app, String key) =>
      (contractApps[app] as Map<String, dynamic>)[key] as Map<String, dynamic>;

  test('có đúng ba flavor theo thứ tự dev, staging, prod', () {
    expect(contract['flavors'], flavors);
  });

  test('có đúng hai app: customer và driver', () {
    expect(contractApps.keys, unorderedEquals(apps));
  });

  test('ID khớp bảng mục 3.2 của curriculum', () {
    for (final app in apps) {
      expect(section(app, 'ids'), section32[app], reason: app);
    }
  });

  test('prod giữ ID gốc, dev và staging thêm đúng hậu tố tên flavor', () {
    for (final app in apps) {
      final ids = section(app, 'ids');
      expect(ids['dev'], '${ids['prod']}.dev');
      expect(ids['staging'], '${ids['prod']}.staging');
    }
  });

  test('sáu ID đều khác nhau, nên ba bản cài cạnh nhau được', () {
    final all = [
      for (final app in apps) ...section(app, 'ids').values.cast<String>(),
    ];
    expect(all.toSet().length, 6);
  });

  test('tên app khác nhau giữa ba flavor của mỗi app', () {
    for (final app in apps) {
      final names = section(app, 'displayNames').values.toSet();
      expect(names.length, 3, reason: app);
    }
  });
}
