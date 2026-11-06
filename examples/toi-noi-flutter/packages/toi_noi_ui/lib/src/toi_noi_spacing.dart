/// Các mức khoảng cách (logical pixel) của design system Tới Nơi.
///
/// Dùng hằng thay cho số tự do (`8.0`, `16.0`) để mọi màn hình cùng một nhịp
/// và đổi nhịp ở một chỗ.
abstract final class ToiNoiSpacing {
  /// 4: khoảng cách giữa icon và chữ trong cùng một chip.
  static const double xs = 4;

  /// 8: khoảng cách giữa hai phần tử liền nhau trong một nhóm.
  static const double sm = 8;

  /// 16: lề màn hình và khoảng cách giữa các nhóm.
  static const double md = 16;

  /// 24: khoảng cách giữa hai khối lớn.
  static const double lg = 24;

  /// 32: khoảng cách trước một tiêu đề phần.
  static const double xl = 32;
}
