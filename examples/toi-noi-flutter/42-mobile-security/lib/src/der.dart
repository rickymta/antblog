import 'dart:typed_data';

/// Lấy nguyên khối `SubjectPublicKeyInfo` (cả tag và độ dài) từ chứng chỉ
/// X.509 DER.
///
/// Cấu trúc theo RFC 5280:
/// `Certificate ::= SEQUENCE { tbsCertificate, signatureAlgorithm, signature }`
/// và `tbsCertificate ::= SEQUENCE { [0] version OPTIONAL, serialNumber,
/// signature, issuer, validity, subject, subjectPublicKeyInfo, ... }`.
/// Hàm đi qua các phần tử cho tới SPKI, không giải nghĩa nội dung.
///
/// Ném [FormatException] khi DER hỏng hoặc không đúng cấu trúc.
Uint8List extractSubjectPublicKeyInfo(Uint8List der) {
  final certificate = _readElement(der, 0);
  _expectTag(certificate, 0x30, 'Certificate');

  final tbs = _readElement(der, certificate.contentStart);
  _expectTag(tbs, 0x30, 'tbsCertificate');

  var offset = tbs.contentStart;
  // version là [0] EXPLICIT và có thể vắng (v1).
  var element = _readElement(der, offset);
  if (element.tag == 0xA0) {
    offset = element.end;
    element = _readElement(der, offset);
  }
  // serialNumber, signature, issuer, validity, subject.
  for (var i = 0; i < 5; i++) {
    offset = element.end;
    element = _readElement(der, offset);
  }
  _expectTag(element, 0x30, 'subjectPublicKeyInfo');
  return Uint8List.sublistView(der, element.start, element.end);
}

final class _Element {
  const _Element(this.tag, this.start, this.contentStart, this.end);

  final int tag;
  final int start;
  final int contentStart;
  final int end;
}

void _expectTag(_Element element, int tag, String name) {
  if (element.tag != tag) {
    throw FormatException(
      '$name: tag 0x${element.tag.toRadixString(16)}, mong 0x'
      '${tag.toRadixString(16)}',
    );
  }
}

_Element _readElement(Uint8List data, int offset) {
  if (offset + 2 > data.length) {
    throw const FormatException('DER bị cắt cụt');
  }
  final tag = data[offset];
  final first = data[offset + 1];
  var length = first;
  var headerLength = 2;
  if (first & 0x80 != 0) {
    final lengthBytes = first & 0x7F;
    // 0 là độ dài vô định (BER), và quá 4 byte thì không thực tế cho chứng chỉ.
    if (lengthBytes == 0 || lengthBytes > 4) {
      throw FormatException('Độ dài DER không hỗ trợ: $lengthBytes byte');
    }
    if (offset + 2 + lengthBytes > data.length) {
      throw const FormatException('DER bị cắt cụt');
    }
    length = 0;
    for (var i = 0; i < lengthBytes; i++) {
      length = (length << 8) | data[offset + 2 + i];
    }
    headerLength = 2 + lengthBytes;
  }
  final end = offset + headerLength + length;
  if (end > data.length) {
    throw const FormatException('Độ dài DER vượt quá dữ liệu');
  }
  return _Element(tag, offset, offset + headerLength, end);
}
