// Hàm báo giá tối giản cho lab: trả về route, danh tính lấy từ JWT claims và một giá mẫu.
// Với HTTP API payload 2.0, claims đã được API Gateway xác minh nằm ở
// requestContext.authorizer.jwt.claims; hàm không phải tự kiểm chữ ký token.
export const handler = async (event) => {
  const claims = event.requestContext?.authorizer?.jwt?.claims ?? {};
  const body = {
    route: event.routeKey,
    caller: claims.sub ?? 'anonymous',
    quote: { distanceKm: 5, fareVnd: 24000, note: 'giá minh hoạ của lab' },
  };
  return {
    statusCode: 200,
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  };
};
