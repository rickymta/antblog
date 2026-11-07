import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';
import 'package:native_toolchain_c/native_toolchain_c.dart';

// Hook chạy tự động khi `dart test`, `dart run`, `dart build` (theo dart.dev) và
// `flutter test` (đã thử trên macOS). Nó biên dịch file C bằng compiler của máy
// và khai báo kết quả là một code asset tên
// package:toi_noi_round_fare/toi_noi_round_fare.dart.
void main(List<String> args) async {
  await build(args, (input, output) async {
    if (!input.config.buildCodeAssets) return;
    await CBuilder.library(
      name: 'toi_noi_round_fare',
      assetName: 'toi_noi_round_fare.dart',
      sources: ['src/toi_noi_round_fare.c'],
    ).run(input: input, output: output);
  });
}
