// GIT-044 · Submodule: dùng chung thư viện tiền tệ

import { LIB_V1, LIB_V2 } from '../lib/history5.mjs';
import { freshDir } from '../lib/lab.mjs';
import { checkRefs5, LIB_SHOW, P } from './chung.mjs';

const LAN = (t) => ({ as: 'lan', at: `2026-11-27T${t}:00+07:00` });
const MINH = (t) => ({ as: 'minh', at: `2026-11-27T${t}:00+07:00` });

export default {
  id: 'GIT-044',
  start: 3,
  end: 4,
  run(s, { refs5 }) {
    s.say('Lan tách phần định dạng tiền ra repository riêng tien-te (remote của lab: work/remote/tien-te.git)');
    freshDir(P.libRemote);
    s.git(`init -q --bare -b main ${P.libRemote}`, { show: `init -q --bare -b main ${LIB_SHOW}` });
    freshDir(P.libWork);
    s.cd('../thu-vien', { show: false });
    s.git(`clone -q ${P.libRemote} tien-te`, { show: `clone -q ${LIB_SHOW} tien-te` });
    s.cd('tien-te', { show: false });
    s.write('src/Vnd.cs', LIB_V1);
    s.git('add -A', { quiet: true });
    s.git('commit -q -m "Định dạng tiền Việt Nam"', LAN('09:30'));
    s.git('push -q origin main');
    s.cd('../../bao-gia');
    s.blank();
    s.say('Minh thêm thư viện vào lib/tien-te. Git 2.38+ chặn submodule từ đường dẫn cục bộ:');
    s.say('URL tương đối ../tien-te.git: tính từ URL của origin (remote/bao-gia.git), nên đúng cho mọi người clone');
    s.git('submodule add ../tien-te.git lib/tien-te', { expectFail: true });
    s.say('Remote của lab là thư mục nên cho phép riêng lệnh này; với GitHub/GitLab (https, ssh) không cần.');
    s.git('-c protocol.file.allow=always submodule add ../tien-te.git lib/tien-te');
    s.print('$ Get-Content .gitmodules');
    s.print(s.read('.gitmodules').trimEnd());
    s.git('status -s');
    s.git('diff --cached --submodule=short');
    s.git('commit -q -m "Dùng thư viện tiền tệ chung qua submodule"', MINH('10:00'));
    s.git('submodule status');
    s.blank();
    s.say('Thư viện có phiên bản mới; repository chính vẫn ghi nhận commit cũ cho tới khi Minh cập nhật');
    s.cd('../thu-vien/tien-te', { show: false });
    s.write('src/Vnd.cs', LIB_V2);
    s.git('commit -q -am "Thêm dạng ngắn 31k"', LAN('10:30'));
    s.git('push -q origin main');
    s.cd('../../bao-gia', { show: false });
    s.git('-c protocol.file.allow=always submodule update --remote lib/tien-te');
    s.git('diff --submodule=log');
    s.git('commit -q -am "Cập nhật thư viện tiền tệ: dạng ngắn"', MINH('11:00'));
    s.git('push -q');
    s.blank();
    s.say('Phía Lan: pull về chỉ có thư mục rỗng và commit được ghi nhận; phải khởi tạo submodule');
    s.cd('../lan/bao-gia');
    s.git('pull -q --ff-only upstream main');
    s.git('submodule status');
    s.git('-c protocol.file.allow=always submodule update --init');
    s.git('submodule status');
    s.cd('../../bao-gia', { show: false });
    checkRefs5(s, refs5(4));
  },
};
