// GIT-001 · Git là gì và GitStudio giúp gì cho nhóm Tới Nơi
// Xem lịch sử của repository bao-gia ở cuối Phần 1 để thấy "lịch sử" trông thế nào.

export default {
  id: 'GIT-001',
  start: 10,
  end: 10,
  run(s, { hashes }) {
    s.say('Mười commit của dịch vụ báo giá, mới nhất ở trên cùng');
    s.git('log --oneline');
    s.blank();
    s.say('Một commit: ai, khi nào, vì sao, đổi file nào');
    s.git('show --stat HEAD~5');
    s.blank();
    s.check('HEAD trùng commit c10 của lịch sử chuẩn', s.value('rev-parse HEAD') === hashes.get('c10'));
    s.check('repository có đúng 10 commit', s.value('rev-list --count HEAD') === '10');
  },
};
