// GIT-017 · Cherry-pick và revert một commit sang nhánh khác

import { RELEASE, TAG_MSG } from '../lib/history2.mjs';
import { checkRefs } from './chung.mjs';

export default {
  id: 'GIT-017',
  start: 8,
  end: 10,
  run(s, { refs2 }) {
    const m2 = s.value('log --format=%h -1 --grep=0,1.km main');
    const p1 = s.value('log --format=%h -1 --grep=hệ.số.phụ.phí main');
    s.say(`Bản vá làm tròn quãng đường: ${m2}; commit cấu hình giờ cao điểm: ${p1}`);
    s.blank();
    s.say('Nhánh phát hành 1.0 tạo từ tag v1.0.0, mang bản vá sang bằng cherry-pick');
    s.git(`switch -c ${RELEASE} v1.0.0`);
    s.git(`cherry-pick -x ${m2}`, { as: 'minh', at: '2026-11-18T09:00:00+07:00' });
    s.git('log -1 --format="%h %s%n%n%b"');
    s.git('log -1 --format="tác giả: %an %ad%nngười commit: %cn %cd" --date=iso');
    s.git(`tag -a v1.0.1 -m "${TAG_MSG['v1.0.1']}"`, { as: 'minh', at: '2026-11-18T09:10:00+07:00' });
    s.git('log --oneline --graph --decorate -3');
    s.blank();
    s.say('Bộ phận vận hành hoãn phụ phí giờ cao điểm: revert commit cấu hình trên main');
    s.git('switch main');
    s.git(`revert --no-edit ${p1}`, { as: 'lan', at: '2026-11-18T14:00:00+07:00' });
    s.git('log -1 --format="%h %s%n%n%b"');
    s.git('show --stat --format= HEAD');
    s.git('grep -c Multiplier -- src/BaoGia/appsettings.json', { expectFail: true });
    s.check('code vẫn nhận hệ số (mặc định 1)', s.read('src/BaoGia/FareCalculator.cs').includes('peakMultiplier = 1m'));
    checkRefs(s, refs2(10));
  },
};
