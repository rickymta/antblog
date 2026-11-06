---
slug: flutter-lint-analyzer-format-analyzer-plugin-chat-luong-code
title: "Lint, analyzer, dart format và analyzer plugin: chất lượng code có kiểm chứng"
summary: "Bài 32: dựng cổng chất lượng cho Tới Nơi: analysis_options với flutter_lints và ba cờ strict, riverpod_lint qua analyzer plugin chính thức (custom_lint không còn là đường khuyên dùng), một lint tự viết cấm presentation import data và test bằng analyzer_testing, dart format, dart fix, pre-commit."
category: programming
level: intermediate
tags: [flutter, dart, mobile, toi-noi, lint, analyzer, riverpod, testing]
language: vi
---

Lần review thứ ba trong tháng, Lan lại để cùng một bình luận: "presentation import thẳng data rồi". Tuần trước có thêm một PR trộn logic với hai trăm dòng chỉ khác cách xuống dòng, và một hàm đọc `fareVnd` từ JSON nhận `dynamic` mà không ai kiểm. Cả ba việc máy làm đều đặn hơn người.

Bài này dựng cổng chất lượng cho Tới Nơi: `analysis_options.yaml` chặt, một lint tự viết cấm presentation import data, `dart format`, `dart fix`, `// ignore` có lý do, pre-commit và ngân sách cảnh báo bằng 0. Đây là bài 32 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 31 về integration test và patrol](/articles/flutter-integration-test-patrol-luong-dat-chuyen-tren-thiet-bi) trước, và [bài 21 về Clean Architecture](/articles/flutter-clean-architecture-domain-data-presentation-mobile), vì lint mới thay giới hạn mà test regex ở bài đó đã ghi lại. Phần lint đầu tiên nằm ở [bài 06](/articles/dart-package-pubspec-lint-test-toi-noi-core).

Một điểm lệch so với khung loạt bài: khung ghi `custom_lint`, nhưng cuối năm 2026 đó không còn là đường khuyên dùng. Bài đi theo analyzer plugin chính thức của Dart; bằng chứng nằm ở phần "Plugin chính thức".

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Dự án bật <code>plugins: riverpod_lint</code>. Một file có <code>Notifier</code> với field công khai (lint của riverpod_lint, mức info), một file khác import lớp data từ presentation (lint ta tự viết, mức warning). Ta chạy <code>flutter analyze</code>. Nó báo gì?</p>
<details><summary>Xem đáp án</summary><p>Không báo gì: với Flutter 3.47.4, <code>flutter analyze</code> in <code>No issues found!</code> dù hai vi phạm có thật. <code>dart analyze</code> trong cùng thư mục báo cả hai, nên cổng chặn của lab là <code>dart analyze --fatal-infos</code>.</p></details>
</div>

## Ba tầng kiểm tra và ai chạy chúng

Ba thứ hay bị gọi chung là "lint" nằm ở ba tầng:

- **Analyzer** là bộ phân tích tĩnh của Dart: lỗi biên dịch, cảnh báo, và ba cờ `strict-*` ở phần kế tiếp.
- **Linter** là tập rule đi kèm SDK, bật bằng `include` hoặc `linter: rules:`. Phần lớn ở mức info.
- **Plugin** là code ngoài SDK, do analysis server khởi chạy riêng và nói chuyện qua giao thức plugin, khai báo ở khối `plugins:`. `riverpod_lint` và lint tự viết thuộc tầng này.

