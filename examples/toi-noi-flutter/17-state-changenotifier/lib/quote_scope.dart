import 'package:flutter/widgets.dart';

import 'quote_draft.dart';

/// Đưa một [QuoteDraft] xuống cây widget.
///
/// Dựa trên [InheritedNotifier]: widget con gọi [QuoteScope.of] sẽ được dựng
/// lại mỗi khi draft gọi `notifyListeners`. Widget chỉ cần *gọi hành động*
/// hoặc tự lắng nghe một phần thì dùng [QuoteScope.read] để không đăng ký
/// phụ thuộc.
class QuoteScope extends InheritedNotifier<QuoteDraft> {
  const QuoteScope({super.key, required QuoteDraft draft, required super.child})
    : super(notifier: draft);

  /// Draft gần nhất phía trên, hoặc `null` nếu không có scope.
  ///
  /// Đăng ký `context` làm dependent: dựng lại khi draft đổi.
  static QuoteDraft? maybeOf(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<QuoteScope>()?.notifier;

  /// Như [maybeOf] nhưng ném [FlutterError] có thông điệp rõ ràng khi thiếu.
  static QuoteDraft of(BuildContext context) =>
      maybeOf(context) ?? (throw _missing(context));

  /// Lấy draft mà không đăng ký phụ thuộc: `context` không bị dựng lại khi
  /// draft đổi. Dùng trong trình xử lý sự kiện hoặc khi tự lắng nghe.
  static QuoteDraft read(BuildContext context) =>
      context.getInheritedWidgetOfExactType<QuoteScope>()?.notifier ??
      (throw _missing(context));

  static FlutterError _missing(BuildContext context) => FlutterError(
    'Không tìm thấy QuoteScope phía trên ${context.widget.runtimeType}.\n'
    'Đặt QuoteScope ở trên cùng của mọi widget cần đọc báo giá, '
    'kể cả trên MaterialApp nếu các route được đẩy cũng cần nó.',
  );
}
