import 'package:flutter/widgets.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'quote_draft.dart';

/// Ảnh chụp bất biến của [QuoteDraft]. So sánh theo giá trị.
@immutable
class QuoteSnapshot {
  const QuoteSnapshot({
    required this.service,
    required this.km,
    required this.fare,
  });

  QuoteSnapshot.of(QuoteDraft draft)
    : this(service: draft.service, km: draft.km, fare: draft.fare);

  final ServiceKind service;
  final double km;
  final Money fare;

  @override
  bool operator ==(Object other) =>
      other is QuoteSnapshot &&
      other.service == service &&
      other.km == km &&
      other.fare == fare;

  @override
  int get hashCode => Object.hash(service, km, fare);
}

/// `InheritedWidget` viết tay: chỉ mang một [QuoteSnapshot] bất biến.
///
/// `InheritedWidget` tự nó không biết "đổi": nó chỉ so widget cũ với widget
/// mới khi cha dựng lại. [updateShouldNotify] quyết định có đánh thức các
/// dependent hay không.
class QuoteSnapshotScope extends InheritedWidget {
  const QuoteSnapshotScope({
    super.key,
    required this.snapshot,
    required super.child,
  });

  final QuoteSnapshot snapshot;

  /// Đăng ký phụ thuộc rồi trả ảnh chụp hiện tại.
  static QuoteSnapshot of(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<QuoteSnapshotScope>();
    assert(scope != null, 'Thiếu QuoteSnapshotScope phía trên');
    return scope!.snapshot;
  }

  @override
  bool updateShouldNotify(QuoteSnapshotScope oldWidget) =>
      snapshot != oldWidget.snapshot;
}

/// Phần "có state" mà `InheritedWidget` còn thiếu: lắng nghe [draft], giữ
/// ảnh chụp trong `State` và dựng lại [QuoteSnapshotScope] mỗi khi draft đổi.
///
/// Đây là việc `InheritedNotifier` làm sẵn (bản thật làm ở tầng Element).
class QuoteSnapshotHost extends StatefulWidget {
  const QuoteSnapshotHost({
    super.key,
    required this.draft,
    required this.child,
  });

  final QuoteDraft draft;
  final Widget child;

  @override
  State<QuoteSnapshotHost> createState() => _QuoteSnapshotHostState();
}

class _QuoteSnapshotHostState extends State<QuoteSnapshotHost> {
  @override
  void initState() {
    super.initState();
    widget.draft.addListener(_onChanged);
  }

  @override
  void didUpdateWidget(QuoteSnapshotHost oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.draft != widget.draft) {
      oldWidget.draft.removeListener(_onChanged);
      widget.draft.addListener(_onChanged);
    }
  }

  @override
  void dispose() {
    widget.draft.removeListener(_onChanged);
    super.dispose();
  }

  void _onChanged() => setState(() {});

  @override
  Widget build(BuildContext context) => QuoteSnapshotScope(
    snapshot: QuoteSnapshot.of(widget.draft),
    child: widget.child,
  );
}
