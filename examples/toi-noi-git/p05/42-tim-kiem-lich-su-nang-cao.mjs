// GIT-042 · Tìm kiếm nâng cao trong lịch sử

import { checkRefs5 } from './chung.mjs';

export default {
  id: 'GIT-042',
  start: 2,
  end: 2,
  run(s, { refs5 }) {
    s.say('-S (pickaxe): commit làm THAY ĐỔI SỐ LẦN xuất hiện của chuỗi');
    s.git('log --oneline -S"Math.Round(atLeastMinimum"');
    s.say('-G: commit có dòng thêm/bớt khớp biểu thức chính quy (kể cả khi số lần không đổi)');
    s.git('log --oneline -G"atLeastMinimum / 1000m"');
    s.blank();
    s.say('Theo thông điệp, người viết, thời gian; kết hợp nhiều điều kiện');
    s.git('log --oneline -i --grep="phụ phí đêm"');
    s.git('log --oneline --author="Lê Lan" --since="2026-11-26T00:00+07:00"');
    s.git('log --oneline --grep="đêm" --grep="test" --all-match');
    s.blank();
    s.say('Tìm trong nội dung file ở một commit bất kỳ, không cần checkout');
    s.git('grep -n "Rate" v1.1.0 -- src', { expectFail: true });
    s.git('grep -n "Rate" main -- src');
    s.blank();
    s.say('Khoảng commit: hai chấm và ba chấm');
    s.git('rev-list --count phat-hanh/1.0..main');
    s.git('rev-list --count main..phat-hanh/1.0');
    s.git('log --oneline main..phat-hanh/1.0');
    s.git('rev-list --left-right --count main...phat-hanh/1.0');
    s.git('merge-base --all main phat-hanh/1.0');
    s.git('log --oneline -1 v1.0.0');
    s.blank();
    s.say('Chỉ đi theo cha thứ nhất: lịch sử của main như một chuỗi merge và commit trực tiếp');
    s.git('log --oneline --first-parent -6');
    s.git('log --oneline --merges -3');
    checkRefs5(s, refs5(2));
  },
};
