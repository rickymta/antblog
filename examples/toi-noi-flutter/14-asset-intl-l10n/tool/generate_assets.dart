// Sinh biểu tượng PNG của lab: cùng một hình vẽ ở ba độ phân giải.
//
//   dart tool/generate_assets.dart
//
// Hình do chính script này vẽ (ô vuông bo góc màu seed của theme, vòng tròn
// và chấm trắng như một chiếc ghim), nên không dính bản quyền ảnh của ai.
// Chỉ dùng `dart:io`, tự ghi PNG RGBA 8 bit, không cần package ảnh.
import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';

/// Kích thước điểm ảnh của từng bản: 48 dp ở 1.0x, gấp đôi ở 2.0x, gấp ba ở 3.0x.
const Map<String, int> variants = {
  'assets/images/logo.png': 48,
  'assets/images/2.0x/logo.png': 96,
  'assets/images/3.0x/logo.png': 144,
};

const int seedRed = 0x0F;
const int seedGreen = 0x76;
const int seedBlue = 0x6E;

void main() {
  variants.forEach((path, size) {
    final file = File(path)..createSync(recursive: true);
    file.writeAsBytesSync(encodePng(size, draw(size)));
    stdout.writeln('$path  ${size}x$size  ${file.lengthSync()} byte');
  });
}

/// Vẽ hình vuông cạnh [size] px, mỗi điểm ảnh lấy 4x4 mẫu để mép mịn.
Uint8List draw(int size) {
  final rgba = Uint8List(size * size * 4);
  const samples = 4;
  for (var y = 0; y < size; y++) {
    for (var x = 0; x < size; x++) {
      var coverBackground = 0;
      var coverWhite = 0;
      for (var sy = 0; sy < samples; sy++) {
        for (var sx = 0; sx < samples; sx++) {
          // Toạ độ chuẩn hoá trong [0, 1], không phụ thuộc kích thước.
          final u = (x + (sx + 0.5) / samples) / size;
          final v = (y + (sy + 0.5) / samples) / size;
          if (insideRoundedSquare(u, v)) {
            coverBackground++;
            if (insidePin(u, v)) coverWhite++;
          }
        }
      }
      const total = samples * samples;
      final alpha = (255 * coverBackground / total).round();
      final white = coverBackground == 0 ? 0.0 : coverWhite / coverBackground;
      final i = (y * size + x) * 4;
      rgba[i] = (seedRed + (255 - seedRed) * white).round();
      rgba[i + 1] = (seedGreen + (255 - seedGreen) * white).round();
      rgba[i + 2] = (seedBlue + (255 - seedBlue) * white).round();
      rgba[i + 3] = alpha;
    }
  }
  return rgba;
}

bool insideRoundedSquare(double u, double v) {
  const radius = 0.22;
  final dx = math.max(0.0, math.max(radius - u, u - (1 - radius)));
  final dy = math.max(0.0, math.max(radius - v, v - (1 - radius)));
  return dx * dx + dy * dy <= radius * radius;
}

/// Vòng tròn trắng dày ở giữa, bên trong có chấm đặc.
bool insidePin(double u, double v) {
  final d = math.sqrt((u - 0.5) * (u - 0.5) + (v - 0.5) * (v - 0.5));
  return (d >= 0.24 && d <= 0.34) || d <= 0.10;
}

Uint8List encodePng(int size, Uint8List rgba) {
  // Mỗi dòng bắt đầu bằng một byte filter (0 = không lọc).
  final raw = BytesBuilder();
  for (var y = 0; y < size; y++) {
    raw.addByte(0);
    raw.add(rgba.sublist(y * size * 4, (y + 1) * size * 4));
  }
  final header = ByteData(13)
    ..setUint32(0, size)
    ..setUint32(4, size)
    ..setUint8(8, 8) // 8 bit mỗi kênh
    ..setUint8(9, 6); // RGBA
  final out = BytesBuilder()
    ..add(const [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    ..add(chunk('IHDR', header.buffer.asUint8List()))
    ..add(chunk('IDAT', ZLibEncoder().convert(raw.toBytes())))
    ..add(chunk('IEND', Uint8List(0)));
  return out.toBytes();
}

Uint8List chunk(String type, List<int> data) {
  final typeAndData = [...type.codeUnits, ...data];
  final length = ByteData(4)..setUint32(0, data.length);
  final crc = ByteData(4)..setUint32(0, crc32(typeAndData));
  return Uint8List.fromList([
    ...length.buffer.asUint8List(),
    ...typeAndData,
    ...crc.buffer.asUint8List(),
  ]);
}

int crc32(List<int> bytes) {
  var crc = 0xFFFFFFFF;
  for (final byte in bytes) {
    crc ^= byte;
    for (var bit = 0; bit < 8; bit++) {
      crc = (crc & 1) != 0 ? (crc >> 1) ^ 0xEDB88320 : crc >> 1;
    }
  }
  return crc ^ 0xFFFFFFFF;
}
