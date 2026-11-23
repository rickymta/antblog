// GIT-026 · Pull request và code review (mô phỏng trên remote cục bộ)

import { PAYMENT_BRANCH, WAITING_FEE_V1, WAITING_FEE_V2, WAITING_TESTS_V1, WAITING_TESTS_V2 } from '../lib/history3.mjs';
import { checkRefs3 } from './chung.mjs';

const LAN = (t) => ({ as: 'lan', at: `2026-11-21T${t}:00+07:00` });

export default {
  id: 'GIT-026',
  start: 3,
  end: 4,
  run(s, { refs3 }) {
    s.say('Lan: nhánh tính năng, commit, push với upstream (rồi mở pull request trên GitHub)');
    s.cd('../lan/bao-gia');
    s.git(`switch -c ${PAYMENT_BRANCH}`);
    s.write('src/BaoGia/WaitingFee.cs', WAITING_FEE_V1);
    s.write('tests/BaoGia.Tests/WaitingFeeTests.cs', WAITING_TESTS_V1);
    s.git('add src tests');
    s.git('commit -m "Tính phí chờ theo phút, tối đa 30.000 đ"', LAN('09:00'));
    s.git(`push -u origin ${PAYMENT_BRANCH}`);
    s.blank();
    s.say('Minh review: nhánh có gì mà main chưa có, đổi những file nào');
    s.cd('../../bao-gia');
    s.git('fetch');
    s.git(`log --oneline main..origin/${PAYMENT_BRANCH}`);
    s.git(`diff --stat main...origin/${PAYMENT_BRANCH}`);
    s.git(`diff main...origin/${PAYMENT_BRANCH} -- src`);
    s.say('Nhận xét review: docs/phi-cho.md ghi "chờ quá 5 phút" mới tính, code đang tính từ phút đầu.');
    s.blank();
    s.say('Lan sửa theo review, push thêm commit vào cùng nhánh (pull request tự cập nhật)');
    s.cd('../lan/bao-gia');
    s.write('src/BaoGia/WaitingFee.cs', WAITING_FEE_V2);
    s.write('tests/BaoGia.Tests/WaitingFeeTests.cs', WAITING_TESTS_V2);
    s.git('commit -am "Miễn phí 5 phút chờ đầu tiên theo tài liệu"', LAN('09:40'));
    s.git('push');
    s.blank();
    s.say('Minh xem phần mới kể từ lần review trước, rồi merge kiểu squash');
    s.cd('../../bao-gia');
    s.git('fetch');
    s.git(`log --oneline main..origin/${PAYMENT_BRANCH}`);
    s.git(`merge --squash origin/${PAYMENT_BRANCH}`);
    s.git(
      'commit --author="Lê Lan <lan@toinoi.test>" -m "Tính phí chờ tại điểm đón" -m "Pull request của Lê Lan (2 commit), Trần Minh review: miễn phí 5 phút đầu, tối đa 30.000 đ."',
      { as: 'lan', by: 'minh', at: '2026-11-21T10:00:00+07:00' },
    );
    s.git('log -1 --format=fuller --stat');
    s.git('push');
    s.git(`push origin --delete ${PAYMENT_BRANCH}`);
    s.blank();
    s.say('Lan dọn nhánh: -d từ chối vì squash tạo commit mới, các commit cũ không nằm trên main');
    s.cd('../lan/bao-gia');
    s.git('switch main');
    s.git('pull --ff-only --prune');
    s.git(`branch -d ${PAYMENT_BRANCH}`, { expectFail: true });
    s.git(`branch -D ${PAYMENT_BRANCH}`);
    s.cd('../../bao-gia', { show: false });
    checkRefs3(s, refs3(4));
  },
};
