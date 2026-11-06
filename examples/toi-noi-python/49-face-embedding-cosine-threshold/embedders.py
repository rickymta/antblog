"""Hai bộ embedding chạy offline qua ONNX: ArcFace R50 (onnxruntime) và SFace (OpenCV).

Cả hai nhận crop BGR 112 x 112 đã căn chỉnh. Vector ra luôn được chuẩn hóa L2 để cosine similarity
chỉ còn là tích vô hướng. Ngưỡng của hai mô hình khác nhau hoàn toàn: không dùng chung được.
"""

from pathlib import Path

import cv2
import numpy as np
import onnxruntime as ort

from models import ARCFACE_R50, SFACE, ensure_model


def l2_normalize(x: np.ndarray) -> np.ndarray:
    return x / np.linalg.norm(x, axis=-1, keepdims=True)


ORT_THREADS = 4  # đặt cố định để số đo ít dao động khi máy còn nhiều việc khác (mặc định ORT dùng mọi lõi)
ort.set_default_logger_severity(3)  # chỉ lỗi: model khai báo batch 1 nên ORT cảnh báo mỗi lần chạy batch > 1


class ArcFaceR50:
    """ArcFace ResNet-50 huấn luyện trên WebFace600K (tên file w600k_r50), embedding 512 chiều."""

    name = "arcface_r50"
    dim = 512

    def __init__(self, providers: list[str] | None = None) -> None:
        path = ensure_model(ARCFACE_R50, verbose=False)
        # Mặc định CPU để số đo lặp lại được; máy có GPU/CoreML đổi qua providers (bài 52).
        options = ort.SessionOptions()
        options.intra_op_num_threads = ORT_THREADS
        self._session = ort.InferenceSession(str(path), options, providers=providers or ["CPUExecutionProvider"])
        self._input = self._session.get_inputs()[0].name

    def embed(self, aligned_bgr: list[np.ndarray]) -> np.ndarray:
        """Danh sách crop 112 x 112 BGR -> ma trận (N, 512) chuẩn hóa L2."""
        # Tiền xử lý của InsightFace: BGR -> RGB, (x - 127.5) / 127.5, NCHW.
        blob = cv2.dnn.blobFromImages(aligned_bgr, 1.0 / 127.5, (112, 112), (127.5, 127.5, 127.5), swapRB=True)
        (out,) = self._session.run(None, {self._input: blob})
        return l2_normalize(out.astype(np.float32))


class SFace:
    """SFace (OpenCV Zoo, 2021dec), embedding 128 chiều, gọi qua cv2.FaceRecognizerSF."""

    name = "sface"
    dim = 128

    def __init__(self) -> None:
        path: Path = ensure_model(SFACE, verbose=False)
        self._recognizer = cv2.FaceRecognizerSF.create(str(path), "")

    def embed(self, aligned_bgr: list[np.ndarray]) -> np.ndarray:
        vectors = [self._recognizer.feature(img)[0] for img in aligned_bgr]  # mỗi lần một ảnh
        return l2_normalize(np.asarray(vectors, dtype=np.float32))