Cả ba cho cùng dạng kết quả: chẩn đoán có mức và mã rule. Mã thoát của `dart analyze` phản ánh mức đó: 0 khi sạch, 1 khi chỉ còn info và ta thêm `--fatal-infos`, 2 khi có warning, 3 khi có error, và 4 khi không dựng được plugin (ví dụ khi không với tới pub.dev). Lab đo cả năm giá trị bằng test.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 430" role="img" aria-label="Hình 1. Ba tầng kiểm tra. Mã nguồn và analysis_options.yaml ở bên trái đi vào ba tầng xếp dọc: tầng 1 analyzer với ba cờ strict, tầng 2 linter với flutter_lints và rule thêm, tầng 3 plugin gồm riverpod_lint và toi_noi_lints. Cả ba đều đổ về hộp chẩn đoán bên phải gồm error, warning và info. Ghi chú: flutter analyze trên Flutter 3.47.4 báo tầng 1 và 2, không báo tầng 3; dùng dart analyze.">
<defs><marker id="fl32-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="150" width="210" height="110" rx="10" class="f-surface s-muted"/><text x="34" y="180" class="t-b">Mã nguồn</text><text x="34" y="206" class="t-small">lib/, test/ (.dart)</text><text x="34" y="226" class="t-small">analysis_options.yaml</text><text x="34" y="246" class="t-small">pubspec.yaml</text>
<rect x="300" y="20" width="330" height="100" rx="10" class="f-phys-soft s-phys"/><text x="316" y="48" class="t-b">1. Analyzer (ngôn ngữ)</text><text x="316" y="74" class="t-small">lỗi biên dịch, strict-casts,</text><text x="316" y="94" class="t-small">strict-inference, strict-raw-types</text>
<rect x="300" y="150" width="330" height="100" rx="10" class="f-pk-soft s-pk"/><text x="316" y="178" class="t-b">2. Linter (rule trong SDK)</text><text x="316" y="204" class="t-small">flutter_lints + 9 rule thêm</text><text x="316" y="224" class="t-small">mặc định ở mức info</text>
<rect x="300" y="280" width="330" height="100" rx="10" class="f-warn-soft s-warn"/><text x="316" y="308" class="t-b">3. Plugin (khối plugins:)</text><text x="316" y="334" class="t-small">riverpod_lint, toi_noi_lints</text><text x="316" y="354" class="t-small">server khởi chạy riêng</text>
<rect x="690" y="150" width="170" height="110" rx="10" class="f-surface s-muted"/><text x="704" y="180" class="t-b">Chẩn đoán</text><text x="704" y="206" class="t-small">error, warning,</text><text x="704" y="226" class="t-small">info + mã rule</text>
<path d="M232 190 C268 190 268 70 298 70" class="f-none s-muted" stroke-width="2" marker-end="url(#fl32-1-a)"/>
<path d="M232 205 L298 200" class="f-none s-muted" stroke-width="2" marker-end="url(#fl32-1-a)"/>
<path d="M232 220 C268 220 268 330 298 330" class="f-none s-muted" stroke-width="2" marker-end="url(#fl32-1-a)"/>
<path d="M632 70 C660 70 660 180 688 180" class="f-none s-muted" stroke-width="2" marker-end="url(#fl32-1-a)"/>
<path d="M632 200 L688 205" class="f-none s-muted" stroke-width="2" marker-end="url(#fl32-1-a)"/>
<path d="M632 330 C660 330 660 235 688 235" class="f-none s-muted" stroke-width="2" marker-end="url(#fl32-1-a)"/>
<text x="20" y="412" class="t-small t-warn">flutter analyze (3.47.4) báo tầng 1 và 2 nhưng không báo tầng 3. Dùng dart analyze.</text>
</svg>
</div>
<figcaption>Hình 1. Một nguồn, ba tầng, một dạng kết quả. Tầng plugin là tầng duy nhất flutter analyze bỏ qua trong lần thử của ta.</figcaption>
</figure>

Một file nối cả ba tầng. Đây là `analysis_options.yaml` của lab 32:

### analysis_options.yaml · 32-lint-analyzer/analysis_options.yaml

```yaml
include: package:flutter_lints/flutter.yaml

analyzer:
  exclude:
    - build/**
    - android/**
    - ios/**
    # Mẫu cố ý có lỗi để test kiểm; có analysis_options riêng, không thuộc app.
    - fixtures/**
    # Package plugin có pubspec và test riêng (dart analyze, dart test trong đó).
    - lint_plugin/**
  language:
    strict-casts: true
    strict-inference: true
    strict-raw-types: true

linter:
  rules:
    - avoid_dynamic_calls
    - directives_ordering
    - document_ignores
    - prefer_const_constructors
    - prefer_const_declarations
    - prefer_final_locals
    - prefer_single_quotes
    - unawaited_futures
    - unnecessary_ignore

# Analyzer plugin chính thức (Dart 3.10+). Rule của plugin tắt mặc định,
# nên toi_noi_lints phải bật từng rule dưới `diagnostics`.
# riverpod_lint ghim đúng một bản: plugin không có lockfile, bản mới có thể
# thêm lint và làm đỏ ngân sách bằng 0 mà không ai đổi code.
plugins:
  riverpod_lint: 3.2.0
  toi_noi_lints:
    path: lint_plugin
    diagnostics:
      no_data_import_in_presentation: true
```

`flutter_lints` 6.0.0 là `lints/recommended` cộng 10 rule riêng cho Flutter và không có biến thể "strict". "Mức strict" của lab là ba cờ cộng 9 rule ta chọn.

## Ba cờ strict: dynamic không lọt qua âm thầm

Ba cờ nằm ở `analyzer.language`, mỗi cờ chặn một kẽ hở để `dynamic` chui vào mà không ai thấy. Việc quen thuộc của Tới Nơi, đọc cước từ body JSON, viết hai lần. Bản "trước":

### before.dart · 32-lint-analyzer/fixtures/strict_before/lib/before.dart

```dart
import 'dart:async';
import 'dart:convert';

/// Đọc cước (đồng) từ body JSON của trip-service.
int fareOf(String body) {
  final Map<String, dynamic> json = jsonDecode(body); // strict-casts
  final int fare = json['fareVnd']; // strict-casts
  return fare;
}

/// Số cước đang chờ.
int pendingCount(List fares) => fares.length; // strict-raw-types

void remember(String body) {
  final fares = []; // strict-inference
  final done = Completer(); // strict-inference
  done.complete(fareOf(body));
  fares.add(done.future);
}
```

