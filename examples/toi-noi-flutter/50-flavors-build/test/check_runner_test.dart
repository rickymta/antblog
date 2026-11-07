import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'support/files.dart';

final nodeAvailable = () {
  try {
    return Process.runSync('node', ['--version']).exitCode == 0;
  } on ProcessException {
    return false;
  }
}();
final skip = nodeAvailable ? null : 'cần node trên PATH';

/// Một khối XCBuildConfiguration đúng dạng Xcode ghi trong project.pbxproj
/// (theo khối của target Runner trong template Flutter 3.47.4).
String block(
  String name, {
  String? base,
  Map<String, String> settings = const {},
  bool runnerTarget = true,
}) {
  final lines = [
    '\t\tABCDEF${name.hashCode.abs().toRadixString(16).padLeft(18, '0')} /* $name */ = {',
    '\t\t\tisa = XCBuildConfiguration;',
    if (base != null)
      '\t\t\tbaseConfigurationReference = 0123456789ABCDEF01234567 /* $base */;',
    '\t\t\tbuildSettings = {',
    if (runnerTarget) '\t\t\t\tINFOPLIST_FILE = Runner/Info.plist;',
    for (final e in settings.entries) '\t\t\t\t${e.key} = ${e.value};',
    '\t\t\t\tPRODUCT_NAME = "\$(TARGET_NAME)";',
    '\t\t\t};',
    '\t\t\tname = "$name";',
    '\t\t};',
  ];
  return '${lines.join('\n')}\n';
}

const modes = ['Debug', 'Profile', 'Release'];

/// Chín configuration; [override] cho tên config nào target còn đặt giá trị cứng.
String project({
  Set<String> missing = const {},
  Map<String, String> wrongBase = const {},
  Map<String, Map<String, String>> override = const {},
  bool baseOnProjectOnly = false,
}) {
  final out = StringBuffer(
    '/* Begin XCBuildConfiguration section */\n${block('Debug')}',
  );
  for (final flavor in flavors) {
    for (final mode in modes) {
      final name = '$mode-$flavor';
      if (missing.contains(name)) continue;
      if (baseOnProjectOnly) {
        // Xcode: xcconfig gắn ở dòng PROJECT, còn target Runner không gắn gì.
        out.write(block(name, base: '$name.xcconfig', runnerTarget: false));
        out.write(block(name, settings: override[name] ?? const {}));
        continue;
      }
      out.write(
        block(
          name,
          base: wrongBase[name] ?? '$name.xcconfig',
          settings: override[name] ?? const {},
        ),
      );
    }
  }
  out.write('/* End XCBuildConfiguration section */\n');
  return out.toString();
}

ProcessResult check(String pbxproj) {
  final dir = Directory.systemTemp.createTempSync('tn50_pbx_');
  addTearDown(() => dir.deleteSync(recursive: true));
  final file = File('${dir.path}/project.pbxproj')..writeAsStringSync(pbxproj);
  return Process.runSync('node', [path('tool/check-runner.mjs'), file.path]);
}

void main() {
  test('chín configuration đủ, base đúng, không ghi đè: sạch', () {
    final result = check(project());
    expect(result.exitCode, 0, reason: '${result.stderr}');
  }, skip: skip);

  test('template chưa sửa (chỉ Debug) thiếu cả chín configuration', () {
    final result = check(
      '/* Begin XCBuildConfiguration section */\n${block('Debug', base: 'Debug.xcconfig')}',
    );
    expect(result.exitCode, 1);
    expect(
      '${result.stderr}'
          .split('\n')
          .where((l) => l.contains('target Runner không có configuration')),
      hasLength(9),
    );
  }, skip: skip);

  test('thiếu một configuration: nêu đúng tên', () {
    final result = check(project(missing: {'Profile-staging'}));
    expect(result.exitCode, 1);
    expect(
      result.stderr,
      contains('target Runner không có configuration Profile-staging'),
    );
  }, skip: skip);

  test('configuration gắn nhầm xcconfig', () {
    final result = check(
      project(wrongBase: {'Release-dev': 'Release.xcconfig'}),
    );
    expect(result.exitCode, 1);
    expect(
      result.stderr,
      contains(
        'Release-dev: target Runner có base Release.xcconfig, cần Release-dev.xcconfig',
      ),
    );
  }, skip: skip);

  test('target còn PRODUCT_BUNDLE_IDENTIFIER cứng: ghi đè xcconfig', () {
    final result = check(
      project(
        override: {
          'Debug-dev': {'PRODUCT_BUNDLE_IDENTIFIER': 'vn.toinoi.customer'},
        },
      ),
    );
    expect(result.exitCode, 1);
    expect(
      result.stderr,
      contains(
        'Debug-dev: target đặt PRODUCT_BUNDLE_IDENTIFIER = vn.toinoi.customer',
      ),
    );
  }, skip: skip);

  test('target còn ASSETCATALOG_COMPILER_APPICON_NAME cứng cũng bị bắt', () {
    final result = check(
      project(
        override: {
          'Release-prod': {'ASSETCATALOG_COMPILER_APPICON_NAME': 'AppIcon'},
        },
      ),
    );
    expect(result.exitCode, 1);
    expect(
      result.stderr,
      contains('Release-prod: target đặt ASSETCATALOG_COMPILER_APPICON_NAME'),
    );
  }, skip: skip);

  test(
    'xcconfig chỉ gắn ở project, target Runner còn ghi đè: không báo OK',
    () {
      final result = check(
        project(
          baseOnProjectOnly: true,
          override: {
            'Debug-dev': {'PRODUCT_BUNDLE_IDENTIFIER': 'vn.toinoi.customer'},
          },
        ),
      );
      expect(result.exitCode, 1);
      final err = '${result.stderr}';
      expect(err, contains('Debug-dev: target Runner có base (không có)'));
      expect(err, contains('Debug-dev: target đặt PRODUCT_BUNDLE_IDENTIFIER'));
    },
    skip: skip,
  );

  test('khối cùng tên ở project không thay được khối của target Runner', () {
    final result = check(project(baseOnProjectOnly: true));
    expect(result.exitCode, 1);
    expect(
      '${result.stderr}'
          .split('\n')
          .where((l) => l.contains('target Runner có base (không có)')),
      hasLength(9),
    );
  }, skip: skip);

  test(r'$(inherited) không phải ghi đè', () {
    final result = check(
      project(
        override: {
          'Debug-dev': {'PRODUCT_BUNDLE_IDENTIFIER': r'"$(inherited)"'},
        },
      ),
    );
    expect(result.exitCode, 0, reason: '${result.stderr}');
  }, skip: skip);

  test('thiếu đường dẫn hoặc file: thoát 2', () {
    expect(
      Process.runSync('node', [path('tool/check-runner.mjs')]).exitCode,
      2,
    );
    expect(
      Process.runSync('node', [
        path('tool/check-runner.mjs'),
        '/khong/co/file',
      ]).exitCode,
      2,
    );
  }, skip: skip);
}
