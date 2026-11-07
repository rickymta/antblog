import 'dart:io' show ZLibEncoder;
import 'dart:typed_data';

/// Tạo ảnh PNG RGBA hợp lệ [width] x [height] (dải màu chuyển) bằng tay, để
/// test không cần file ảnh hay package ngoài.
Uint8List makePng(int width, int height) {
  final raw = BytesBuilder();
  for (var y = 0; y < height; y++) {
    raw.addByte(0); // filter type: None
    for (var x = 0; x < width; x++) {
      raw.add([x * 255 ~/ width, y * 255 ~/ height, 128, 255]);
    }
  }
  final out = BytesBuilder()
    ..add(const [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    ..add(
      _chunk('IHDR', [
        ..._u32(width),
        ..._u32(height),
        8, // bit depth
        6, // colour type: RGBA
        0, 0, 0,
      ]),
    )
    ..add(_chunk('IDAT', ZLibEncoder().convert(raw.toBytes())))
    ..add(_chunk('IEND', const []));
  return out.toBytes();
}

List<int> _u32(int v) => [
  (v >> 24) & 0xFF,
  (v >> 16) & 0xFF,
  (v >> 8) & 0xFF,
  v & 0xFF,
];

List<int> _chunk(String type, List<int> data) {
  final body = [...type.codeUnits, ...data];
  return [..._u32(data.length), ...body, ..._u32(_crc32(body))];
}

int _crc32(List<int> bytes) {
  var crc = 0xFFFFFFFF;
  for (final b in bytes) {
    crc ^= b;
    for (var i = 0; i < 8; i++) {
      crc = (crc & 1) != 0 ? (crc >> 1) ^ 0xEDB88320 : crc >> 1;
    }
  }
  return crc ^ 0xFFFFFFFF;
}