Dưới ba cờ, `dart analyze` báo đúng 5 chẩn đoán: hai `invalid_assignment` (error, `strict-casts`, vì `jsonDecode` và `json['fareVnd']` trả `dynamic`), một `strict_raw_type` (warning, `List` trơn), một `inference_failure_on_collection_literal` và một `inference_failure_on_instance_creation` (warning, `strict-inference`). Cùng file đó dưới `analysis_options.yaml` rỗng cho 0 chẩn đoán, và bật từng cờ riêng thì mỗi cờ bắt đúng nhóm của nó. Bản "sau":

### after.dart · 32-lint-analyzer/fixtures/strict_after/lib/after.dart

```dart
import 'dart:async';
import 'dart:convert';

/// Đọc cước (đồng) từ body JSON của trip-service.
int fareOf(String body) {
  final json = jsonDecode(body) as Map<String, dynamic>;
  final fare = json['fareVnd'];
  if (fare is! int) {
    throw FormatException('fareVnd phải là số nguyên', body);
  }
  return fare;
}

/// Số cước đang chờ.
int pendingCount(List<Future<int>> fares) => fares.length;

void remember(String body) {
  final fares = <Future<int>>[];
  final done = Completer<int>();
  done.complete(fareOf(body));
  fares.add(done.future);
}
```

Cast `as Map<String, dynamic>` vẫn có thể ném `TypeError` khi body không phải object; điều cờ làm là buộc chỗ có thể sai hiện ra trong code, nơi review thấy. Còn `fare is! int` ném `FormatException` mang theo body gốc, nên lỗi dữ liệu có tên rõ ràng.

## Plugin chính thức, và vì sao không dùng custom_lint

`custom_lint` từng là cách phổ biến nhất để viết lint theo gói. Cuối năm 2026, bằng chứng cho thấy đó không còn là đường khuyên dùng. Gói chưa bị đánh dấu discontinued và pub.dev không ghi deprecated; chỉ là:

- `custom_lint` 0.8.1 (09/2025) là bản mới nhất trên pub.dev và buộc `analyzer ^8.0.0`; bản analyzer hiện hành là 14.7.0.
- Trong một project trống, `dart pub add dev:custom_lint` rồi `dev:riverpod_generator` cho `riverpod_generator 4.0.0+1` (mới nhất 4.0.10) và `analyzer 8.4.0`: cả chuỗi generator bị kéo lùi.
- `riverpod_lint` 3.1.0 (12/2025) ghi trong changelog rằng nó chuyển từ `custom_lint` sang `analysis_server_plugin`, cơ chế Dart 3.10 đưa vào SDK qua khối `plugins:`.

Vì vậy lab dùng đường chính thức cho cả `riverpod_lint` (3.2.0, phát hành 06/11/2026, nên kiểm lại khi đọc) lẫn plugin tự viết. Ta ghim đúng `3.2.0` thay vì `^3.2.0`: plugin không có lockfile, nên bản mới thêm lint có thể làm đỏ ngân sách bằng 0 mà không ai đổi dòng code nào. Theo dart.dev, warning của plugin bật mặc định còn lint rule của plugin tắt mặc định, nên `toi_noi_lints` bật rule dưới `diagnostics:`. Sửa khối `plugins:` xong phải khởi động lại analysis server.

Trong `~/.dartServer/.plugin_manager/`, mỗi tổ hợp plugin được dựng thành một package `plugin_entrypoint` rồi biên dịch AOT, nên lần đầu hoặc lần đầu sau khi sửa plugin chậm hơn hẳn. Bước resolve phụ thuộc thì chạy lại ở mỗi lần phân tích: ta trỏ `PUB_HOSTED_URL` vào một cổng chết và `dart analyze` thoát 4 kèm stack trace dù cache đã ấm. Vậy pre-commit và CI có plugin đều cần với tới pub.dev. Với `riverpod_lint`, ta thử tay một `Notifier` có field công khai và `dart analyze` báo `avoid_public_notifier_properties` (info); lab không có test tự động cho nó.

Còn `flutter analyze`? Trên Flutter 3.47.4 nó báo lỗi của analyzer (ta thêm `int y = 'a';` và nó bắt) nhưng không in chẩn đoán của plugin, lệch với dart.dev vốn mô tả plugin chạy cùng analyzer. Giả thuyết của ta, từ log giao thức: `flutter analyze` dùng language server và kết thúc ngay sau khi plugin báo đang phân tích, trước khi chẩn đoán của plugin về tới. Đó mới là giả thuyết và có thể đổi ở bản Flutter khác; ta chọn `dart analyze` làm cổng, lệnh này chạy được trong project Flutter.

## Tự viết một lint: presentation không được import data

