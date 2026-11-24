// GIT-045 · Git LFS cho file lớn

import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { pngThietKe } from '../lib/history5.mjs';
import { checkRefs5, P } from './chung.mjs';

export default {
  id: 'GIT-045',
  start: 4,
  end: 5,
  run(s, { refs5 }) {
    s.say('Bật Git LFS cho repository: cài filter và hook vào .git');
    s.git('lfs install --local');
    s.say('Theo dõi ảnh thiết kế bằng LFS: mẫu được ghi vào .gitattributes');
    s.git('lfs track "docs/thiet-ke/*.png"');
    s.print('$ Get-Content .gitattributes -Tail 1');
    s.print(s.read('.gitattributes').trimEnd().split('\n').pop());
    mkdirSync(join(P.minh, 'docs', 'thiet-ke'), { recursive: true });
    const png = pngThietKe();
    writeFileSync(join(P.minh, 'docs', 'thiet-ke', 'bang-gia-2027.png'), png);
    s.say(`Bản thiết kế bảng giá 2027: ${png.length} byte`);
    s.git('add .gitattributes docs/thiet-ke');
    s.git('lfs status');
    s.git('commit -q -m "Thêm bản thiết kế bảng giá 2027 (Git LFS)"', { as: 'minh', at: '2026-11-27T14:00:00+07:00' });
    s.say('Trong commit chỉ là một file con trỏ vài dòng; nội dung thật nằm trong .git/lfs');
    s.git('show HEAD:docs/thiet-ke/bang-gia-2027.png');
    s.git('lfs ls-files -l');
    s.git('cat-file -s HEAD:docs/thiet-ke/bang-gia-2027.png');
    s.git('push -q');
    s.check('repository có file lưu bằng LFS', s.value('lfs ls-files -n') === 'docs/thiet-ke/bang-gia-2027.png');
    checkRefs5(s, refs5(5));
  },
};
