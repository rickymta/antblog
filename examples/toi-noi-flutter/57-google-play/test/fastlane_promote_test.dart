import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_57/track_promotion.dart';

const String promotePath = 'native-config/fastlane/Fastfile.promote.example';

/// Chạy Ruby thật với bản giả của fastlane (UI, lane, upload_to_play_store...)
/// để thi hành lane `promote` và bắt tham số nó sẽ truyền cho supply. Đây là
/// kiểm logic của file, KHÔNG phải kiểm fastlane thật.
const String _harness = r'''
require "json"
class LaneError < StandardError; end
module UI
  def self.user_error!(message)
    raise LaneError, message
  end
end
$lanes = {}
$captured = nil
def platform(_name)
  yield
end
def desc(_text); end
def lane(name, &block)
  $lanes[name] = block
end
def env!(name)
  value = ENV[name].to_s.strip
  raise LaneError, "Thiếu biến môi trường #{name}" if value.empty?
  value
end
def pick_target(_options)
  ["staging", "vn.toinoi.customer.staging"]
end
def upload_to_play_store(**params)
  $captured = params
end
load ARGV[0]
options = JSON.parse(ARGV[1], symbolize_names: true)
begin
  $lanes.fetch(:promote).call(options)
  puts JSON.generate({ok: true, params: $captured})
rescue LaneError => e
  puts JSON.generate({ok: false, error: e.message})
end
''';

bool get _hasRuby =>
    Process.runSync('ruby', ['-v'], runInShell: true).exitCode == 0;