Test của bài 21 đọc dòng `import` bằng regex và tự nhận giới hạn: URI viết ở dòng khác với từ khoá, hoặc `if (dart.library.io) '...'`, sẽ lọt. Rule mới đọc đích mà analyzer đã phân giải, không đoán từ chữ. Plugin là một package riêng:

### pubspec.yaml · 32-lint-analyzer/lint_plugin/pubspec.yaml (trích)

```yaml
dependencies:
  analysis_server_plugin: ^0.3.26
  analyzer: ^14.7.0
  path: ^1.9.0

dev_dependencies:
  analyzer_testing: ^0.4.5
  lints: ^6.1.0
  test: ^1.25.0
  test_reflective_loader: ^0.2.3
```

Analysis server tìm biến `plugin` ở `lib/main.dart`:

### main.dart · 32-lint-analyzer/lint_plugin/lib/main.dart

```dart
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
```

`registerLintRule` đăng ký rule opt-in (`registerWarningRule` sẽ bật sẵn), để mỗi dự án tự quyết. Rule là một lớp `AnalysisRule` cùng một visitor:

### no_data_import_in_presentation.dart · 32-lint-analyzer/lint_plugin/lib/src/no_data_import_in_presentation.dart (trích)

```dart
class NoDataImportInPresentation extends AnalysisRule {
  static const LintCode code = LintCode(
    'no_data_import_in_presentation',
    "Lớp presentation không được import lớp data: '{0}'.",
    correctionMessage:
        'Phụ thuộc interface của domain và để composition root cắm thực thi.',
    severity: DiagnosticSeverity.WARNING,
  );

  NoDataImportInPresentation()
    : super(
        name: 'no_data_import_in_presentation',
        description: 'Presentation chỉ phụ thuộc domain, không phụ thuộc data.',
      );

  @override
  LintCode get diagnosticCode => code;

  @override
  void registerNodeProcessors(
    RuleVisitorRegistry registry,
    RuleContext context,
  ) {
    final visitor = _Visitor(this, context);
    registry.addImportDirective(this, visitor);
    registry.addExportDirective(this, visitor);
  }
}
```

`LintCode` phải là `static const`: tài liệu của `analysis_server_plugin` nói nhiều instance cùng mã sẽ làm `// ignore` không chặn được. `registerNodeProcessors` chỉ xin hai loại node, `import` và `export`. Phần việc nằm ở visitor:

### no_data_import_in_presentation.dart · 32-lint-analyzer/lint_plugin/lib/src/no_data_import_in_presentation.dart (trích)

```dart
  void _check(NamespaceDirective node, DirectiveUri? uri) {
    final package = context.package;
    if (package == null) {
      return;
    }
    final pathContext = package.root.provider.pathContext;
    final from = layerOf(
      pathContext,
      package.root.path,
      context.definingUnit.file.path,
    );
    if (from != 'presentation') {
      return;
    }
    void check(StringLiteral literal, DirectiveUri? target) {
      if (target is! DirectiveUriWithSource) {
        return;
      }
      final to = layerOf(
        pathContext,
        package.root.path,
        target.source.fullName,
      );
      if (to == 'data') {
        rule.reportAtNode(literal, arguments: [target.relativeUriString]);
      }
    }

    check(node.uri, uri);
    for (final configuration in node.configurations) {
      check(configuration.uri, configuration.resolvedUri);
    }
  }
```

