// GIT-059 · Sự cố Git thường gặp và cách xử lý

import { rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { MAPS_DEFAULT_ON } from '../lib/history6.mjs';
import { checkRefs6, P } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-12-04T${t}:00+07:00` });

export default {
  id: 'GIT-059',
  start: 8,
  end: 9,
  run(s, { refs6 }) {
    s.say('Sự cố 1: index.lock còn sót (git bị tắt ngang, hai chương trình cùng chạy git)');
    writeFileSync(join(P.minh, '.git', 'index.lock'), '');
    s.write('docs/ghi-chu.md', 'Ghi chú tạm.\n');
    s.git('add docs/ghi-chu.md', { expectFail: true });
    s.say('Chắc chắn không còn tiến trình git nào (đóng GitStudio, IDE, terminal khác), rồi mới xoá file khoá');
    rmSync(join(P.minh, '.git', 'index.lock'));
    s.print('$ Remove-Item .git/index.lock');
    s.git('add docs/ghi-chu.md');
    s.git('restore --staged docs/ghi-chu.md');
    s.remove('docs/ghi-chu.md');
    s.blank();
    s.say('Sự cố 2: commit trong lúc detached HEAD rồi chuyển đi');
    s.git('switch --detach v1.3.1');
    s.git('commit -q --allow-empty -m "test(cuoc): thử lại lỗi làm tròn trên 1.3.1"', M('13:00'));
    const lost = s.value('rev-parse --short HEAD');
    s.git('switch main');
    s.say('Commit chưa mất: reflog còn nhớ (GIT-034). Tạo nhánh để giữ, hoặc bỏ đi nếu chỉ là thử');
    s.git('reflog -3 --format="%h %gs"');
    s.git(`branch thu/lam-tron-1.3.1 ${lost}`);
    s.git('log --oneline -1 thu/lam-tron-1.3.1');
    s.git('branch -D thu/lam-tron-1.3.1');
    s.blank();
    s.say('Sự cố 3: push nhầm lên main một thay đổi chưa được duyệt');
    s.write('src/BaoGia/appsettings.json', MAPS_DEFAULT_ON(s.read('src/BaoGia/appsettings.json')));
    s.git('commit -q -am "chore(ban-do): bật ước lượng quãng đường mặc định"', M('14:00'));
    s.git('push -q');
    s.say('main đã chia sẻ: không reset hay force push. Giữ thay đổi trên nhánh riêng, revert trên main');
    s.git('branch tinh-nang/ban-do-mac-dinh');
    s.git('revert --no-edit HEAD', M('14:20'));
    s.git('push -q');
    s.git('log --oneline -3');
    s.git('diff main~2 main --stat');
    s.check('main không còn bật mặc định, nhánh riêng còn giữ', !s.read('src/BaoGia/appsettings.json').includes('EstimateByDefault') && s.value('show tinh-nang/ban-do-mac-dinh:src/BaoGia/appsettings.json').includes('EstimateByDefault'));
    checkRefs6(s, refs6(9));
  },
};
