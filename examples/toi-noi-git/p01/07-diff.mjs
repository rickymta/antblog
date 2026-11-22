// GIT-007 · Diff: xem đúng thay đổi trước khi commit

import { GITATTRIBUTES, README, SETTINGS_V2 } from '../lib/history.mjs';
import { writeText } from '../lib/lab.mjs';

const README_EDIT = README + '\nChạy test: `dotnet test`.\n';
const SETTINGS_EDIT = SETTINGS_V2.replace('"PerKm": 2500', '"PerKm": 3000');

export default {
  id: 'GIT-007',
  start: 9,
  end: 10,
  prepare(dir) {
    writeText(dir, 'README.md', README_EDIT);
    writeText(dir, 'src/BaoGia/appsettings.json', SETTINGS_EDIT);
  },
  run(s, { hashes }) {
    s.say('Hai thay đổi: README.md (sẽ stage) và appsettings.json (chưa stage)');
    s.write('README.md', README_EDIT);
    s.write('src/BaoGia/appsettings.json', SETTINGS_EDIT);
    s.git('add README.md');
    s.git('status --short');
    s.blank();
    s.say('git diff: working tree so với staging area (phần CHƯA stage)');
    s.git('diff');
    s.say('git diff --staged: staging area so với HEAD (phần SẼ vào commit)');
    s.git('diff --staged');
    s.say('git diff HEAD: working tree so với HEAD (cả hai)');
    s.git('diff HEAD --stat');
    s.blank();
    s.say('Diff theo từ: thấy ngay con số nào đổi');
    s.git('diff --word-diff src/BaoGia/appsettings.json');
    s.blank();
    s.say('Trả lại như cũ');
    s.git('restore --staged README.md');
    s.git('restore README.md src/BaoGia/appsettings.json');
    s.git('status --short');
    s.blank();
    s.say('Diff giữa hai commit, giới hạn một file');
    s.git('diff HEAD~4 HEAD --stat');
    s.git('diff HEAD~4 HEAD -- src/BaoGia/FareCalculator.cs');
    s.blank();
    s.say('Đổi tên file: Git nhận ra rename nhờ so độ giống nội dung');
    s.git('diff --stat HEAD~1 HEAD');
    s.git('diff --stat --no-renames HEAD~1 HEAD');
    s.blank();
    s.say('Một trình soạn thảo trên Windows lưu appsettings.json với xuống dòng CRLF');
    s.write('src/BaoGia/appsettings.json', SETTINGS_V2.replace(/\n/g, '\r\n'));
    s.git('diff --stat');
    s.git('diff --ignore-cr-at-eol --stat');
    s.git('restore src/BaoGia/appsettings.json');
    s.blank();
    s.say('Thêm .gitattributes: chuẩn hoá xuống dòng ngay khi add');
    s.write('.gitattributes', GITATTRIBUTES);
    s.git('add .gitattributes');
    s.git('commit -m "Chuẩn hoá xuống dòng LF bằng .gitattributes"', { as: 'minh', at: '2026-11-10T08:45:00+07:00' });
    s.check('commit trùng c10', s.value('rev-parse HEAD') === hashes.get('c10'));
    s.git('ls-files --eol');
    s.blank();
    s.say('Lưu lại file với CRLF: Git không còn coi đó là thay đổi nội dung');
    s.write('src/BaoGia/appsettings.json', SETTINGS_V2.replace(/\n/g, '\r\n'));
    s.git('add src/BaoGia/appsettings.json');
    s.git('status --short');
    s.git('ls-files --eol src/BaoGia/appsettings.json');
    s.check('staging area không có thay đổi', s.value('diff --staged --name-only') === '');
    s.git('restore src/BaoGia/appsettings.json');
  },
};