Rule không đụng tới chuỗi URI. `DirectiveUriWithSource` đã mang đường dẫn file đích, nên import tương đối, `package:` về chính package, `export` và URI trong `if (...)` đều quy về một phép so: lớp của file đang phân tích (thư mục con đầu tiên dưới `lib/`) có là `presentation` và lớp của đích có là `data` không.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Hình 2. Quyết định của rule. Bốn hộp nối tiếp từ trái sang phải: một directive import hoặc export, kể cả URI trong if; câu hỏi file đang phân tích có nằm trong lib/presentation không; câu hỏi đích đã phân giải có nằm trong lib/data không; cuối cùng báo lint tại chuỗi URI. Từ hai câu hỏi, đáp án không đi xuống hộp bỏ qua. Ghi chú: bài 21 đọc chữ trên dòng bằng regex, rule này đọc cây cú pháp và đích đã phân giải.">
<defs><marker id="fl32-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker><marker id="fl32-2-m" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="30" width="180" height="110" rx="10" class="f-surface s-muted"/><text x="34" y="60" class="t-b">Directive</text><text x="34" y="86" class="t-small">import hoặc</text><text x="34" y="106" class="t-small">export, kể cả URI</text><text x="34" y="126" class="t-small">trong if (...)</text>
<rect x="240" y="30" width="180" height="110" rx="10" class="f-phys-soft s-phys"/><text x="254" y="60" class="t-b">File này</text><text x="254" y="86" class="t-small">nằm dưới lib/</text><text x="254" y="106" class="t-small">presentation/ ?</text>
<rect x="460" y="30" width="180" height="110" rx="10" class="f-pk-soft s-pk"/><text x="474" y="60" class="t-b">Đích</text><text x="474" y="86" class="t-small">đã phân giải nằm</text><text x="474" y="106" class="t-small">dưới lib/data/ ?</text>
<rect x="680" y="30" width="180" height="110" rx="10" class="f-warn-soft s-warn"/><text x="694" y="60" class="t-b">Báo lint</text><text x="694" y="86" class="t-small">tại chuỗi URI</text><text x="694" y="106" class="t-small">(reportAtNode)</text>
<path d="M202 85 L238 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl32-2-a)"/>
<path d="M422 85 L458 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl32-2-a)"/>
<path d="M642 85 L678 85" class="f-none s-pk" stroke-width="2" marker-end="url(#fl32-2-a)"/>
<text x="204" y="76" class="t-small t-pk">có</text><text x="424" y="76" class="t-small t-pk">có</text><text x="644" y="76" class="t-small t-pk">có</text>
<rect x="240" y="220" width="400" height="50" rx="10" class="f-surface s-muted"/><text x="256" y="250" class="t-small">Không: bỏ qua, rule im lặng</text>
<path d="M330 142 L330 218" class="f-none s-muted" stroke-width="2" marker-end="url(#fl32-2-m)"/>
<path d="M550 142 L550 218" class="f-none s-muted" stroke-width="2" marker-end="url(#fl32-2-m)"/>
<text x="338" y="186" class="t-small">không</text><text x="558" y="186" class="t-small">không</text>
<text x="20" y="312" class="t-small">Bài 21 đọc chữ trên dòng bằng regex. Rule này đọc cây cú pháp và đích đã phân giải.</text>
</svg>
</div>
<figcaption>Hình 2. Hai phép kiểm lớp, một lần báo. Mọi biến thể cách viết import quy về cùng đường đi này.</figcaption>
</figure>

### Test bằng công cụ chính thức

`analyzer_testing` cung cấp `AnalysisRuleTest`: ta ghi file giả vào hệ file ảo rồi khẳng định chẩn đoán ở đúng offset và độ dài. Phương thức test mang tên `test_...` vì dùng `test_reflective_loader`.

### no_data_import_in_presentation_test.dart · 32-lint-analyzer/lint_plugin/test/no_data_import_in_presentation_test.dart (trích)

```dart
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
```

Một ca đáng xem là import có điều kiện, đúng chỗ regex của bài 21 bó tay:

### no_data_import_in_presentation_test.dart · 32-lint-analyzer/lint_plugin/test/no_data_import_in_presentation_test.dart (trích)

```dart
  void test_conditional_import_configuration() => expectInPresentation(
    "import '../domain/trip_repository.dart'\n"
    "    if (dart.library.io) '../data/trip_repository_impl.dart';\n"
    'Port? a;\n',
    ['../data/trip_repository_impl.dart'],
  );
```

Mười một test phủ các cách viết import, `export`, hai vi phạm một file, thư mục con của presentation, và các ca không được báo (domain, `dart:`, data, composition root, package khác có `lib/data`). Ta cũng phá thật: đổi `'data'` trong rule thành chuỗi khác thì test đỏ. Một nhóm test khác chạy `dart analyze` thật trên `fixtures/layers`: plugin được nạp, warning ra đúng file, mã thoát 2; còn `layers_off` khai báo plugin mà không bật rule thì im lặng.

## dart format, dart fix và ignore có lý do

**`dart format`.** Kiểu trình bày phụ thuộc language version: từ 3.7 là kiểu mới (mỗi tham số một dòng, có dấu phẩy cuối), trước đó là kiểu cũ; test dùng hai package tạm chỉ khác `sdk: ^3.6.0` và `^3.7.0`. Bẫy: `dart format` đọc language version từ `.dart_tool/package_config.json`, không đọc thẳng `pubspec.yaml`, nên đổi sdk constraint mà chưa `pub get` thì formatter vẫn dùng kiểu cũ. Ở CI và pre-commit, dùng `dart format --output=none --set-exit-if-changed`: thoát 1 khi có file cần format và không ghi gì.

**`dart fix`.** `--dry-run` liệt kê, `--apply` sửa, và chỉ với chẩn đoán có fix đi kèm và lint đã bật. Trên `fixtures/fixable` (4 lint bật), `--dry-run` đề xuất 4 sửa mà không đổi file; `--apply` sắp lại import, đổi `var` thành `final`, nháy kép thành nháy đơn, bỏ `this.` thừa, rồi `dart analyze` sạch. Tài liệu của `analysis_server_plugin` nói fix do plugin đăng ký không áp dụng hàng loạt được; plugin của ta không có fix nào.

