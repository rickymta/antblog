#!/bin/bash
# User data: dựng service "quote" giả lập nghe cổng 8080. Chỉ dùng để thử ALB/ASG, không phải service thật.
set -euxo pipefail
mkdir -p /opt/toinoi /var/log/toinoi

cat >/opt/toinoi/quote.py <<'PY'
import json
import logging
import socket
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

logging.basicConfig(filename="/var/log/toinoi/quote.log", level=logging.INFO,
                    format="%(asctime)s %(levelname)s %(message)s")
HOST = socket.gethostname()


class Handler(BaseHTTPRequestHandler):
    def _send(self, code, body, ctype="application/json"):
        data = body.encode()
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path == "/health/ready":
            self._send(200, "ready", "text/plain")
        elif self.path.startswith(("/v1/quote", "/quote")):
            logging.info("quote ok path=%s", self.path)
            self._send(200, json.dumps({"distanceKm": 5, "fareVnd": 25500, "host": HOST}))
        elif self.path == "/boom":
            logging.error("ERROR quote failed path=%s", self.path)
            self._send(500, json.dumps({"error": "boom"}))
        else:
            self._send(404, json.dumps({"error": "not found"}))

    def log_message(self, *args):
        pass


ThreadingHTTPServer(("0.0.0.0", 8080), Handler).serve_forever()
PY

cat >/etc/systemd/system/toinoi-quote.service <<'UNIT'
[Unit]
Description=Toi Noi quote service (lab)
After=network.target

[Service]
ExecStart=/usr/bin/python3 /opt/toinoi/quote.py
Restart=always

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable --now toinoi-quote
