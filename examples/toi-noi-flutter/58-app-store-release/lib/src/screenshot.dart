import 'dart:typed_data';

/// Thông tin đọc từ phần đầu file PNG: đủ để kiểm kích thước và kênh alpha mà
/// không cần giải mã ảnh.
final class PngInfo {
  const PngInfo({
    required this.width,
    required this.height,
    required this.hasAlpha,
  });

  /// Đọc [bytes]; ném [FormatException] nếu không phải PNG.
  ///
  /// Chữ ký 8 byte, rồi chunk `IHDR` (rộng 4 byte, cao 4 byte, bit depth,
  /// colour type). Colour type 4 và 6 có kênh alpha; chunk `tRNS` cũng thêm
  /// độ trong suốt cho các colour type 0, 2, 3, nên phải duyệt chunk.
  factory PngInfo.parse(Uint8List bytes) {
    const signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if (bytes.length < 33) {
      throw const FormatException('PNG: file quá ngắn');
    }
    for (var i = 0; i < signature.length; i++) {
      if (bytes[i] != signature[i]) {
        throw const FormatException('PNG: sai chữ ký');
      }
    }
    final data = ByteData.sublistView(bytes);
    if (String.fromCharCodes(bytes.sublist(12, 16)) != 'IHDR') {
      throw const FormatException('PNG: chunk đầu không phải IHDR');
    }
    final width = data.getUint32(16);
    final height = data.getUint32(20);
    final colorType = bytes[25];
    var hasAlpha = colorType == 4 || colorType == 6;

    // Duyệt các chunk sau IHDR để tìm tRNS; dừng ở IDAT vì tRNS phải đứng trước.
    var offset = 33;
    while (!hasAlpha && offset + 8 <= bytes.length) {
      final length = data.getUint32(offset);
      final type = String.fromCharCodes(bytes.sublist(offset + 4, offset + 8));
      if (type == 'tRNS') hasAlpha = true;
      if (type == 'IDAT' || type == 'IEND') break;
      offset += 12 + length;
    }
    return PngInfo(width: width, height: height, hasAlpha: hasAlpha);
  }

  final int width;
  final int height;
  final bool hasAlpha;
}
