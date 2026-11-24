// GIT-050 · Thực hành: điều tra lỗi cước và bảo trì repository

import { rmSync } from 'node:fs';
import { join } from 'node:path';

import { REPORT } from '../lib/history5.mjs';
import { checkRefs5, P } from './chung.mjs';
import { report } from './kiem-tra.mjs';

export default {
  id: 'GIT-050',
  start: 7,
  end: 8,
  run(s, { refs5 }) {
    s.say('Báo cáo điều tra: commit qua hook của nhóm (pre-commit, commit-msg), push qua pre-push');
    s.write('docs/bao-cao-sai-cuoc.md', REPORT);
    s.git('add docs/bao-cao-sai-cuoc.md');
    s.git('commit -q -m "Thêm báo cáo điều tra sai cước"', { as: 'minh', at: '2026-11-28T11:00:00+07:00' });
    s.git('push -q');
    s.git('commit-graph write --reachable --changed-paths');
    s.git('log --oneline -10');
    checkRefs5(s, refs5(8));
    s.print('$ node ../../p05/kiem-tra.mjs .');
    const good = report(s.cwd, (l) => s.print(l));
    s.check('repository đạt mọi tiêu chí', good.okAll);
    s.blank();
    s.say('Làm hỏng có chủ đích: bỏ core.hooksPath, huỷ khởi tạo submodule, xoá commit-graph');
    s.git('config --unset core.hooksPath');
    s.git('submodule deinit -f lib/tien-te');
    rmSync(join(P.minh, '.git', 'objects', 'info', 'commit-graph'), { force: true });
    rmSync(join(P.minh, '.git', 'objects', 'info', 'commit-graphs'), { recursive: true, force: true });
    s.print('$ Remove-Item .git/objects/info/commit-graph*');
    s.print('$ node ../../p05/kiem-tra.mjs .');
    const bad = report(s.cwd, (l) => s.print(l));
    const failed = bad.res.filter((r) => !r.ok).map((r) => r.label);
    s.check('kiểm tra bắt được 3 lỗi', failed.length === 3, failed.join(' | '));
    s.blank();
    s.say('Sửa từng lỗi');
    s.git('config core.hooksPath .githooks');
    s.git('-c protocol.file.allow=always submodule update --init');
    s.git('commit-graph write --reachable --changed-paths');
    s.print('$ node ../../p05/kiem-tra.mjs .');
    const after = report(s.cwd, (l) => s.print(l));
    s.check('sửa xong đạt mọi tiêu chí', after.okAll);
  },
};
