import 'package:analyzer/analysis_rule/analysis_rule.dart';
import 'package:analyzer/analysis_rule/rule_context.dart';
import 'package:analyzer/analysis_rule/rule_visitor_registry.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:analyzer/dart/element/element.dart';
import 'package:analyzer/error/error.dart';

import 'layers.dart';

/// Cấm file dưới `lib/presentation/` import hoặc export file dưới `lib/data/`
/// của cùng package.
///
/// Rule đọc đích mà analyzer đã phân giải, nên bắt được import tương đối,
/// `package:` của chính package, `export`, và URI trong `if (...)`.
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

class _Visitor extends SimpleAstVisitor<void> {
  _Visitor(this.rule, this.context);

  final AnalysisRule rule;
  final RuleContext context;

  @override
  void visitImportDirective(ImportDirective node) =>
      _check(node, node.libraryImport?.uri);

  @override
  void visitExportDirective(ExportDirective node) =>
      _check(node, node.libraryExport?.uri);

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
}
