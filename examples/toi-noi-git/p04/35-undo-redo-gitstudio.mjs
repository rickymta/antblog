// GIT-035 · Undo/Redo của GitStudio và giới hạn của nó
// GitStudio không chạy được trong lab; bài cho thấy đúng các lệnh git mà Hoàn tác/Làm lại chạy.

import { checkRefs4 } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-11-24T${t}:00+07:00` });

export default {
  id: 'GIT-035',
  start: 1,
  end: 1,
  run(s, { refs4 }) {
    s.say('Commit (hoặc amend): Hoàn tác = reset --soft về commit trước; thay đổi còn nguyên, đã stage');
    s.write('README.md', s.read('README.md') + '\nGhi chú thử.\n');
    s.git('commit -q -am "Ghi chú thử"', M('12:00'));
    s.git('reset --soft HEAD~1');
    s.git('status -s');
    s.git('restore --staged README.md');
    s.git('restore README.md');
    s.blank();
    s.say('Merge, pull, revert, cherry-pick, sửa/bỏ commit, rebase: Hoàn tác = reset --keep về đỉnh nhánh cũ');
    s.git('revert --no-edit HEAD', M('12:10'));
    s.git('log --oneline -1');
    s.say('--keep từ chối nếu file sẽ bị đổi đang có thay đổi chưa commit (an toàn hơn --hard)');
    s.write('src/BaoGia/WaitingFee.cs', s.read('src/BaoGia/WaitingFee.cs').replace('    private const int FreeMinutes = 5;', '    private const int FreeMinutes = 6;'));
    s.git('reset --keep HEAD~1', { expectFail: true });
    s.git('restore src/BaoGia/WaitingFee.cs');
    s.git('reset --keep HEAD~1');
    s.git('status -sb');
    s.blank();
    s.say('Huỷ bỏ thay đổi trong GitStudio chạy checkout -- (và clean -f cho file mới): không có bản sao nào để lấy lại');
    s.write('README.md', s.read('README.md') + '\nĐoạn sắp bị huỷ.\n');
    s.git('checkout -- README.md');
    s.git('status -s');
    s.git('stash list');
    checkRefs4(s, refs4(1));
  },
};
