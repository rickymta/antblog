// Sinh file log mẫu của dịch vụ bao-gia (cố định: không dùng Date.now hay Math.random).
// Chạy: node tools/tao-log.mjs > docker/fixtures/bao-gia.log
let seed = 20261122;
const rnd = () => (seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648;
const pick = (a) => a[Math.floor(rnd() * a.length)];
const services = ['car4', 'car7', 'bike', 'delivery'];
const lines = [];
let t = Date.UTC(2026, 10, 20, 23, 0, 0); // 2026-11-21 06:00:00 +07
const stamp = () => {
  const d = new Date(t + 7 * 3600e3);
  return d.toISOString().replace('T', ' ').slice(0, 19);
};
lines.push(`${stamp()} INFO  bao-gia 1.4.2 khoi dong, cong 8080, moi truong production`);
lines.push(`${stamp()} INFO  nap bang gia: 4 loai dich vu tu /opt/toinoi/bao-gia/bang-gia.csv`);
for (let i = 1; i <= 360; i++) {
  t += (5 + Math.floor(rnd() * 115)) * 1000;
  const id = `q-${String(100000 + i)}`;
  const svc = pick(services);
  const km = (1 + rnd() * 24).toFixed(1);
  const r = rnd();
  if (r < 0.05) {
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
