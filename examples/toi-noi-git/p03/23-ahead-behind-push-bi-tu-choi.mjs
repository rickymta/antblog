// GIT-023 · Ahead/behind và push bị từ chối

import { PRICE_DOC_CONTACT, README_REMOTE } from '../lib/history3.mjs';
import { checkRefs3 } from './chung.mjs';

const LAN = (t) => ({ as: 'lan', at: `2026-11-20T${t}:00+07:00` });
const MINH = (t) => ({ as: 'minh', at: `2026-11-20T${t}:00+07:00` });

export default {
  id: 'GIT-023',
  start: 2,
  end: 3,
  run(s, { refs3 }) {
    s.say('Lan push trước');
    s.cd('../lan/bao-gia', { show: false });
    s.write('docs/bang-gia-cuoc.md', PRICE_DOC_CONTACT(s.read('docs/bang-gia-cuoc.md')));
    s.git('commit -q -am "Ghi đầu mối đổi bảng giá vào tài liệu"', LAN('10:00'));
    s.git('push -q');
    s.blank();
    s.say('Minh commit trên main cũ rồi push: bị từ chối');
    s.cd('../../bao-gia', { show: false });
    s.write('README.md', README_REMOTE(s.read('README.md')));
    s.git('commit -am "Ghi quy ước dùng remote vào README"', MINH('10:05'));
    s.git('status -sb');
    s.git('push', { expectFail: true });
    s.blank();
    s.say('Fetch rồi đọc ahead/behind');
    s.git('fetch');
    s.git('status -sb');
    s.git('rev-list --left-right --count main...origin/main');
    s.git('log --oneline --graph main origin/main -3');
    s.blank();
    s.say('Đặt commit của mình lên sau commit của Lan, rồi push');
    s.git('pull --rebase', { ...MINH('10:05'), committedAt: '2026-11-20T10:10:00+07:00' });
    s.git('push');
    s.git('status -sb');
    s.blank();
    s.say('Force push an toàn trên nhánh của riêng mình: --force-with-lease');
    s.git('switch -c thu/force-with-lease');
    s.write('ghi-chu-review.md', '- [ ] xem lại phí chờ\n');
    s.git('add ghi-chu-review.md');
    s.git('commit -q -m "Ghi chú review"', MINH('10:20'));
    s.git('push -q -u origin thu/force-with-lease');
    s.write('ghi-chu-review.md', '- [ ] xem lại phí chờ\n- [ ] xem lại giờ cao điểm\n');
    s.git('commit -q -a --amend --no-edit', MINH('10:25'));
    s.git('push', { expectFail: true });
    s.git('push --force-with-lease');
    s.blank();
    s.say('Lan cũng push lên nhánh đó; Minh chưa fetch mà ép push: lease chặn lại');
    s.cd('../lan/bao-gia', { show: false });
    s.git('fetch -q');
    s.git('switch -q thu/force-with-lease');
    s.write('ghi-chu-review.md', s.read('ghi-chu-review.md') + '- [ ] Lan: xem test\n');
    s.git('commit -q -am "Thêm việc của Lan"', LAN('10:30'));
    s.git('push -q');
    s.cd('../../bao-gia', { show: false });
    s.git('commit -q --allow-empty -m "Thử ép push"', MINH('10:35'));
    s.git('push --force-with-lease', { expectFail: true });
    s.blank();
    s.say('Dọn nhánh thử ở cả ba nơi');
    s.git('switch -q main');
    s.git('push -q origin --delete thu/force-with-lease');
    s.git('branch -q -D thu/force-with-lease');
    s.cd('../lan/bao-gia', { show: false });
    s.git('switch -q main');
    s.git('branch -q -D thu/force-with-lease');
    s.git('pull -q --ff-only --prune');
    s.cd('../../bao-gia', { show: false });
    checkRefs3(s, refs3(3));
  },
};
