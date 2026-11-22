// GIT-008 · Bên trong .git: blob, tree, commit, ref và HEAD

import { SETTINGS_V2 } from '../lib/history.mjs';

export default {
  id: 'GIT-008',
  start: 10,
  end: 10,
  run(s, { hashes }) {
    s.say('HEAD là một tham chiếu tới nhánh, nhánh là tham chiếu tới commit');
    s.say(`.git/HEAD chứa: ${s.read('.git/HEAD').trim()}`);
    s.git('symbolic-ref HEAD');
    s.git('rev-parse HEAD');
    s.say(`.git/refs/heads/main chứa: ${s.read('.git/refs/heads/main').trim()}`);
    s.blank();
    s.say('Commit object: trỏ tới một tree, commit cha, người viết, message');
    s.git('cat-file -t HEAD');
    s.git('cat-file -p HEAD');
    s.blank();
    s.say('Tree object: danh sách tên → blob/tree');
    s.git('cat-file -p HEAD^{tree}');
    s.git('ls-tree HEAD src/BaoGia/');
    s.blank();
    s.say('Blob object: chỉ có nội dung, không có tên file');
    const blob = s.value('rev-parse HEAD:src/BaoGia/appsettings.json');
    s.git(`cat-file -t ${blob.slice(0, 7)}`);
    s.git(`cat-file -s ${blob.slice(0, 7)}`);
    s.git('hash-object src/BaoGia/appsettings.json');
    s.check('hash-object của file = blob trong tree', s.value('hash-object src/BaoGia/appsettings.json') === blob);
    s.blank();
    s.say('Đổi một chữ số trong nội dung → blob khác hẳn');
    s.git('hash-object --stdin', { input: SETTINGS_V2.replace('"PerKm": 5000', '"PerKm": 5500'), show: 'hash-object --stdin   # nội dung với PerKm 5500' });
    s.blank();
    s.say('Commit sau amend: AuthorDate và CommitDate khác nhau (GIT-005)');
    s.git(`cat-file -p ${hashes.get('c7').slice(0, 7)}`);
    s.blank();
    s.say('Tất cả object trong repository, đếm theo loại');
    const all = s.value('cat-file --batch-all-objects --batch-check=%(objecttype)').split('\n');
    const count = all.reduce((m, t) => ((m[t] = (m[t] ?? 0) + 1), m), {});
    s.git('cat-file --batch-all-objects --batch-check', { quiet: true });
    s.say(`git cat-file --batch-all-objects --batch-check → commit: ${count.commit}, tree: ${count.tree}, blob: ${count.blob}`);
    s.check('có đúng 10 commit object', count.commit === 10);
    s.blank();
    s.say('Object lưu dưới .git/objects/<2 ký tự đầu>/<38 ký tự còn lại>, nén zlib');
    s.check('file object của blob tồn tại', s.exists(`.git/objects/${blob.slice(0, 2)}/${blob.slice(2)}`));
  },
};
