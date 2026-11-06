// CloudFront Functions (runtime cloudfront-js-2.0), sự kiện viewer-request.
// 1) toinoi.example -> https://www.toinoi.example (giữ nguyên đường dẫn)
// 2) /dat-xe (đường dẫn cũ) -> /ride/
// 3) URI kết thúc bằng "/" thì thêm index.html (S3 qua OAC không tự phục vụ index của thư mục con)
function handler(event) {
  var request = event.request;
  var host = request.headers.host ? request.headers.host.value : '';
  var uri = request.uri;

  if (host === 'toinoi.example') {
    return {
      statusCode: 301,
      statusDescription: 'Moved Permanently',
      headers: { location: { value: 'https://www.toinoi.example' + uri } },
    };
  }

  if (uri === '/dat-xe' || uri === '/dat-xe/') {
    return {
      statusCode: 301,
      statusDescription: 'Moved Permanently',
      headers: { location: { value: '/ride/' } },
    };
  }

  if (uri.endsWith('/')) {
    request.uri = uri + 'index.html';
  }
  return request;
}
