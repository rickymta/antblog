// Sinh file log mẫu của dịch vụ bao-gia (cố định: không dùng Date.now hay Math.random).
//
//   node tools/tao-log.mjs > docker/fixtures/bao-gia.log                  ngày 21/11 (Phần 1)
//   node tools/tao-log.mjs 2026-11-20 > docker/fixtures/p02/bao-gia-2026-11-20.log
//   node tools/tao-log.mjs 2026-11-19 > docker/fixtures/p02/bao-gia-2026-11-19.log
//
// Ngày 20/11 có một lần nâng cấp 1.4.1 → 1.4.2 lúc 09:15 (cùng lúc bảng giá được sửa) và
// một sự cố đọc bảng giá ngay trước đó; ngày 19/11 chạy bản 1.4.1 cả ngày.

const DAYS = {
  // ngày: [seed, số yêu cầu, mã yêu cầu đầu tiên]
  '2026-11-21': [20261122, 360, 100001],
  '2026-11-20': [20261120, 300, 99701],
  '2026-11-19': [20261119, 300, 99401],
};
const day = process.argv[2] || '2026-11-21';
if (!DAYS[day]) throw new Error('Ngày không có trong lab: ' + day);
let [seed, count, firstId] = DAYS[day];

const rnd = () => (seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648;
const pick = (a) => a[Math.floor(rnd() * a.length)];
const services = ['car4', 'car7', 'bike', 'delivery'];
const lines = [];
const [y, m, d] = day.split('-').map(Number);
let t = Date.UTC(y, m - 1, d - 1, 23, 0, 0); // 06:00:00 +07 của ngày đó
const stamp = () => new Date(t + 7 * 3600e3).toISOString().replace('T', ' ').slice(0, 19);
const clock = () => stamp().slice(11, 16);

let version = day === '2026-11-21' ? '1.4.2' : '1.4.1';
const start = () => {
  lines.push(`${stamp()} INFO  bao-gia ${version} khoi dong, cong 8080, moi truong production`);
  lines.push(`${stamp()} INFO  nap bang gia: 4 loai dich vu tu /opt/toinoi/bao-gia/bang-gia.csv`);
};
start();

let upgraded = false;
for (let i = 0; i < count; i++) {
  t += (5 + Math.floor(rnd() * 115)) * 1000;
  if (day === '2026-11-20' && !upgraded && clock() >= '09:15') {
    // Nâng cấp: dừng bản cũ, khởi động bản mới với bảng giá mới.
    lines.push(`${stamp()} INFO  nhan tin hieu SIGTERM, dung bao-gia ${version}`);
    t += 40 * 1000;
    version = '1.4.2';
    start();
    upgraded = true;
    t += 5 * 1000;
  }
  const id = `q-${String(firstId + i).padStart(6, '0')}`;
  const svc = pick(services);
  const km = (1 + rnd() * 24).toFixed(1);
  const r = rnd();
  const incident = day === '2026-11-20' && clock() >= '09:05' && clock() < '09:15';
  if (incident && r < 0.6) {
    lines.push(`${stamp()} ERROR ${id} dich_vu=${svc} khong doc duoc bang gia: file dang bi khoa`);
  } else if (r < 0.05) {
    lines.push(`${stamp()} ERROR ${id} dich_vu=${svc} khong lay duoc quang duong: timeout sau 3000 ms goi ban-do`);
  } else if (r < 0.13) {
    const ms = 800 + Math.floor(rnd() * 1900);
    lines.push(`${stamp()} WARN  ${id} dich_vu=${svc} km=${km} bao gia cham: ${ms} ms`);
  } else {
    const base = { car4: 12000, car7: 15000, bike: 8000, delivery: 10000 }[svc];
    const per = { car4: 4500, car7: 5500, bike: 2500, delivery: 3000 }[svc];
    const fare = Math.ceil((base + per * Number(km)) / 1000) * 1000;
    const ms = 20 + Math.floor(rnd() * 160);
    lines.push(`${stamp()} INFO  ${id} dich_vu=${svc} km=${km} gia=${fare} ${ms} ms`);
  }
}
process.stdout.write(lines.join('\n') + '\n');
