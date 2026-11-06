import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Một lớp và những gì nó được phép import.
class Layer {
  const Layer(
    this.name, {
    required this.packages,
    required this.layers,
    this.dartLibraries,
  });

  /// Thư mục dưới `lib/`.
  final String name;

  /// Package ngoài được phép (không tính chính package của lab).
  final Set<String> packages;

  /// Lớp trong lab được phép (luôn gồm chính nó).
  final Set<String> layers;

  /// Thư viện `dart:` được phép; `null` là không kiểm.
  final Set<String>? dartLibraries;
}

const ownPackage = 'toi_noi_21';

const layers = [
  Layer(
    'domain',
    packages: {'toi_noi_core'},
    layers: {'domain'},
    dartLibraries: {'async', 'collection', 'core', 'math'},
  ),
  Layer(
    'data',
    packages: {'toi_noi_core', 'toi_noi_api'},
    layers: {'domain', 'data'},
  ),
  Layer(
    'presentation',
    packages: {'flutter', 'flutter_riverpod', 'toi_noi_core', 'toi_noi_ui'},
    layers: {'domain', 'presentation'},
  ),
];

final _directive = RegExp(
  r'''^\s*(?:import|export)\s+['"]([^'"]+)['"]''',
  multiLine: true,
);

/// Các URI mà `import` và `export` trong [source] trỏ tới.
List<String> importsOf(String source) => [
  for (final match in _directive.allMatches(source)) match.group(1)!,
];

/// Lớp trong lab mà [uri] (viết trong [file], tính từ `lib/`) trỏ tới;
/// `'(gốc lib)'` nếu là file ở gốc `lib/` (composition root, main); `null`
/// nếu không phải import nội bộ.
String? ownLayerOf(String file, String uri) {
  final String path;
  if (uri.startsWith('package:$ownPackage/')) {
    path = uri.substring('package:$ownPackage/'.length);
  } else if (!uri.contains(':')) {
    path = Uri.parse('lib/$file').resolve(uri).path.replaceFirst('lib/', '');
  } else {
    return null;
  }
  final segments = path.split('/');
  return segments.length > 1 ? segments.first : '(gốc lib)';
}

/// Những import trong [source] (của `lib/[file]`) vi phạm quy tắc của [layer].
List<String> violationsIn(Layer layer, String file, String source) {
  final problems = <String>[];
  for (final uri in importsOf(source)) {
    if (uri.startsWith('dart:')) {
      final lib = uri.substring('dart:'.length);
      final allowed = layer.dartLibraries;
      if (allowed != null && !allowed.contains(lib)) {
        problems.add('$file: $uri (thư viện dart: không được phép)');
      }
    } else if (uri.startsWith('package:') &&
        !uri.startsWith('package:$ownPackage/')) {
      final name = uri.substring('package:'.length).split('/').first;
      if (!layer.packages.contains(name)) {
        problems.add('$file: $uri (package $name không được phép)');
      }
    } else {
      final target = ownLayerOf(file, uri);
      if (target == null || !layer.layers.contains(target)) {
        problems.add('$file: $uri (lớp $target không được phép)');
      }
    }
  }
  return problems;
}

Map<String, String> sourcesOf(Layer layer) => {
  for (final entity in Directory('lib/${layer.name}').listSync(recursive: true))
    if (entity is File && entity.path.endsWith('.dart'))
      entity.path.replaceFirst('lib/', ''): entity.readAsStringSync(),
};

void main() {
  group('domain không biết framework và tầng mạng', () {
    final sources = sourcesOf(layers.first);

    test('có file để kiểm (test không đậu vì thư mục rỗng)', () {
      expect(sources.length, greaterThanOrEqualTo(5));
    });

    for (final banned in [
      'package:flutter',
      'package:dio',
      'package:toi_noi_api',
    ]) {
      test('không import $banned', () {
        final offenders = [
          for (final MapEntry(:key, :value) in sources.entries)
            if (importsOf(value).any((uri) => uri.startsWith(banned))) key,
        ];
        expect(offenders, isEmpty);
      });
    }
  });

  for (final layer in layers) {
    test('lớp ${layer.name} chỉ import những gì được phép', () {
      final sources = sourcesOf(layer);
      expect(sources, isNotEmpty);
      final problems = [
        for (final MapEntry(:key, :value) in sources.entries)
          ...violationsIn(layer, key, value),
      ];
      expect(problems, isEmpty);
    });
  }

  group('bộ kiểm tự nó bắt được vi phạm', () {
    final domain = layers.first;

    test('import package bị cấm', () {
      expect(
        violationsIn(domain, 'domain/x.dart', "import 'package:dio/dio.dart';"),
        hasLength(1),
      );
    });

    test('export cũng được kiểm', () {
      expect(
        violationsIn(
          domain,
          'domain/x.dart',
          "export 'package:toi_noi_api/toi_noi_api.dart';",
        ),
        hasLength(1),
      );
    });

    test('import tương đối trèo sang lớp khác', () {
      expect(
        violationsIn(
          domain,
          'domain/x.dart',
          "import '../data/trip_repository_impl.dart';",
        ),
        hasLength(1),
      );
    });

    test('import bằng đường dẫn package của chính lab sang lớp khác', () {
      expect(
        violationsIn(
          domain,
          'domain/x.dart',
          "import 'package:$ownPackage/presentation/providers.dart';",
        ),
        hasLength(1),
      );
    });

    test('dart:io trong domain', () {
      expect(
        violationsIn(domain, 'domain/x.dart', "import 'dart:io';"),
        hasLength(1),
      );
    });

    test('dòng chú thích không tính', () {
      expect(
        violationsIn(
          domain,
          'domain/x.dart',
          "// import 'package:dio/dio.dart';",
        ),
        isEmpty,
      );
    });

    test('import hợp lệ không báo', () {
      expect(
        violationsIn(
          domain,
          'domain/x.dart',
          "import 'package:toi_noi_core/toi_noi_core.dart';\n"
              "import 'quote_request.dart';\n"
              "import 'dart:async';",
        ),
        isEmpty,
      );
    });

    test('presentation không được import data', () {
      expect(
        violationsIn(
          layers.last,
          'presentation/x.dart',
          "import 'package:$ownPackage/data/mappers.dart';",
        ),
        hasLength(1),
      );
    });
  });
}
