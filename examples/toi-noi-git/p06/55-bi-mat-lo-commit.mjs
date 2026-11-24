// GIT-055 · Bí mật lỡ commit: xoay khoá, xoá khỏi lịch sử, force push có phối hợp

import { rmSync } from 'node:fs';
import { join } from 'node:path';

import {
  DOC_MAPS,
  FILTER_ARGS,
  FILTER_ENV,
  GITIGNORE_STAGING,
  HOOK_PRE_COMMIT_SECRET,
  MAPS_TESTS,
  MAPS_V1,
  README_MAPS,
  SECRET_LINE,
  STAGING_JSON,
} from '../lib/history6.mjs';
import { SECRET_RE } from './kiem-tra.mjs';
import { checkRefs6, P } from './chung.mjs';

const L = (t) => ({ as: 'lan', at: `2026-12-03T${t}:00+07:00` });
const M = (t) => ({ as: 'minh', at: `2026-12-03T${t}:00+07:00` });
const SEARCH = `log --all --oneline -G "${SECRET_RE}"`;

export default {
  id: 'GIT-055',
  start: 4,
  end: 5,
  run(s, { refs6 }) {
    s.say('Lan làm tính năng bản đồ trong fork, lỡ commit file cấu hình staging có khoá API');
    s.cd('../lan/bao-gia');
    s.git('fetch -q upstream');
    s.git('switch -c tinh-nang/ban-do upstream/main');
    s.write('src/BaoGia/MapsDistance.cs', MAPS_V1);
    s.write('src/BaoGia/appsettings.Staging.json', STAGING_JSON);
    s.git('add src/BaoGia');
    s.git('commit -q -m "feat(ban-do): ước lượng quãng đường bằng dịch vụ bản đồ"', L('09:00'));
    s.write('tests/BaoGia.Tests/MapsDistanceTests.cs', MAPS_TESTS);
    s.git('add tests');
    s.git('commit -q -m "test(ban-do): làm tròn quãng đường một chữ số thập phân"', L('09:30'));
    s.git('push -q -u origin tinh-nang/ban-do');
    s.blank();
    s.say('Minh review pull request, thấy khoá. Tìm mọi commit có chuỗi giống khoá trong mọi nhánh:');
    s.git(SEARCH);
    s.git('show --stat --format=%s HEAD~1');
    s.blank();
    s.say('Bước 1 (ngoài Git): thu hồi khoá cũ, cấp khoá mới trên trang quản trị dịch vụ bản đồ.');
    s.say('Khoá đã lên remote coi như đã lộ: xoá khỏi lịch sử không làm nó an toàn trở lại.');
    s.blank();
    s.say('Bước 2: viết lại nhánh, bỏ file khỏi mọi commit của nhánh');
    s.say('(git filter-repo --invert-paths --path src/BaoGia/appsettings.Staging.json --refs upstream/main..tinh-nang/ban-do;');
    s.say(' lab dùng filter-branch có sẵn trong Git để không phải cài thêm)');
    s.git(FILTER_ARGS.map((a) => (a.includes(' ') ? `"${a}"` : a)).join(' '), { quiet: true, env: FILTER_ENV });
    s.print(`$ git ${FILTER_ARGS.map((a) => (a.includes(' ') ? `"${a}"` : a)).join(' ')}`);
    s.git('update-ref -d refs/original/refs/heads/tinh-nang/ban-do');
    s.git('log --oneline upstream/main..tinh-nang/ban-do');
    s.git('show --stat --format=%s HEAD~1');
    s.blank();
    s.say('Bước 3: không để tái diễn: bỏ qua file, ghi cách cấu hình khoá');
    rmSync(join(P.lan, 'src', 'BaoGia', 'appsettings.Staging.json'), { force: true });
    s.write('.gitignore', GITIGNORE_STAGING(s.read('.gitignore')));
    s.write('docs/ban-do.md', DOC_MAPS);
    s.write('README.md', README_MAPS(s.read('README.md')));
    s.git('add .gitignore docs/ban-do.md README.md');
    s.git('commit -q -m "chore(ban-do): khoá API đọc từ biến môi trường, không commit"', L('10:30'));
    s.blank();
    s.say('Bước 4: thay nhánh trên remote. --force-with-lease chỉ ghi đè nếu remote vẫn là bản mình đã thấy');
    s.git('push -q --force-with-lease origin tinh-nang/ban-do');
    s.say('Bản cũ vẫn còn trong reflog và object rời: quên reflog, xoá object không còn ai trỏ tới');
    s.git('reflog expire --expire=now --all');
    s.git('gc -q --prune=now', { env: { GIT_CONFIG_COUNT: '1', GIT_CONFIG_KEY_0: 'pack.threads', GIT_CONFIG_VALUE_0: '1' } });
    s.git(SEARCH);
    s.check('nhánh của Lan và fork không còn khoá', s.value(SEARCH) === '' && s.value(`--git-dir=${P.fork} ${SEARCH}`) === '');
    s.blank();
    s.say('Minh merge pull request (bản đã sạch), rồi thêm chặn khoá vào hook pre-commit của nhóm');
    s.cd('../../bao-gia');
    s.git('fetch -q ../remote/lan-bao-gia.git tinh-nang/ban-do');
    s.git('merge -q --no-ff -m "Merge pull request #7 từ lan/tinh-nang/ban-do" FETCH_HEAD', M('11:00'));
    s.write('.githooks/pre-commit', HOOK_PRE_COMMIT_SECRET(s.read('.githooks/pre-commit')));
    s.write('src/BaoGia/appsettings.Thu.json', `{\n  "Maps": {\n${SECRET_LINE}\n  }\n}\n`);
    s.git('add src/BaoGia/appsettings.Thu.json');
    s.git('commit -q -m "chore(ban-do): thử cấu hình"', { ...M('11:20'), expectFail: true });
    s.git('rm -q --cached src/BaoGia/appsettings.Thu.json');
    s.remove('src/BaoGia/appsettings.Thu.json');
    s.git('commit -q -am "chore(hook): chặn khoá API tn_live_ trong thay đổi đã stage"', M('11:30'));
    s.git('push -q');
    s.check('lịch sử main không có khoá', s.value(SEARCH) === '');
    checkRefs6(s, refs6(5));
  },
};
