// GIT-010 · Thực hành: dựng repository dịch vụ báo giá có lịch sử sạch

import { join } from 'node:path';

import { GITATTRIBUTES, GITIGNORE, README, build } from '../lib/history.mjs';
import { WORK } from '../lib/lab.mjs';
import { report } from './kiem-tra.mjs';

export default {
  id: 'GIT-010',
  start: null,
  end: 10,
  run(s, { hashes }) {
    s.cd('.', { show: false });
    s.say('Repository mẫu: lịch sử chuẩn của Phần 1');
    build(join(WORK, 'bao-gia'));
    s.cd('bao-gia');
    s.git('log --format="%h %<(9)%an %s"');
    s.check('HEAD trùng c10', s.value('rev-parse HEAD') === hashes.get('c10'));
    s.blank();
    s.print('$ node ../../p01/kiem-tra.mjs .');
    const good = report(s.cwd, (l) => s.print(l));
    s.check('repository mẫu đạt mọi tiêu chí', good.okAll);
    s.blank();

    s.say('Repository mắc lỗi thường gặp: commit cả bin/ và .env, summary chung chung, CRLF, quên commit');
    s.cd('..');
    s.git('init -q -b main bao-gia-loi');
    s.cd('bao-gia-loi');
    s.write('README.md', README.replace(/\n/g, '\r\n'));
    s.write('src/BaoGia/bin/Debug/net10.0/BaoGia.dll', 'MZ\n');
    s.write('.env', 'MAPS_API_KEY=abc\n');
    s.git('add -A');
    s.git('commit -q -m "update"', { as: 'minh', at: '2026-11-02T08:30:00+07:00' });
    s.write('src/BaoGia/FareCalculator.cs', '// tính cước\n');
    s.git('add -A');
    s.git('commit -q -m "Thêm hàm tính cước theo km cho ô tô bốn chỗ và xe máy, có giá tối thiểu và làm tròn"', {
      as: 'minh',
      at: '2026-11-02T09:00:00+07:00',
    });
    s.write('ghi-chu.txt', 'việc dở\n');
    s.print('$ node ../../p01/kiem-tra.mjs .');
    const bad = report(s.cwd, (l) => s.print(l));
    const failed = bad.res.filter((r) => !r.ok).map((r) => r.label);
    s.check('kiểm tra bắt được 8 lỗi của repository lỗi', failed.length === 8, failed.join(' | '));
    s.blank();

    s.say('Sửa: ngừng theo dõi file build và bí mật, thêm .gitignore, .gitattributes, chuẩn hoá xuống dòng');
    s.write('.gitignore', GITIGNORE);
    s.write('.gitattributes', GITATTRIBUTES);
    s.remove('ghi-chu.txt');
    s.git('rm -r -q --cached src/BaoGia/bin .env');
    s.git('add .gitignore .gitattributes');
    s.git('add --renormalize .');
    s.git('status --short');
    s.git('commit -q -m "Ngừng theo dõi kết quả build và .env, chuẩn hoá LF"', {
      as: 'minh',
      at: '2026-11-02T09:30:00+07:00',
    });
    s.print('$ node ../../p01/kiem-tra.mjs .');
    const after = report(s.cwd, (l) => s.print(l));
    const left = after.res.filter((r) => !r.ok).map((r) => r.label);
    s.say('Hai lỗi còn lại nằm trong message của commit cũ; Phần 4 học cách viết lại lịch sử.');
    s.check(
      'chỉ còn 2 lỗi về summary trong lịch sử cũ',
      left.length === 2 && left.every((l) => l.includes('summary')),
      left.join(' | '),
    );
  },
};