**`// ignore` có lý do.** Ignore không lý do là món nợ không ai nhớ. Lab bật hai lint của SDK để giữ ta trung thực: `document_ignores` (từ Dart 3.5) đòi mỗi ignore có lời giải thích, `unnecessary_ignore` (từ Dart 3.8) báo ignore mà chẩn đoán không còn xuất hiện.

### ignores.dart · 32-lint-analyzer/fixtures/ignores/lib/ignores.dart

```dart
// ignore: unused_element
int _undocumented = 1;

// Bảng cước dự phòng, bài sau mới dùng tới.
// ignore: unused_element
int _documented = 2;

int _trailing = 3; // ignore: unused_element -- lý do viết cùng dòng

// Hằng này từng là private; ignore còn sót lại.
// ignore: unused_element
int stale = 4;
```

File này cho đúng hai info: `document_ignores` ở dòng 1 (không lý do) và `unnecessary_ignore` ở dòng 11 (`stale` là public nên không có gì để chặn). Lý do ở dòng ngay trên (dòng 4) hoặc cùng dòng sau `--` (dòng 8) đều được chấp nhận. Với lint của plugin, tên trần không đủ:

### ignored_screen.dart · 32-lint-analyzer/fixtures/layers/lib/presentation/ignored_screen.dart (trích)

```dart
// Màn hình thử nội bộ, sẽ đổi sang provider ở bài sau.
// ignore: toi_noi_lints/no_data_import_in_presentation
import '../data/fare_table_repository.dart';
```

Tiền tố `toi_noi_lints/` là điều kiện: một file mẫu dùng `// ignore: no_data_import_in_presentation` không tiền tố vẫn bị báo, file ở trên thì im lặng (test kiểm cả hai).

## Pre-commit và ngân sách bằng 0

**Ngân sách bằng 0** nghĩa là `dart analyze --fatal-infos` phải thoát 0, kể cả với chẩn đoán của plugin, và không có danh sách "lỗi cũ được phép". Chẩn đoán mới chỉ có hai lối: sửa code, hoặc thêm `// ignore` có lý do để review đọc được. Với codebase cũ, hãy đếm số chẩn đoán hiện có và giảm dần; ngân sách 0 chỉ áp ngay cho code mới.

Cổng đầu tiên sau editor là pre-commit. Lab có một script `sh` đơn giản, không tự cài và không đụng hook nào của repository:

### pre-commit · 32-lint-analyzer/tool/pre-commit (trích)

```sh
# Khi git gọi hook, nó đặt GIT_DIR (và GIT_WORK_TREE). Trong worktree liên kết
# giá trị đó không còn khớp với thư mục lab sau `cd`, làm `dart format` đọc sai
# đường dẫn mà vẫn thoát 0. Bỏ chúng để git tự tìm repo từ thư mục lab.
# GIT_INDEX_FILE thì giữ: `git commit <file>` dùng một index tạm.
unset GIT_DIR GIT_WORK_TREE

# core.quotePath=false: tên file có dấu không bị bọc nháy và escape.
# ACMR: thêm, sửa, copy và đổi tên (R); bỏ file đã xoá.
staged="$(git -c core.quotePath=false diff --cached --name-only \
  --diff-filter=ACMR --relative -- . | grep '\.dart$' || true)"
# ...
for file in $staged; do
  code=0
  out="$(dart format --output=none --set-exit-if-changed "$file" 2>&1)" ||
    code=$?
  case "$code" in
  0) ;;
  # Mã 1 của --set-exit-if-changed: file sẽ bị đổi, tức chưa format.
  1) unformatted="$unformatted$file
" ;;
  # Mã khác (ví dụ 65 khi file có lỗi cú pháp): không phải chuyện format.
  *) broken="$broken$file: $(printf '%s' "$out" | head -n 1)
" ;;
  esac
done
# ...
${ANALYZE_CMD:-dart analyze --fatal-infos}
```

