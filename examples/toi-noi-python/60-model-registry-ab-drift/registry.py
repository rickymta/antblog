"""Registry mô hình dạng thư mục: mỗi phiên bản bất biến, một con trỏ `current`, lịch sử chỉ ghi thêm.

    out/registry/
      <model_version>/model.joblib        do model_io.save_model ghi (kèm sha256)
      <model_version>/model_meta.json
      <model_version>/manifest.json       ai đăng ký, khi nào, bản cha, số đo holdout, ghi chú
      current.json                        {"version": ..., "since": ...}: bản đang phục vụ
      history.jsonl                       registered / promoted / rolled_back, mỗi dòng một sự kiện
      .lock                               khóa tệp (fcntl.flock) cho mọi thao tác ghi

Quy tắc: không ghi đè phiên bản đã đăng ký; không promote bản chưa qua cổng (số đo holdout, không chồng lên train)
trừ khi `force=True` kèm lý do (được ghi vào lịch sử); rollback KHÔNG huấn luyện lại, chỉ đưa con trỏ về bản
đứng dưới trong ngăn xếp promote. CẢNH BÁO: joblib là pickle, chỉ nạp file do chính pipeline của mình tạo; ai được ghi vào
thư mục registry là việc của hạ tầng (quyền truy cập), không nằm trong lab.
"""

import fcntl
import json
import os
import shutil
import uuid
from contextlib import contextmanager
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from model_io import ModelLoadError, ScoringModel, load_model, save_model

MANIFEST = "manifest.json"
CURRENT = "current.json"
HISTORY = "history.jsonl"
GATE_TOLERANCE = 0.01  # bản mới được kém champion tối đa chừng này PR-AUC holdout (giả định của lab)


class RegistryError(RuntimeError):
    pass


def now_iso() -> str:
    return datetime.now(UTC).isoformat(timespec="seconds")


def _write_json_atomic(path: Path, data: dict[str, Any]) -> None:
    tmp = path.with_name(f".{path.name}.{uuid.uuid4().hex[:8]}.tmp")
    with open(tmp, "w", encoding="utf-8") as f:
        f.write(json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True))
        f.flush()
        os.fsync(f.fileno())  # ghi xuống đĩa trước khi đổi tên: mất điện không để lại con trỏ rỗng
    os.replace(tmp, path)  # đổi tên nguyên tử: người đọc thấy bản cũ hoặc bản mới, không thấy bản dở


