// GIT-029 · Pull làm rối lịch sử: chẩn đoán và sửa, cấu hình pull.rebase / pull.ff

import { PRICE_DOC_PULL_MINH, README_PULL_LAN } from '../lib/history3.mjs';
import { checkRefs3 } from './chung.mjs';

const MINH = { as: 'minh', at: '2026-11-23T09:10:00+07:00' };

export default {
  id: 'GIT-029',
  start: 6,
  end: 7,
  run(s, { refs3 }) {
    s.say('Lan push lên main của repository chung');
    s.cd('../lan/bao-gia', { show: false });
    s.write('README.md', README_PULL_LAN(s.read('README.md')));
    s.git('commit -q -am "Nhắc chạy test trước khi push"', { as: 'lan', at: '2026-11-23T09:00:00+07:00' });
    s.git('push -q upstream main');
    s.blank();
    s.say('Minh có commit riêng chưa push, rồi pull khi chưa cấu hình gì');
    s.cd('../../bao-gia', { show: false });
    s.write('docs/bang-gia-cuoc.md', PRICE_DOC_PULL_MINH(s.read('docs/bang-gia-cuoc.md')));
    s.git('commit -q -am "Ghi múi giờ áp dụng bảng giá"', MINH);
    s.git('pull', { ...MINH, expectFail: true });
    s.blank();
    s.say('Chọn merge cho nhanh: lịch sử có thêm một merge commit vô nghĩa');
    s.git('pull --no-rebase', MINH);
    s.git('log --graph --format=%s -4');
    s.git('log -1 --format=%s');
    s.blank();
    s.say('Chưa push: bỏ merge commit vừa tạo (ORIG_HEAD = main trước khi pull)');
    s.git('reset --hard ORIG_HEAD');
    s.git('status -sb');
    s.blank();
    s.say('Cấu hình pull mặc định là rebase cho repository này, rồi pull lại');
    s.git('config pull.rebase true');
    s.git('pull', { ...MINH, committedAt: '2026-11-23T09:20:00+07:00' });
    s.git('log --oneline --graph -3');
    s.git('push');
    s.cd('../lan/bao-gia', { show: false });
    s.git('pull -q --ff-only upstream main');
    s.cd('../../bao-gia', { show: false });
    s.check('main của Minh không có merge commit "Merge branch \'main\' of"', !s.value('log --format=%s main').includes("Merge branch 'main' of"));
    checkRefs3(s, refs3(7));
  },
};