Bản đầu của script từng lọt cổng ba cách, và test bằng `git commit` thật trong repo tạm bắt cả ba. Git đặt `GIT_DIR` khi gọi hook; trong worktree liên kết giá trị đó làm `dart format` đọc sai đường dẫn mà vẫn thoát 0, nên script `unset` nó. File đổi tên lọt khi bộ lọc thiếu `R`, và tên file có dấu bị bọc nháy khi `core.quotePath` còn bật. Script cũng tách mã 1 (chưa format) khỏi mã khác của `dart format`, như 65 khi file có lỗi cú pháp. Hai giới hạn còn lại: nó kiểm file trên đĩa chứ không phải bản trong index, và `git commit --no-verify` bỏ qua nó. Cổng cuối là CI ở [bài 33](/articles/flutter-ci-github-actions-analyze-test-golden-cache).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Hình 3. Ba cổng chặn theo thời gian, từ trái sang phải. Cổng 1 là editor: analysis server chạy khi gõ, mỗi file. Cổng 2 là pre-commit: file .dart đang staged, dart format kiểm và dart analyze --fatal-infos. Cổng 3 là CI ở bài 33: toàn bộ repository, không phụ thuộc máy người viết. Mũi tên git commit nối cổng 1 với 2, git push nối cổng 2 với 3.">
<defs><marker id="fl32-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<rect x="20" y="30" width="240" height="140" rx="10" class="f-phys-soft s-phys"/><text x="34" y="60" class="t-b">1. Editor</text><text x="34" y="88" class="t-small">analysis server chạy</text><text x="34" y="108" class="t-small">khi ta gõ, từng file</text><text x="34" y="128" class="t-small">format khi lưu</text>
<rect x="320" y="30" width="240" height="140" rx="10" class="f-pk-soft s-pk"/><text x="334" y="60" class="t-b">2. pre-commit</text><text x="334" y="88" class="t-small">file .dart đang staged</text><text x="334" y="108" class="t-small">dart format kiểm, rồi</text><text x="334" y="128" class="t-small">analyze --fatal-infos</text>
<rect x="620" y="30" width="240" height="140" rx="10" class="f-surface s-line"/><text x="634" y="60" class="t-b">3. CI (bài 33)</text><text x="634" y="88" class="t-small">toàn bộ repository,</text><text x="634" y="108" class="t-small">không phụ thuộc máy</text><text x="634" y="128" class="t-small">của người viết</text>
<path d="M262 100 L318 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl32-3-a)"/>
<path d="M562 100 L618 100" class="f-none s-pk" stroke-width="2" marker-end="url(#fl32-3-a)"/>
<text x="266" y="90" class="t-small t-pk">commit</text><text x="574" y="90" class="t-small t-pk">push</text>
<text x="20" y="230" class="t-small">Càng sang phải, phản hồi càng muộn nhưng càng khó lách.</text>
<text x="20" y="254" class="t-small">Cổng 1 và 2 chạy trên máy ta nên bỏ qua được; cổng 3 chạy ở máy chủ nên khó lách hơn.</text>
</svg>
</div>
<figcaption>Hình 3. Editor phản hồi theo từng file, còn pre-commit và CI cùng chạy dart analyze --fatal-infos; ba cổng khác nhau ở thời điểm và ở mức dễ bỏ qua.</figcaption>
</figure>

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp với lint</span>
<ul>
<li><strong>Dùng <code>flutter analyze</code> làm cổng khi có plugin.</strong> Dấu hiệu: IDE đỏ, CI xanh. Nguyên nhân: trên Flutter 3.47.4, nó không in chẩn đoán của plugin. Cách sửa: chạy <code>dart analyze --fatal-infos</code> ở CI và pre-commit.</li>
<li><strong>Thêm <code>custom_lint</code> theo hướng dẫn cũ.</strong> Dấu hiệu: pub kéo <code>riverpod_generator</code> và <code>analyzer</code> về bản cũ, hoặc báo xung đột. Nguyên nhân: nó buộc <code>analyzer ^8.0.0</code>. Cách sửa: bỏ nó, dùng khối <code>plugins:</code>.</li>
<li><strong>Lint của plugin im lặng.</strong> Dấu hiệu: vi phạm rõ ràng mà <code>dart analyze</code> sạch và thoát 0. Nguyên nhân: lint rule của plugin tắt mặc định, thiếu <code>diagnostics:</code>; chưa khởi động lại server sau khi sửa <code>plugins:</code>; hoặc ta chỉ định thư mục con, vì <code>dart analyze --fatal-infos lib</code> không báo chẩn đoán của plugin còn không đối số, <code>.</code> và đường dẫn một file thì báo (test kiểm). Cách sửa: bật rule, khởi động lại, chạy không đối số hoặc <code>.</code> ở thư mục package.</li>
<li><strong>Ignore thiếu tiền tố cho lint của plugin.</strong> Dấu hiệu: đã viết <code>// ignore: tên_rule</code> mà vẫn bị báo. Nguyên nhân: rule của plugin cần dạng <code>plugin/rule</code>. Cách sửa: <code>// ignore: toi_noi_lints/no_data_import_in_presentation</code> kèm lý do.</li>
<li><strong>Format khác nhau giữa các máy.</strong> Dấu hiệu: pre-commit chặn vì chưa format, nhưng <code>dart format .</code> ở máy khác không đổi gì. Nguyên nhân: đổi <code>sdk</code> mà chưa <code>pub get</code> thì <code>package_config.json</code> giữ language version cũ. Cách sửa: <code>pub get</code> sau khi đổi SDK constraint.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/32-lint-analyzer
flutter pub get
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
flutter test

