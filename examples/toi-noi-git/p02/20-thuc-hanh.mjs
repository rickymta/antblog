// GIT-020 · Thực hành: phụ phí giờ cao điểm từ nhánh tới bản phát hành

import { join } from 'node:path';

import { FEATURE, CALC_PEAK, build2 } from '../lib/history2.mjs';
import { WORK } from '../lib/lab.mjs';
import { report } from './kiem-tra.mjs';
import { checkRefs } from './chung.mjs';

export default {
  id: 'GIT-020',
  start: null,
  end: 12,
  run(s, { refs2 }) {
    s.cd('.', { show: false });
    s.say('Repository mẫu: lịch sử chuẩn của Phần 2');
    build2(join(WORK, 'bao-gia'));
    s.cd('bao-gia');
    s.git('log --oneline --graph --decorate --all');
    checkRefs(s, refs2(12));
    s.print('$ node ../../p02/kiem-tra.mjs .');
    const good = report(s.cwd, (l) => s.print(l));
    s.check('repository mẫu đạt mọi tiêu chí', good.okAll);
    s.blank();

    s.say('Làm hỏng có chủ đích: nhánh đã merge còn sót, stash bị quên, merge dở dang, tag lightweight');
    s.git(`branch ${FEATURE} HEAD~3`);
    s.write('README.md', s.read('README.md') + '\nGhi chú tạm.\n');
    s.git('stash push -m "Ghi chú tạm"', { as: 'minh', at: '2026-11-19T10:00:00+07:00' });
    s.git('tag -d v1.0.1');
    s.git('tag v1.0.1 phat-hanh/1.0~1');
    s.git('switch -q -c thu-xung-dot v1.0.0');
    s.write('src/BaoGia/FareCalculator.cs', CALC_PEAK.replace('peakMultiplier = 1m', 'peakMultiplier = 1.0m'));
    s.git('commit -qam "Thử đổi giá trị mặc định"', { as: 'minh', at: '2026-11-19T10:05:00+07:00' });
    s.git('switch -q main');
    s.git('merge thu-xung-dot', { as: 'minh', at: '2026-11-19T10:10:00+07:00', expectFail: true, quiet: true });
    s.print('$ git merge thu-xung-dot   # xung đột, bỏ dở');
    s.print('$ node ../../p02/kiem-tra.mjs .');
    const bad = report(s.cwd, (l) => s.print(l));
    const failed = bad.res.filter((r) => !r.ok).map((r) => r.label);
    s.check('kiểm tra bắt được 6 lỗi', failed.length === 6, failed.join(' | '));
    s.blank();

    s.say('Sửa từng lỗi');
    s.git('merge --abort');
    s.git('branch -D thu-xung-dot');
    s.git(`branch -d ${FEATURE}`);
    s.git('stash drop', { quiet: true });
    s.print('$ git stash drop');
    s.git('tag -d v1.0.1');
    s.git('tag -a v1.0.1 -m "Phiên bản 1.0.1: bản vá làm tròn quãng đường" phat-hanh/1.0~1', {
      as: 'minh',
      at: '2026-11-18T09:10:00+07:00',
    });
    s.print('$ node ../../p02/kiem-tra.mjs .');
    const after = report(s.cwd, (l) => s.print(l));
    s.check('sửa xong đạt mọi tiêu chí', after.okAll);
    checkRefs(s, refs2(12), 'tag v1.0.1 tạo lại trùng mã hash chuẩn (cùng người tạo, thời điểm, lời nhắn)');
  },
};
