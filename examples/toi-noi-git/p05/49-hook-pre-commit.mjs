// GIT-049 · Hook: chặn commit sai trước khi rời máy

import { HOOK_COMMIT_MSG, HOOK_PRE_COMMIT, HOOK_PRE_PUSH, NIGHT_DEBUG_LINE } from '../lib/history5.mjs';
import { checkRefs5 } from './chung.mjs';

const MINH = (t) => ({ as: 'minh', at: `2026-11-28T${t}:00+07:00` });

export default {
  id: 'GIT-049',
  start: 6,
  end: 7,
  run(s, { refs5 }) {
    s.say('Hook mặc định nằm trong .git/hooks: không commit được, mỗi người tự cài. LFS đã đặt vài hook ở đó:');
    s.ls('.git/hooks');
    s.blank();
    s.say('Hook chung của nhóm đặt trong .githooks (commit cùng mã nguồn), mỗi máy trỏ core.hooksPath tới đó');
    s.write('.githooks/pre-commit', HOOK_PRE_COMMIT);
    s.write('.githooks/commit-msg', HOOK_COMMIT_MSG);
    s.write('.githooks/pre-push', HOOK_PRE_PUSH);
    s.print('$ Get-Content .githooks/pre-commit');
    s.print(HOOK_PRE_COMMIT.trimEnd());
    s.git('add .githooks');
    s.git('update-index --chmod=+x .githooks/pre-commit .githooks/commit-msg .githooks/pre-push');
    s.git('ls-files -s .githooks');
    s.git('commit -q -m "Thêm hook chung của nhóm"', MINH('10:00'));
    s.git('config core.hooksPath .githooks');
    s.blank();
    s.say('pre-commit chặn dòng debug trong phần thay đổi đã stage');
    s.write('src/BaoGia/NightSurcharge.cs', NIGHT_DEBUG_LINE(s.read('src/BaoGia/NightSurcharge.cs')));
    s.git('add src/BaoGia/NightSurcharge.cs');
    s.git('commit -m "Ghi log khi tính phụ phí đêm"', { ...MINH('10:10'), expectFail: true });
    s.git('restore --staged --worktree src/BaoGia/NightSurcharge.cs');
    s.blank();
    s.say('commit-msg chặn tiêu đề WIP và tiêu đề quá ngắn');
    s.git('commit --allow-empty -m "WIP phụ phí"', { ...MINH('10:20'), expectFail: true });
    s.git('commit --allow-empty -m "sửa"', { ...MINH('10:20'), expectFail: true });
    s.say('--no-verify bỏ qua pre-commit và commit-msg (chỉ dùng khi biết rõ vì sao); bỏ commit thử ngay sau đó');
    s.git('commit -q --allow-empty --no-verify -m "WIP"', MINH('10:30'));
    s.git('log --oneline -1');
    s.git('reset -q --hard HEAD~1');
    s.blank();
    s.say('pre-push kiểm tra quy tắc làm tròn rồi chuyển cho Git LFS; push bình thường đi qua');
    s.git('push');
    checkRefs5(s, refs5(7));
  },
};