class Registry:
    def __init__(self, root: Path) -> None:
        self.root = root
        root.mkdir(parents=True, exist_ok=True)
        self._cache: dict[str, ScoringModel] = {}

    # ---- khóa và lịch sử ----

    @contextmanager
    def _locked(self):
        with open(self.root / ".lock", "a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            try:
                yield
            finally:
                fcntl.flock(lock, fcntl.LOCK_UN)

    def history(self) -> list[dict[str, Any]]:
        path = self.root / HISTORY
        return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()] if path.exists() else []

    def _append(self, event: dict[str, Any]) -> None:
        with open(self.root / HISTORY, "a", encoding="utf-8") as f:
            f.write(json.dumps({"at": now_iso(), **event}, ensure_ascii=False, sort_keys=True) + "\n")

    def _check_consistent(self) -> None:
        """Con trỏ và lịch sử là HAI file, ghi hai bước: chết giữa hai bước thì lệch nhau. Lệch thì từ chối mọi thao tác ghi
        (promote, rollback) cho tới khi người vận hành xem lịch sử và sửa tay; không đoán bản nào đúng."""
        stack = self._stack()
        expected = stack[-1] if stack else None
        if self.current() != expected:
            raise RegistryError(f"con trỏ current ({self.current()}) lệch lịch sử ({expected}): dừng, cần người xem history.jsonl")

    def _stack(self) -> list[str]:
        """Phát lại lịch sử: promote đẩy vào, rollback lấy ra. Phần tử cuối là bản đang phục vụ."""
        stack: list[str] = []
        for e in self.history():
            if e["event"] == "promoted":
                stack.append(e["version"])
            elif e["event"] == "rolled_back":
                stack.pop()
        return stack

    # ---- đăng ký ----

    def versions(self) -> list[str]:
        return sorted(p.name for p in self.root.iterdir() if p.is_dir() and (p / MANIFEST).exists())

    def manifest(self, version: str) -> dict[str, Any]:
        path = self.root / version / MANIFEST
        if not path.exists():
            raise RegistryError(f"không có phiên bản {version} trong registry")
        return json.loads(path.read_text(encoding="utf-8"))

    def register(self, model: Any, meta: dict[str, Any], *, actor: str, parent: str | None = None,
                 holdout: dict[str, Any] | None = None, note: str = "") -> str:
        """Ghi mô hình vào thư mục tạm rồi đổi tên thành thư mục phiên bản. Phiên bản đã có thì từ chối."""
        with self._locked():
            staging = self.root / f".staging-{uuid.uuid4().hex[:8]}"
            try:
                full = save_model(model, meta, staging)
                version = full["model_version"]
                if (self.root / version).exists():
                    raise RegistryError(f"phiên bản {version} đã có; registry không ghi đè")
                if parent is not None and parent not in self.versions():
                    raise RegistryError(f"bản cha {parent} không có trong registry")
                manifest = {
                    "version": version, "registered_at": now_iso(), "registered_by": actor, "parent": parent,
                    "sha256": full["artifact"]["sha256"], "bytes": full["artifact"]["bytes"],
                    "feature_count": len(full["feature_list"]), "holdout": holdout, "note": note,
                    "libraries": full["libraries"],
                }
                _write_json_atomic(staging / MANIFEST, manifest)
                os.replace(staging, self.root / version)
            finally:
                shutil.rmtree(staging, ignore_errors=True)
            self._append({"event": "registered", "version": version, "actor": actor, "parent": parent})
            return version

    # ---- phục vụ ----

    def current(self) -> str | None:
        path = self.root / CURRENT
        return json.loads(path.read_text(encoding="utf-8"))["version"] if path.exists() else None

    def load(self, version: str) -> ScoringModel:
        """Nạp một phiên bản: model_io kiểm sha256, danh sách đặc trưng, phiên bản thư viện; registry kiểm thêm khớp manifest."""
        manifest = self.manifest(version)
        scoring = load_model(self.root / version)
        if scoring.meta["artifact"]["sha256"] != manifest["sha256"]:
            raise RegistryError(f"sha256 trong model_meta.json của {version} khác manifest")
        return scoring

    def load_current(self) -> ScoringModel:
        """Bản đang phục vụ. Đọc con trỏ mỗi lần gọi, nạp lại chỉ khi phiên bản đổi: rollback có hiệu lực ở lần chấm kế tiếp."""
        version = self.current()
        if version is None:
            raise RegistryError("chưa promote phiên bản nào")
        if version not in self._cache:
            self._cache[version] = self.load(version)
        return self._cache[version]

    # ---- promote và rollback ----

    def _gate(self, version: str) -> list[str]:
        """Lý do chặn promote (rỗng = qua cổng): thiếu số đo holdout, holdout chồng train, kém champion quá dung sai."""
        problems = []
        h = self.manifest(version).get("holdout")
        if not h:
            return ["chưa có số đo holdout trong manifest"]
        if h.get("overlaps_train"):
            problems.append("holdout chồng lên dữ liệu huấn luyện, số đo không đáng tin")
        cur = self.current()
        if cur is not None and cur != version:
            ch = self.manifest(cur).get("holdout")
            if ch and ch.get("days") != h.get("days"):
                problems.append(f"holdout của {cur} ({ch.get('days')}) khác ngày với holdout của {version} ({h.get('days')}): không so được")
            elif ch and h["pr_auc"] < ch["pr_auc"] - GATE_TOLERANCE:
                problems.append(f"PR-AUC holdout {h['pr_auc']:.4f} kém champion {cur} ({ch['pr_auc']:.4f}) quá {GATE_TOLERANCE}")
        return problems

    def promote(self, version: str, *, actor: str, reason: str, force: bool = False) -> None:
        if not reason.strip():
            raise RegistryError("promote phải có lý do")
        with self._locked():
            self._check_consistent()
            if self.current() == version:
                raise RegistryError(f"{version} đã là bản đang phục vụ")
            try:
                self.load(version)  # sha256 + đặc trưng + phiên bản thư viện, trước khi đổi con trỏ
            except ModelLoadError as e:
                raise RegistryError(f"không promote được {version}: {e}") from e
            problems = self._gate(version)
            if problems and not force:
                raise RegistryError(f"{version} không qua cổng: " + "; ".join(problems))
            previous = self.current()
            _write_json_atomic(self.root / CURRENT, {"version": version, "since": now_iso()})
            self._append({"event": "promoted", "version": version, "previous": previous, "actor": actor, "reason": reason,
                          "forced_over": problems if force and problems else None})

    def rollback(self, *, actor: str, reason: str) -> str:
        """Đưa con trỏ về bản đứng dưới bản hiện tại trong ngăn xếp promote. Không huấn luyện lại, không xóa gì."""
        if not reason.strip():
            raise RegistryError("rollback phải có lý do")
        with self._locked():
            self._check_consistent()
            stack = self._stack()
            if len(stack) < 2:
                raise RegistryError("không có bản trước để rollback")
            target, bad = stack[-2], stack[-1]
            try:
                self.load(target)
            except (ModelLoadError, RegistryError) as e:
                raise RegistryError(f"bản trước {target} không nạp được: {e}") from e
            _write_json_atomic(self.root / CURRENT, {"version": target, "since": now_iso()})
            self._append({"event": "rolled_back", "version": bad, "to": target, "actor": actor, "reason": reason})
            return target
