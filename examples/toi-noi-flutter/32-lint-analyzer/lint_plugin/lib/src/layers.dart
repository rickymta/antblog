import 'package:path/path.dart' as p;

/// Lớp của [filePath] trong package gốc [packageRoot]: tên thư mục con đầu
/// tiên dưới `lib/`, hoặc `null` nếu file không nằm dưới `lib/<lớp>/`
/// (ví dụ `lib/main.dart`, hay file của package khác).
String? layerOf(p.Context context, String packageRoot, String filePath) {
  final libDir = context.join(packageRoot, 'lib');
  if (!context.isWithin(libDir, filePath)) {
    return null;
  }
  final parts = context.split(context.relative(filePath, from: libDir));
  return parts.length > 1 ? parts.first : null;
}
