import 'package:analysis_server_plugin/plugin.dart';
import 'package:analysis_server_plugin/registry.dart';

import 'src/no_data_import_in_presentation.dart';

/// Điểm vào của plugin: analysis server tìm biến `plugin` ở `lib/main.dart`.
final plugin = ToiNoiLintsPlugin();

class ToiNoiLintsPlugin extends Plugin {
  @override
  String get name => 'toi_noi_lints';

  @override
  void register(PluginRegistry registry) {
    // Lint rule mặc định tắt: dự án phải bật trong analysis_options.yaml.
    registry.registerLintRule(NoDataImportInPresentation());
  }
}
