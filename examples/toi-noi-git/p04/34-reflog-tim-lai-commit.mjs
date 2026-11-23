// GIT-034 · Reflog: tìm lại commit "đã mất"

import { checkRefs4 } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-11-24T${t}:00+07:00` });

export default {
  id: 'GIT-034',
  start: 1,
  end: 1,
  run(s, { refs4 }) {
    s.say('Lỡ tay: reset --hard lùi main hai commit');
    s.git('reset --hard HEAD~2');
    s.git('log --oneline -1');
    s.git('reflog -3');
    s.say('Mục HEAD@{1} là chỗ HEAD đứng ngay trước lệnh reset');
    s.git('reset --hard HEAD@{1}');
    s.git('status -sb');
    s.blank();
    s.say('Xoá nhầm một nhánh chưa merge');
    s.git('switch -q -c thu/ghi-chu');
    s.write('docs/ghi-chu.md', 'Ý tưởng: giảm phí chờ cho khách thân thiết.\n');
    s.git('add docs/ghi-chu.md');
    s.git('commit -q -m "Ghi ý tưởng giảm phí chờ"', M('11:00'));
    s.git('switch -q main');
    s.git('branch -D thu/ghi-chu');
    s.git('reflog -2');
    s.git('branch thu/ghi-chu HEAD@{1}');
    s.git('log --oneline -1 thu/ghi-chu');
    s.git('branch -q -D thu/ghi-chu');
    s.blank();
    s.say('Rebase xong mới thấy sai: reflog của chính nhánh giữ vị trí trước rebase');
    s.git('switch -q -c thu/rebase HEAD~2');
    s.write('docs/ghi-chu.md', 'Nháp\n');
    s.git('add docs/ghi-chu.md');
    s.git('commit -q -m "Nháp trên nhánh thu/rebase"', M('11:10'));
    s.git('rebase -q main', { committedAt: '2026-11-24T11:15:00+07:00' });
    s.git('reflog -3 thu/rebase');
    s.git('reset --hard thu/rebase@{1}');
    s.git('log --oneline -2');
    s.git('switch -q main');
    s.git('branch -q -D thu/rebase');
    checkRefs4(s, refs4(1));
  },
};
