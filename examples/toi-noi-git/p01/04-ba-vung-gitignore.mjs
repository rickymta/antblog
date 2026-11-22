// GIT-004 · Working tree, staging area và repository; file bị bỏ qua

import { CALC_V1, GITIGNORE, SETTINGS_V1 } from '../lib/history.mjs';
import { writeText } from '../lib/lab.mjs';

/** Giả lập kết quả build .NET và file riêng trên máy (lab không cần cài .NET). */
function fakeBuild(s) {
  s.write('src/BaoGia/bin/Debug/net10.0/BaoGia.dll', 'MZ giả lập kết quả build\n');
  s.write('src/BaoGia/obj/project.assets.json', '{ "version": 3 }\n');
  s.write('src/BaoGia/BaoGia.csproj.user', '<Project />\n');
  s.write('src/BaoGia/appsettings.Development.json', '{ "Fare": { "Car4": { "PerKm": 1 } } }\n');
  s.write('.env', 'MAPS_API_KEY=khoa-that-khong-duoc-commit\n');
}

export default {
  id: 'GIT-004',
  start: 1,
  end: 3,
  prepare(dir) {
    // Trạng thái đầu bài cho GitStudio: hai file mới chưa theo dõi + kết quả build giả lập.
    writeText(dir, 'src/BaoGia/FareCalculator.cs', CALC_V1);
    writeText(dir, 'src/BaoGia/appsettings.json', SETTINGS_V1);
  },
  run(s, { hashes }) {
    s.say('Hai file mới: Git chưa theo dõi (untracked)');
    s.write('src/BaoGia/FareCalculator.cs', CALC_V1);
    s.write('src/BaoGia/appsettings.json', SETTINGS_V1);
    s.git('status');
    s.git('status --short --untracked-files=all');
    s.blank();
    s.say('Stage một file: chụp nội dung lúc này vào staging area');
    s.git('add src/BaoGia/FareCalculator.cs');
    s.git('status --short --untracked-files=all');
    s.blank();
    s.say('Sửa tiếp file đã stage: bản trong staging area và bản trong working tree khác nhau');
    s.write('src/BaoGia/FareCalculator.cs', CALC_V1.replace('public static class', '// TODO: phụ phí giờ cao điểm\npublic static class'));
    s.git('status --short --untracked-files=all');
    s.say('Bỏ dòng TODO vừa thêm: working tree lại giống staging area');
    s.write('src/BaoGia/FareCalculator.cs', CALC_V1);
    s.git('status --short --untracked-files=all');
    s.blank();
    s.git('add src/BaoGia/appsettings.json');
    s.git('commit -m "Thêm hàm tính cước theo km"', { as: 'minh', at: '2026-11-02T09:10:00+07:00' });
    s.git('status');
    s.check('commit trùng c2', s.value('rev-parse HEAD') === hashes.get('c2'));
    s.blank();
    s.say('Build project: thư mục bin/, obj/ và file riêng của máy xuất hiện');
    fakeBuild(s);
    s.git('status --short --untracked-files=all');
    s.blank();
    s.say('Viết .gitignore');
    s.write('.gitignore', GITIGNORE);
    s.git('status --short --untracked-files=all');
    s.git('status --short --ignored');
    s.git('check-ignore -v src/BaoGia/bin/Debug/net10.0/BaoGia.dll .env src/BaoGia/BaoGia.csproj.user');
    s.git('add .gitignore');
    s.git('commit -m "Bỏ qua kết quả build và cấu hình máy cá nhân"', { as: 'minh', at: '2026-11-02T09:25:00+07:00' });
    s.git('ls-files');
    s.check('commit trùng c3', s.value('rev-parse HEAD') === hashes.get('c3'));
    s.check('không file build nào bị theo dõi', !/bin\/|obj\/|\.env/.test(s.value('ls-files')));
    s.blank();
    s.say('Bẫy: file ĐÃ được theo dõi thì .gitignore không còn tác dụng với nó');
    s.cd('..');
    s.git('init -q -b main thu-nghiem-ignore');
    s.cd('thu-nghiem-ignore');
    s.write('.env', 'MAPS_API_KEY=abc\n');
    s.git('add .env');
    s.git('commit -q -m "Lỡ commit .env"', { as: 'minh', at: '2026-11-02T10:00:00+07:00' });
    s.write('.gitignore', '.env\n');
    s.write('.env', 'MAPS_API_KEY=xyz\n');
    s.git('status --short');
    s.git('rm --cached .env');
    s.git('add .gitignore');
    s.git('status --short');
    s.git('commit -q -m "Ngừng theo dõi .env, thêm .gitignore"', { as: 'minh', at: '2026-11-02T10:05:00+07:00' });
    s.git('status --short --ignored');
    s.check('.env không còn trong commit mới nhất', !s.value('ls-files').includes('.env'));
    s.check('.env vẫn nằm trong commit cũ', s.value('ls-tree --name-only HEAD~1') === '.env');
  },
};
