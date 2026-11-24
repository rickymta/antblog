// GIT-057 · Gộp và tách repository giữ lịch sử: subtree

import { rmSync } from 'node:fs';
import { join } from 'node:path';

import { checkRefs6, P } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-12-04T${t}:00+07:00` });

export default {
  id: 'GIT-057',
  start: 6,
  end: 7,
  run(s, { refs6 }) {
    s.say('Hiện tại: thư viện tiền tệ là submodule, trỏ tới một commit của repository tien-te');
    s.git('submodule status');
    s.git('-C ../remote/tien-te.git log --oneline main');
    s.blank();
    s.say('Bước 1: bỏ submodule (gỡ khỏi index và .gitmodules, xoá bản clone trong .git/modules)');
    s.git('submodule deinit -q -f lib/tien-te');
    s.git('rm -q lib/tien-te');
    s.git('rm -q -f .gitmodules');
    rmSync(join(P.minh, '.git', 'modules', 'lib', 'tien-te'), { recursive: true, force: true });
    s.print('$ Remove-Item -Recurse -Force .git/modules/lib/tien-te');
    s.git('status -s');
    s.git('commit -q -m "refactor(tien-te): bỏ submodule lib/tien-te"', M('09:00'));
    s.blank();
    s.say('Bước 2: gộp toàn bộ lịch sử tien-te vào thư mục lib/tien-te bằng một merge commit');
    s.git('subtree add --prefix=lib/tien-te ../remote/tien-te.git main -m "refactor(tien-te): gộp thư viện tiền tệ bằng subtree, giữ lịch sử"', M('09:30'));
    s.git('log --oneline --graph -6');
    s.git('ls-files lib/tien-te');
    s.git('blame -s lib/tien-te/src/Vnd.cs');
    s.git('push -q');
    s.blank();
    s.say('Đi ngược lại: subtree split dựng lại lịch sử chỉ của một thư mục; ra đúng commit cuối của tien-te');
    const split = s.value('subtree split -q --prefix=lib/tien-te');
    s.print('$ git subtree split -q --prefix=lib/tien-te');
    s.print(split);
    s.git('-C ../remote/tien-te.git rev-parse main');
    s.check('subtree split trùng commit cuối của tien-te', split === s.value('-C ../remote/tien-te.git rev-parse main'));
    s.blank();
    s.say('Tách: bộ phận vận hành nhận một repository chỉ có docs/, giữ lịch sử từng file');
    s.git('init -q --bare -b main ../remote/tai-lieu.git');
    s.git('subtree split -q --prefix=docs -b tach/tai-lieu', { quiet: true });
    s.print('$ git subtree split -q --prefix=docs -b tach/tai-lieu');
    const url = `file:///${P.docsRemote.replace(/^\//, '')}`;
    s.git(`push -q ${url} tach/tai-lieu:main`, { quiet: true });
    s.print('$ git push -q file:///<lab>/work/remote/tai-lieu.git tach/tai-lieu:main');
    s.git('branch -q -D tach/tai-lieu');
    s.git('-C ../remote/tai-lieu.git log --oneline -4 main');
    s.git('-C ../remote/tai-lieu.git ls-tree --name-only main');
    s.check('không còn .gitmodules, lib/tien-te là thư mục thường', !s.exists('.gitmodules') && s.exists('lib/tien-te/src/Vnd.cs'));
    checkRefs6(s, refs6(7));
  },
};
