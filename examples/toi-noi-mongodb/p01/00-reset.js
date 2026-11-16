// P01 · 00-reset.js — xoá và tạo lại database `toinoi` (MG-001, MG-002).
// Chạy lại được bao nhiêu lần cũng được. KHÔNG chạy trên database thật.

const old = db.adminCommand({ listDatabases: 1, nameOnly: true })
  .databases.some((d) => d.name === 'toinoi');
if (old) {
  db.dropDatabase();
  print('Đã xoá database toinoi cũ');
}

section('Phiên bản và vai trò của server');
const hello = db.hello();
table([{
  phien_ban: db.version(),
  replica_set: hello.setName,
  vai_tro: hello.isWritablePrimary ? 'PRIMARY' : 'khác',
  database: db.getName(),
  fcv: db.adminCommand({ getParameter: 1, featureCompatibilityVersion: 1 }).featureCompatibilityVersion.version,
}]);
expectValue('phiên bản 8.0.x', db.version().slice(0, 4), '8.0.');
expectValue('replica set', hello.setName, 'rs0');

section('Database có sẵn trong instance (trước khi tạo dữ liệu)');
// Database `toinoi` chưa có trong danh sách: nó chỉ thật sự được tạo khi có collection đầu tiên.
table(db.adminCommand({ listDatabases: 1, nameOnly: true }).databases.map((d) => ({ name: d.name })), ['name']);