void main() {
  final text = File(promotePath).readAsStringSync();
  final skipRuby = _hasRuby ? null : 'máy không có ruby';

  group('nội dung file', () {
    test('bảng PROMOTIONS trùng TrackPromotion.allowed', () {
      final block = RegExp(
        r'PROMOTIONS = \{(.*?)\}\.freeze',
        dotAll: true,
      ).firstMatch(text)!.group(1)!;
      final fromRuby = <String, Set<String>>{
        for (final m in RegExp(r'"(\w+)" => %w\[([^\]]*)\]').allMatches(block))
          m.group(1)!: m.group(2)!.split(' ').toSet(),
      };
      final fromDart = <String, Set<String>>{
        for (final e in TrackPromotion.allowed.entries)
          if (e.value.isNotEmpty) e.key.name: {for (final t in e.value) t.name},
      };
      expect(fromRuby, fromDart);
    });

    test('TRACK_NAMES có đủ bốn track và production giữ nguyên tên', () {
      final block = RegExp(
        r'TRACK_NAMES = \{(.*?)\}\.freeze',
        dotAll: true,
      ).firstMatch(text)!.group(1)!;
      final names = {
        for (final m in RegExp(r'"(\w+)" => "(\w+)"').allMatches(block))
          m.group(1)!: m.group(2)!,
      };
      expect(names.keys.toSet(), {for (final t in Track.values) t.name});
      expect(names['production'], 'production');
      expect(names['internal'], 'internal');
    });

    test('khoá Play đến từ env!, không có giá trị gõ thẳng', () {
      expect(text, contains('json_key_data = env!("PLAY_JSON_KEY_DATA")'));
      expect(text, isNot(contains('-----BEGIN')));
      expect(text, isNot(contains('"private_key"')));
    });

    test('lane chỉ promote: bỏ qua mọi bước tải lên', () {
      for (final key in [
        'skip_upload_apk',
        'skip_upload_aab',
        'skip_upload_metadata',
        'skip_upload_images',
        'skip_upload_screenshots',
        'skip_upload_changelogs',
      ]) {
        expect(text, contains('$key: true'));
      }
      expect(text, contains('track_promote_to:'));
    });

    test('cú pháp Ruby hợp lệ (ruby -c)', () {
      final r = Process.runSync('ruby', ['-c', promotePath]);
      expect(r.exitCode, 0, reason: '${r.stderr}');
      expect(r.stdout.toString(), contains('Syntax OK'));
    }, skip: skipRuby);
  });

  group('thi hành lane với fastlane giả', () {
    late Directory tmp;
    late String harness;

    setUpAll(() {
      tmp = Directory.systemTemp.createTempSync('toi_noi_57_');
      harness = '${tmp.path}/harness.rb';
      File(harness).writeAsStringSync(_harness);
    });
    tearDownAll(() => tmp.deleteSync(recursive: true));

    Map<String, Object?> run(
      Map<String, String> options, {
      bool withKey = true,
    }) {
      options = {'version_code': '43', ...options};
      final r = Process.runSync(
        'ruby',
        [harness, promotePath, jsonEncode(options)],
        environment: {'PLAY_JSON_KEY_DATA': withKey ? '{"fake":"test"}' : ''},
      );
      expect(r.exitCode, 0, reason: '${r.stderr}');
      return jsonDecode(r.stdout.toString()) as Map<String, Object?>;
    }

    test('internal -> closed: đúng track nguồn và đích, không có rollout', () {
      final out = run({'from': 'internal', 'to': 'closed'});
      expect(out['ok'], isTrue);
      final params = out['params']! as Map<String, Object?>;
      expect(params['track'], 'internal');
      expect(params['track_promote_to'], 'alpha');
      expect(params['package_name'], 'vn.toinoi.customer.staging');
      expect(params.containsKey('rollout'), isFalse);
    }, skip: skipRuby);

    test('version_code được chuyển cho supply; thiếu hoặc sai thì từ chối', () {
      final ok = run({'from': 'internal', 'to': 'closed'});
      expect((ok['params']! as Map)['version_code'], '43');
      for (final bad in ['', '0', '-1', 'abc', '2100000001', '4.3']) {
        final out = run({
          'from': 'internal',
          'to': 'closed',
          'version_code': bad,
        });
        expect(out['ok'], isFalse, reason: 'version_code="$bad"');
      }
      expect(
        (run({
              'from': 'internal',
              'to': 'closed',
              'version_code': '2100000000',
            })['params']!
            as Map)['version_code'],
        '2100000000',
      );
    }, skip: skipRuby);

    test('closed -> production với 1%: rollout là tỉ lệ 0,01', () {
      final out = run({
        'from': 'closed',
        'to': 'production',
        'rollout_percent': '1',
      });
      final params = out['params']! as Map<String, Object?>;
      expect(params['track_promote_to'], 'production');
      expect(params['rollout'], '0.01');
    }, skip: skipRuby);

    test('production 99% là biên cao nhất được nhận', () {
      final out = run({
        'from': 'open',
        'to': 'production',
        'rollout_percent': '99',
      });
      expect((out['params']! as Map)['rollout'], '0.99');
    }, skip: skipRuby);

    test('lên production mà thiếu hoặc sai rollout_percent thì từ chối', () {
      for (final bad in ['', '0', '100', '-5', '1.5', 'abc', '007']) {
        final out = run({
          'from': 'closed',
          'to': 'production',
          'rollout_percent': bad,
        });
        expect(out['ok'], isFalse, reason: 'rollout_percent="$bad"');
      }
    }, skip: skipRuby);

    test('rollout_percent bị cấm khi đích không phải production', () {
      final out = run({
        'from': 'internal',
        'to': 'closed',
        'rollout_percent': '10',
      });
      expect(out['ok'], isFalse);
    }, skip: skipRuby);

    test('đường đi ngoài bảng bị từ chối, trước khi đụng tới supply', () {
      for (final (from, to) in [
        ('internal', 'production'),
        ('production', 'closed'),
        ('closed', 'closed'),
        ('closed', 'internal'),
        ('open', 'closed'),
        ('', ''),
      ]) {
        final out = run({'from': from, 'to': to, 'rollout_percent': '1'});
        expect(out['ok'], isFalse, reason: '$from -> $to');
        expect(out.containsKey('params'), isFalse);
      }
    }, skip: skipRuby);

    test('thiếu PLAY_JSON_KEY_DATA thì dừng', () {
      final out = run({'from': 'internal', 'to': 'closed'}, withKey: false);
      expect(out['ok'], isFalse);
      expect(out['error'], contains('PLAY_JSON_KEY_DATA'));
    }, skip: skipRuby);
  });
}
