// GIT-040 · Thực hành: dọn nhánh tính năng và runbook hoàn tác

import { NIGHT_BRANCH, RUNBOOK } from '../lib/history4.mjs';
import { checkRefs4 } from './chung.mjs';
import { report } from './kiem-tra.mjs';

const M = (t) => ({ as: 'minh', at: `2026-11-25T${t}:00+07:00` });

export default {
  id: 'GIT-040',
  start: 6,
  end: 7,
  run(s, { refs4 }) {
    s.say('Merge nhánh phụ phí đêm đã dọn vào main: rerere tự giải quyết xung đột đã gặp ở GIT-039');
    s.git(`log --oneline main..${NIGHT_BRANCH}`);
    s.git(`merge --no-ff ${NIGHT_BRANCH}`, { ...M('14:00'), expectFail: true });
    s.git('diff');
    s.git('add docs/phi-cho.md');
    s.git('merge --continue', M('14:00'));
    s.git('push -q');
    s.git(`branch -d ${NIGHT_BRANCH}`);
    s.blank();
    s.say('Runbook hoàn tác của nhóm, commit và push');
    s.write('docs/runbook-hoan-tac.md', RUNBOOK);
    s.git('add docs/runbook-hoan-tac.md');
    s.git('commit -q -m "Thêm runbook hoàn tác"', M('15:00'));
    s.git('push -q');
    s.git('log --oneline --graph -7');
    checkRefs4(s, refs4(7));
    s.print('$ node ../../p04/kiem-tra.mjs .');
    const good = report(s.cwd, (l) => s.print(l));
    s.check('repository đạt mọi tiêu chí', good.okAll);
    s.blank();
    s.say('Làm hỏng có chủ đích: commit WIP chưa push, tắt rerere, nhánh nháp còn sót');
    s.git('commit -q --allow-empty -m "WIP thử nghiệm"', M('16:00'));
    s.git('config rerere.enabled false');
    s.git('branch thu/nhap');
    s.print('$ node ../../p04/kiem-tra.mjs .');
    const bad = report(s.cwd, (l) => s.print(l));
    s.check('kiểm tra bắt được 4 lỗi', bad.res.filter((r) => !r.ok).length === 4, bad.res.filter((r) => !r.ok).map((r) => r.label).join(' | '));
    s.blank();
    s.say('Sửa: bỏ commit chưa push (reset về origin/main), bật lại rerere, xoá nhánh nháp');
    s.git('reset --hard origin/main');
    s.git('config rerere.enabled true');
    s.git('branch -D thu/nhap');
    s.print('$ node ../../p04/kiem-tra.mjs .');
    const after = report(s.cwd, (l) => s.print(l));
    s.check('sửa xong đạt mọi tiêu chí', after.okAll);
  },
};
