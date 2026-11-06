// Chạy: node --test functions/   (không cần AWS; nạp mã function giống cách CloudFront gọi handler)
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const code = readFileSync(fileURLToPath(new URL('./redirect.js', import.meta.url)), 'utf8');
const handler = new Function(`${code}; return handler;`)();

const ev = (uri, host = 'www.toinoi.example') => ({
  request: { method: 'GET', uri, headers: { host: { value: host } }, cookies: {}, querystring: {} },
});

test('apex chuyển sang www và giữ đường dẫn', () => {
  const res = handler(ev('/gioi-thieu', 'toinoi.example'));
  assert.equal(res.statusCode, 301);
  assert.equal(res.headers.location.value, 'https://www.toinoi.example/gioi-thieu');
});

test('đường dẫn cũ /dat-xe chuyển sang /ride/', () => {
  const res = handler(ev('/dat-xe'));
  assert.equal(res.statusCode, 301);
  assert.equal(res.headers.location.value, '/ride/');
});

test('thư mục thêm index.html', () => {
  assert.equal(handler(ev('/ride/')).uri, '/ride/index.html');
});

test('tệp tĩnh giữ nguyên', () => {
  assert.equal(handler(ev('/assets/app.3f9a1c.js')).uri, '/assets/app.3f9a1c.js');
});
