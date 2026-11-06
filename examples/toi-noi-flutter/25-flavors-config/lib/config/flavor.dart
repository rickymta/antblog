/// Ba môi trường cố định của Tới Nơi (bài viết, mục 1).
///
/// Tên enum trùng tên file `config/<tên>.json` và tên flavor native ở bài 50.
enum Flavor {
  dev,
  staging,
  prod;

  /// Tra theo tên đúng từng chữ; `Prod` hay ` prod` không được đoán thành `prod`.
  static Flavor? tryParse(String name) => values.asNameMap()[name];

  /// Chỉ `dev` được dùng `http` và máy cục bộ.
  bool get allowsLocalBackend => this == Flavor.dev;
}