cd lint_plugin
dart pub get
dart analyze --fatal-infos
dart test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3: lab và `lint_plugin` đều `No issues found!` với `--fatal-infos`, `flutter test` 41 test xanh, `dart test` trong `lint_plugin` 11 test xanh. Mức kiểm chứng: **Kiểm chứng tự động**, không cần thiết bị, nhưng cần pub.dev: mỗi lần phân tích có plugin cần với tới nó, nên bộ test không chạy offline. Phiên bản: `flutter_lints` 6.0.0, `riverpod_lint` 3.2.0, `analysis_server_plugin` 0.3.26, `analyzer` 14.7.0, `analyzer_testing` 0.4.5, `flutter_riverpod` 3.5.0.

- App (`quote_screen_test`): theo bảng giá minh hoạ, 25.500 VND ở 5 km, 30.000 VND ở 6 km, 453.000 VND ở 100 km; ngoài miền là lỗi.
- Ba cờ strict (`strict_analysis_test`): 5 chẩn đoán cho mã "trước", 0 cho mã "sau" và cho mã "trước" khi không có cờ.
- Công cụ Dart (`dart_tools_test`): `dart format` theo language version, `dart fix`, `document_ignores`, `unnecessary_ignore`.
- Plugin trong `dart analyze` thật (`layer_plugin_test`): warning đúng file, ignore có tiền tố, rule tắt mặc định, bẫy chỉ định thư mục `lib`, mã thoát 4 khi mất pub.dev.
- Pre-commit (`pre_commit_test`): script trong repo git tạm với `git commit` thật, gồm file đổi tên, tên có dấu, lỗi cú pháp và worktree liên kết.

**Chưa làm**: chạy trên thiết bị; CI chạy các lệnh trên (bài 33); quick fix cho lint của plugin; cấu hình danh sách lớp thay vì hằng trong rule; test tự động cho `riverpod_lint`. `analysis_server_plugin` và `analyzer_testing` còn dưới 1.0, nên nâng Dart có thể buộc nâng plugin.

## Bài tập

1. Ta bỏ `severity: DiagnosticSeverity.WARNING` khỏi `LintCode`. Mã thoát của `dart analyze` khi có một vi phạm là bao nhiêu, và cần cờ nào để CI vẫn thất bại?
2. File presentation import `data` và có `// ignore: no_data_import_in_presentation` ngay trên. Lint còn báo không? Sửa thế nào?
3. Pre-commit báo một file là "chưa format", nhưng chạy `dart format .` ngay trên cùng máy thì không file nào đổi. Nêu một nguyên nhân và cách xác nhận.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Rule thành info (mức mặc định của <code>LintCode</code>), nên <code>dart analyze</code> thoát 0 dù có vi phạm; thêm <code>--fatal-infos</code> thì thoát 1. Lab đo cặp 0 và 1 này với <code>document_ignores</code>.</p>
<p><strong>2.</strong> Vẫn báo, vì tên trần không chặn được rule của plugin. Viết <code>// ignore: toi_noi_lints/no_data_import_in_presentation</code> kèm lý do ở dòng trên. Nhưng hãy hỏi trước ignore có cần không: thường đáp án đúng là đi qua interface của domain.</p>
<p><strong>3.</strong> File có lỗi cú pháp: <code>dart format</code> thoát 65 chứ không phải 1 và không sửa gì. Bản script đầu coi mọi mã khác 0 là "chưa format" và nuốt output. Xác nhận bằng <code>dart format --output=none --set-exit-if-changed</code> trên file đó rồi xem mã thoát; script hiện tại tách mã 1 khỏi mã khác và in dòng lỗi đầu tiên.</p>
</details>

Bài trước: [Integration test và patrol: luồng đặt chuyến trên simulator/emulator](/articles/flutter-integration-test-patrol-luong-dat-chuyen-tren-thiet-bi). Bài tiếp: [CI với GitHub Actions: analyze, test, golden và cache pub](/articles/flutter-ci-github-actions-analyze-test-golden-cache).

## Tài liệu gốc

- [Customizing static analysis](https://dart.dev/tools/analysis) và [Linter rules](https://dart.dev/tools/linter-rules) (dart.dev).
- [Analyzer plugins](https://dart.dev/tools/analyzer-plugins) (dart.dev), cùng [analysis_server_plugin](https://pub.dev/packages/analysis_server_plugin) và [analyzer_testing](https://pub.dev/packages/analyzer_testing) (pub.dev).
- [document_ignores](https://dart.dev/tools/linter-rules/document_ignores) và [unnecessary_ignore](https://dart.dev/tools/linter-rules/unnecessary_ignore) (dart.dev).
- [dart format](https://dart.dev/tools/dart-format) và [dart fix](https://dart.dev/tools/dart-fix) (dart.dev).
- [riverpod_lint](https://pub.dev/packages/riverpod_lint) và [changelog](https://pub.dev/packages/riverpod_lint/changelog), [custom_lint](https://pub.dev/packages/custom_lint) (pub.dev).
- [flutter_lints](https://pub.dev/packages/flutter_lints), [lints](https://pub.dev/packages/lints) và [very_good_analysis](https://pub.dev/packages/very_good_analysis) (pub.dev).
