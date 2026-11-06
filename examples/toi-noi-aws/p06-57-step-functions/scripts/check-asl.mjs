// Kiểm cấu trúc ASL trên máy, không cần tài khoản AWS: node scripts/check-asl.mjs asl/*.asl.json
// Kiểm: JSON parse được, StartAt tồn tại, mọi Next/Default/Catch trỏ tới state có thật,
// state không phải Succeed/Fail/Choice phải có Next hoặc End, không có state mồ côi, JSONata {% %} cân.
import fs from 'node:fs';

let bad = 0;
const fail = (file, msg) => { bad++; console.log(`  ✗ ${file}: ${msg}`); };

function checkMachine(file, def, path) {
  const states = def.States ?? {};
  if (!def.StartAt || !(def.StartAt in states)) fail(file, `${path}: StartAt "${def.StartAt}" không tồn tại`);
  const reachable = new Set();
  const visit = (name) => {
    if (reachable.has(name) || !(name in states)) return;
    reachable.add(name);
    const s = states[name];
    for (const t of nextTargets(s)) visit(t);
  };
  visit(def.StartAt);
  for (const [name, s] of Object.entries(states)) {
    const where = `${path}/${name}`;
    for (const t of nextTargets(s)) if (!(t in states)) fail(file, `${where}: trỏ tới state không có "${t}"`);
    if (!['Succeed', 'Fail', 'Choice'].includes(s.Type) && !s.Next && !s.End) fail(file, `${where}: thiếu Next hoặc End`);
    if (!reachable.has(name)) fail(file, `${where}: không với tới được từ StartAt`);
    if (s.Type === 'Task' && s.HeartbeatSeconds && s.TimeoutSeconds && s.HeartbeatSeconds >= s.TimeoutSeconds)
      fail(file, `${where}: HeartbeatSeconds phải nhỏ hơn TimeoutSeconds`);
    for (const proc of [s.ItemProcessor, s.Iterator].filter(Boolean)) checkMachine(file, proc, `${where}/processor`);
    for (const b of s.Branches ?? []) checkMachine(file, b, `${where}/branch`);
  }
}

function nextTargets(s) {
  const out = [];
  if (s.Next) out.push(s.Next);
  if (s.Default) out.push(s.Default);
  for (const c of s.Choices ?? []) if (c.Next) out.push(c.Next);
  for (const c of s.Catch ?? []) if (c.Next) out.push(c.Next);
  return out;
}

function checkJsonata(file, node, path) {
  if (typeof node === 'string') {
    const open = (node.match(/\{%/g) ?? []).length;
    const close = (node.match(/%\}/g) ?? []).length;
    if (open !== close) fail(file, `${path}: {% %} không cân`);
    if (open > 0 && !(node.trim().startsWith('{%') && node.trim().endsWith('%}'))) fail(file, `${path}: biểu thức JSONata phải bao trọn chuỗi`);
  } else if (node && typeof node === 'object') {
    for (const [k, v] of Object.entries(node)) checkJsonata(file, v, `${path}.${k}`);
  }
}

for (const file of process.argv.slice(2)) {
  console.log(file);
  let def;
  try { def = JSON.parse(fs.readFileSync(file, 'utf8')); } catch (e) { fail(file, 'JSON không parse: ' + e.message); continue; }
  if (def.QueryLanguage !== 'JSONata') fail(file, 'QueryLanguage nên là JSONata');
  checkMachine(file, def, '');
  checkJsonata(file, def, '$');
  const placeholders = [...new Set([...fs.readFileSync(file, 'utf8').matchAll(/\$\{([a-z_]+)\}/g)].map((m) => m[1]))];
  console.log(`  · ${Object.keys(def.States).length} state, placeholder templatefile: ${placeholders.join(', ')}`);
}
console.log(bad ? `\n${bad} lỗi.` : '\nOK, không lỗi.');
process.exit(bad ? 1 : 0);
