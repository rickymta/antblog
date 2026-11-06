// Tính error budget và burn rate. Chỉ dùng Node, không cần AWS.
// Dùng: node slo-math.mjs [slo %] [số ngày] [error ratio quan sát được]
//   node slo-math.mjs 99.9 30
//   node slo-math.mjs 99.9 30 0.0144    # tỉ lệ lỗi 1,44% -> burn rate 14,4
const slo = Number(process.argv[2] ?? 99.9);
const days = Number(process.argv[3] ?? 30);
const observed = process.argv[4] === undefined ? undefined : Number(process.argv[4]);

if (!(slo > 0 && slo < 100) || !(days > 0)) {
  console.error('Dùng: node slo-math.mjs <slo %, 0-100> <số ngày> [error ratio]');
  process.exit(1);
}

const budgetRatio = 1 - slo / 100;
const windowMinutes = days * 24 * 60;
const fmt = (n) => Number(n.toFixed(2)).toLocaleString('vi-VN');
const dec = (n, d) => n.toFixed(d).replace('.', ',');

console.log(`SLO ${slo}% trong ${days} ngày`);
console.log(`  Error budget: ${dec(budgetRatio * 100, 3)}% = ${fmt(windowMinutes * budgetRatio)} phút lỗi toàn phần cho phép`);
console.log('');
console.log('  Burn rate | Hết budget sau | Đốt 1 giờ bằng ... % budget cả kỳ');
for (const rate of [1, 3, 6, 14.4, 36]) {
  const hours = (days * 24) / rate;
  const pctPerHour = (rate / (days * 24)) * 100;
  console.log(`  ${String(rate).padStart(9)} | ${fmt(hours).padStart(10)} giờ | ${dec(pctPerHour, 2)}%`);
}

if (observed !== undefined) {
  const burn = observed / budgetRatio;
  console.log('');
  console.log(`Tỉ lệ lỗi quan sát ${dec(observed * 100, 3)}% -> burn rate ${fmt(burn)}; hết budget sau ${fmt((days * 24) / burn)} giờ nếu giữ mức này`);
}
