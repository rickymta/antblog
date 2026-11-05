// TEST-052 · Chuẩn bị dữ liệu cho phiên exploratory trên /driver.html (máy chủ lab phải bật API kiểm thử).
// Chạy: node articles/exploratory-testing/seed-session.mjs [baseUrl]   (mặc định http://localhost:3152)
// Tạo 3 chuyến xe máy ở SG (C001 7,4 km tiền mặt · C002 2 km VíXanh · C003 đồ ăn 3 km) + 1 chuyến ô tô (C001 10 km).
const base = process.argv[2] ?? 'http://localhost:3152';

async function call(method, path, { user, body, key } = {}) {
  const headers = { 'content-type': 'application/json' };
  if (user) headers['x-user'] = user;
  if (key) headers['idempotency-key'] = key;
  const res = await fetch(base + path, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(`${method} ${path} ⇒ ${res.status} ${JSON.stringify(data)}`);
  return data;
}

async function book(customer, quoteBody, paymentMethod = 'cash') {
  const { quote } = await call('POST', '/api/quotes', { user: `customer:${customer}`, body: quoteBody });
  const { trip } = await call('POST', '/api/trips', {
    user: `customer:${customer}`, key: `seed-${crypto.randomUUID()}`, body: { quoteId: quote.quoteId, paymentMethod },
  });
  return trip;
}

await call('POST', '/api/test/reset');
const trips = [
  await book('C001', { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 }),
  await book('C002', { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 2 }, 'vixanh'),
  await book('C003', { kind: 'food', cityCode: 'SG', distanceKm: 3, items: [{ name: 'Cơm tấm', unitPrice: 45000, quantity: 2 }] }),
  await book('C001', { kind: 'ride', cityCode: 'SG', vehicle: 'car', distanceKm: 10 }),
];
for (const t of trips) console.log(`${t.id}  ${t.customerId}  ${t.kind.padEnd(6)} ${t.vehicle}  ${t.distanceKm} km  ${t.fare} đ  ${t.paymentMethod}`);
