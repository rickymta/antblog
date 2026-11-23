// GIT-030 · Thực hành: hai developer, một remote

import { join } from 'node:path';

import { build3 } from '../lib/history3.mjs';
import { WORK } from '../lib/lab.mjs';
import { report } from './kiem-tra.mjs';
import { checkRefs3 } from './chung.mjs';

export default {
  id: 'GIT-030',
  start: null,
  end: 7,
  run(s, { refs3 }) {
    s.cd('.', { show: false });
    s.say('Repository mẫu: lịch sử chuẩn của Phần 3 (Minh, Lan, remote chung)');
    build3(join(WORK, 'bao-gia'));
    s.cd('bao-gia');
    s.git('log --oneline --graph -10');
    checkRefs3(s, refs3(7));
    s.print('$ node ../../p03/kiem-tra.mjs .');
    const good = report(s.cwd, (l) => s.print(l));
    s.check('repository của Minh đạt mọi tiêu chí', good.okAll);
    s.blank();

    s.say('Làm hỏng có chủ đích: commit chưa push, nhánh đã merge còn trên remote, nhánh theo dõi nhánh đã xoá, bỏ cấu hình pull');
    s.write('README.md', s.read('README.md') + '\nGhi chú chưa push.\n');
    s.git('commit -q -am "Ghi chú chưa push"', { as: 'minh', at: '2026-11-24T09:00:00+07:00' });
    s.git('push -q origin main~1:refs/heads/sua/da-merge');
    s.git('switch -q -c sua/theo-doi-nhanh-da-xoa');
    s.git('push -q -u origin sua/theo-doi-nhanh-da-xoa');
    s.git('push -q origin --delete sua/theo-doi-nhanh-da-xoa');
    s.git('switch -q main');
    s.git('config --unset pull.rebase');
    s.print('$ node ../../p03/kiem-tra.mjs .');
    const bad = report(s.cwd, (l) => s.print(l));
    const failed = bad.res.filter((r) => !r.ok).map((r) => r.label);
    s.check('kiểm tra bắt được 4 lỗi', failed.length === 4, failed.join(' | '));
    s.blank();

    s.say('Sửa từng lỗi');
    s.git('config pull.rebase true');
    s.git('push -q');
    s.git('push origin --delete sua/da-merge');
    s.git('fetch --prune');
    s.git('branch -D sua/theo-doi-nhanh-da-xoa');
    s.print('$ node ../../p03/kiem-tra.mjs .');
    const after = report(s.cwd, (l) => s.print(l));
    s.check('sửa xong đạt mọi tiêu chí', after.okAll);
  },
};
