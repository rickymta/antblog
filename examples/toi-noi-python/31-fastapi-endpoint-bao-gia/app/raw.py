"""ASGI và WSGI ở dạng trần: hai hàm không dùng framework nào, để thấy FastAPI đứng trên cái gì."""

import json
from collections.abc import Callable, Iterable


# --- WSGI: đồng bộ. Server gọi hàm một lần cho mỗi request, trong một luồng (thread) ----
def wsgi_app(environ: dict, start_response: Callable) -> Iterable[bytes]:
    payload = json.dumps({"method": environ["REQUEST_METHOD"], "path": environ["PATH_INFO"]}).encode()
    start_response("200 OK", [("Content-Type", "application/json")])
    return [payload]  # trả về iterable các bytes; không có chỗ nào để `await`


# --- ASGI: bất đồng bộ. Server gọi coroutine với ba đối số: scope, receive, send ----------
async def asgi_app(scope: dict, receive: Callable, send: Callable) -> None:
    if scope["type"] != "http":
        return  # lifespan và websocket cũng đi qua cùng cổng này; hàm trần chỉ lo http
    body = b""
    while True:  # body đến theo từng mảnh: `more_body` cho biết còn nữa không
        message = await receive()
        body += message.get("body", b"")
        if not message.get("more_body"):
            break
    payload = json.dumps({"method": scope["method"], "path": scope["path"], "bytes_in": len(body)}).encode()
    await send(
        {
            "type": "http.response.start",
            "status": 200,
            "headers": [(b"content-type", b"application/json"), (b"content-length", str(len(payload)).encode())],
        }
    )
    await send({"type": "http.response.body", "body": payload})
