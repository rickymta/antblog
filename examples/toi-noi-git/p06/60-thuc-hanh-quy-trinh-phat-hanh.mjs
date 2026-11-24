// GIT-060 · Thực hành: quy trình phát hành của nhóm Tới Nơi

import { rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { RUNBOOK, addChangelogEntry } from '../lib/history6.mjs';
import { checkRefs6, P, sh } from './chung.mjs';
import { report } from './kiem-tra.mjs';

const M = (t) => ({ as: 'minh', at: `2026-12-07T${t}:00+07:00` });

export default {
  id: 'GIT-060',
  start: 9,
  end: 10,
  run(s, { refs6 }) {
    s.say('Runbook phát hành vào repository');
    s.write('docs/quy-trinh-phat-hanh.md', RUNBOOK);
    s.git('add docs/quy-trinh-phat-hanh.md');
    s.git('commit -q -m "docs: runbook quy trình phát hành"', M('09:00'));
    s.blank();
    s.say('1. Chuẩn bị: main trùng origin/main, bản trước và các commit kể từ đó');
    s.git('push -q');
    s.git('status -sb');
    s.git('describe --tags --abbrev=0');
    s.git('log --oneline --no-merges v1.3.1..main');
    s.say('Có feat → tăng MINOR: 1.4.0');
    s.blank();
    s.say('2. CHANGELOG');
    const entry = sh(s, ['scripts/changelog.sh', '1.4.0', '2026-12-07', 'v1.3.1']).out;
    s.write('CHANGELOG.md', addChangelogEntry(s.read('CHANGELOG.md'), entry));
    s.git('commit -q -am "chore(release): 1.4.0"', M('09:30'));
    s.blank();
    s.say('3. Tag có chữ ký, nhánh phát hành, push một lần');
    s.git('tag -s v1.4.0 -m "Phát hành 1.4.0: ước lượng quãng đường bằng bản đồ"', M('09:45'));
    s.git('tag -v v1.4.0');
    s.git('branch phat-hanh/1.4 v1.4.0');
    s.git('push -q origin main phat-hanh/1.4 v1.4.0');
    sh(s, ['scripts/phien-ban.sh']);
    checkRefs6(s, refs6(10));
    s.blank();
    s.say('4. Chấm theo 10 tiêu chí');
    s.print('$ node ../../p06/kiem-tra.mjs .');
    const good = report(s.cwd, (l) => s.print(l));
    s.check('repository đạt mọi tiêu chí', good.okAll);
    s.blank();
    s.say('Tag phát hành đã push thì không xoá được: hook pre-receive của remote (GIT-027) chặn');
    s.git('push -q origin --delete v1.4.0', { expectFail: true });
    s.blank();
    s.say('Làm hỏng có chủ đích: xoá nhánh phát hành trên remote, tắt hook chung, để sót index.lock');
    s.git('push -q origin --delete phat-hanh/1.4');
    s.git('config --unset core.hooksPath');
    writeFileSync(join(P.minh, '.git', 'index.lock'), '');
    s.print('$ New-Item .git/index.lock');
    s.print('$ node ../../p06/kiem-tra.mjs .');
    const bad = report(s.cwd, (l) => s.print(l));
    const failed = bad.res.filter((r) => !r.ok).map((r) => r.label);
    s.check('kiểm tra bắt được 3 lỗi', failed.length === 3, failed.join(' | '));
    s.blank();
    s.say('Sửa: xoá index.lock (sau khi chắc không còn tiến trình git), bật lại hook, push lại nhánh');
    rmSync(join(P.minh, '.git', 'index.lock'));
    s.print('$ Remove-Item .git/index.lock');
    s.git('config core.hooksPath .githooks');
    s.git('push -q origin phat-hanh/1.4');
    s.print('$ node ../../p06/kiem-tra.mjs .');
    const after = report(s.cwd, (l) => s.print(l));
    s.check('sửa xong đạt mọi tiêu chí', after.okAll);
  },
};
