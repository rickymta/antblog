// test_reflective_loader chỉ nhận phương thức tên `test_...`, nên tên đó
// không thể là lowerCamelCase.
// ignore_for_file: non_constant_identifier_names

import 'package:analyzer_testing/analysis_rule/analysis_rule.dart';
import 'package:test_reflective_loader/test_reflective_loader.dart';
import 'package:toi_noi_lints/src/no_data_import_in_presentation.dart';

void main() {
  defineReflectiveSuite(() {
    defineReflectiveTests(NoDataImportInPresentationTest);
  });
}

/// Mỗi file mẫu dùng thứ nó import, để analyzer không thêm cảnh báo
/// "unused import" làm nhiễu kết quả.
@reflectiveTest
class NoDataImportInPresentationTest extends AnalysisRuleTest {
  String get _lib => testPackageLibPath;

  @override
  void setUp() {
    rule = NoDataImportInPresentation();
    // newPackage phải gọi trước super.setUp.
    newPackage('toi_noi_other').addFile('lib/data/x.dart', 'class X {}');
    super.setUp();
    newFile('$_lib/data/trip_repository_impl.dart', 'class Impl {}');
    newFile('$_lib/domain/trip_repository.dart', 'class Port {}');
  }

  /// Offset của literal [uri] (tính cả dấu nháy mở) trong [source].
  int offsetOf(String source, String uri) => source.indexOf("'$uri'");

  /// Ghi [source] thành một file presentation và kiểm các lint kỳ vọng.
  Future<void> expectInPresentation(
    String source,
    List<String> flaggedUris,
  ) async {
    final path = '$_lib/presentation/screen.dart';
    newFile(path, source);
    await assertDiagnosticsInFile(path, [
      for (final uri in flaggedUris)
        lint(offsetOf(source, uri), uri.length + 2),
    ]);
  }

  void test_absolute_package_import_of_data() => expectInPresentation(
    "import 'package:test/data/trip_repository_impl.dart';\nImpl? a;\n",
    ['package:test/data/trip_repository_impl.dart'],
  );

  void test_relative_import_of_data() => expectInPresentation(
    "import '../data/trip_repository_impl.dart';\nImpl? a;\n",
    ['../data/trip_repository_impl.dart'],
  );

  void test_import_with_prefix() => expectInPresentation(
    "import '../data/trip_repository_impl.dart' as impl;\nimpl.Impl? a;\n",
    ['../data/trip_repository_impl.dart'],
  );

  void test_export_of_data() => expectInPresentation(
    "export '../data/trip_repository_impl.dart';\n",
    ['../data/trip_repository_impl.dart'],
  );

  void test_conditional_import_configuration() => expectInPresentation(
    "import '../domain/trip_repository.dart'\n"
    "    if (dart.library.io) '../data/trip_repository_impl.dart';\n"
    'Port? a;\n',
    ['../data/trip_repository_impl.dart'],
  );

  void test_two_violations_are_both_reported() => expectInPresentation(
    "import '../data/trip_repository_impl.dart';\n"
    "export 'package:test/data/trip_repository_impl.dart';\n"
    'Impl? a;\n',
    [
      '../data/trip_repository_impl.dart',
      'package:test/data/trip_repository_impl.dart',
    ],
  );

  void test_domain_and_dart_imports_are_fine() => expectInPresentation(
    "import 'dart:async';\n"
    "import '../domain/trip_repository.dart';\n"
    'Port? a;\n'
    'Future<void>? b;\n',
    [],
  );

  void test_data_may_import_data_and_domain() async {
    final path = '$_lib/data/other.dart';
    newFile(
      path,
      "import 'trip_repository_impl.dart';\n"
      "import '../domain/trip_repository.dart';\n"
      'Impl? a;\n'
      'Port? b;\n',
    );
    await assertNoDiagnosticsInFile(path);
  }

  void test_composition_root_may_import_data() async {
    final path = '$_lib/composition.dart';
    newFile(path, "import 'data/trip_repository_impl.dart';\nImpl? a;\n");
    await assertNoDiagnosticsInFile(path);
  }

  void test_other_package_with_a_data_folder_is_fine() => expectInPresentation(
    "import 'package:toi_noi_other/data/x.dart';\nX? a;\n",
    [],
  );

  void test_presentation_subfolder_counts_as_presentation() async {
    const source = "import '../../data/trip_repository_impl.dart';\nImpl? a;\n";
    final path = '$_lib/presentation/widgets/card.dart';
    newFile(path, source);
    await assertDiagnosticsInFile(path, [
      lint(
        offsetOf(source, '../../data/trip_repository_impl.dart'),
        '../../data/trip_repository_impl.dart'.length + 2,
      ),
    ]);
  }
}
