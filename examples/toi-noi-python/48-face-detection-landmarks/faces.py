"""Phát hiện khuôn mặt, landmark, căn chỉnh và chỉ số chất lượng ảnh (code trích trong bài 48).

Ba bộ phát hiện cùng giao diện `detect(bgr) -> list[Detection]`:
  - HaarDetector: Haar cascade có sẵn trong gói OpenCV, chạy offline, chỉ cho hộp.
  - YuNetDetector: mạng nhỏ qua OpenCV DNN (`cv2.FaceDetectorYN`), hộp + 5 landmark.
  - BlazeFaceDetector: MediaPipe Tasks `FaceDetector` (short range hoặc full range), hộp + 6 keypoint
    (không dùng để căn chỉnh).
Ngoài ra MediaPipeLandmarker (FaceLandmarker 478 điểm) rút 5 điểm để so với YuNet.

Quy ước toạ độ: ảnh BGR uint8 như OpenCV, hộp (x, y, w, h) theo điểm ảnh, 5 landmark theo thứ tự
mắt phải, mắt trái, đỉnh mũi, khoé miệng phải, khoé miệng trái, với «phải/trái» là của người trong ảnh
(mắt phải của người nằm bên TRÁI khung hình).
"""

import math
from dataclasses import dataclass, field
from typing import Protocol

import cv2
import numpy as np

from models import BLAZEFACE, FACE_LANDMARKER, YUNET, ModelFile, ensure_model

# Mẫu 5 điểm chuẩn của ArcFace/InsightFace cho ảnh 112 x 112 (cũng là mẫu của OpenCV SFace).
ARCFACE_112 = np.array(
    [[38.2946, 51.6963], [73.5318, 51.5014], [56.0252, 71.7366], [41.5493, 92.3655], [70.7299, 92.2041]],
    dtype=np.float64,
)

# Ngưỡng chất lượng: số đo thật nằm trong bài; đổi chúng là đổi tỉ lệ ảnh đưa sang người duyệt.
MIN_FACE_PX = 80  # cạnh ngắn của hộp mặt
MAX_ROLL_DEG = 15.0  # YuNet báo roll tối đa cỡ 22 độ dù ảnh nghiêng hơn (xem bài), nên đặt thấp
MIN_BLUR_VAR = 60.0  # phương sai Laplacian trên crop căn chỉnh 112 x 112
MIN_BRIGHTNESS = 60.0
MAX_BRIGHTNESS = 200.0


@dataclass(frozen=True)
class Detection:
    box: tuple[int, int, int, int]  # x, y, w, h
    score: float
    landmarks: np.ndarray | None = field(default=None, compare=False)  # (5, 2) float, hoặc None


class Detector(Protocol):
    name: str

    def detect(self, bgr: np.ndarray) -> list[Detection]: ...


class HaarDetector:
    name = "haar"

    def __init__(self, min_size: int = 40) -> None:
        path = cv2.data.haarcascades + "haarcascade_frontalface_default.xml"
        self._cascade = cv2.CascadeClassifier(path)
        if self._cascade.empty():
            raise RuntimeError(f"không nạp được {path}")
        self._min_size = (min_size, min_size)

    def detect(self, bgr: np.ndarray) -> list[Detection]:
        gray = cv2.cvtColor(bgr, cv2.COLOR_BGR2GRAY)
        boxes = self._cascade.detectMultiScale(gray, scaleFactor=1.1, minNeighbors=5, minSize=self._min_size)
        # Haar không có điểm tin cậy; để 1.0 cho đủ giao diện.
        return [Detection((int(x), int(y), int(w), int(h)), 1.0) for x, y, w, h in boxes]


class YuNetDetector:
    name = "yunet"

    def __init__(self, score_threshold: float = 0.7) -> None:
        model = ensure_model(YUNET, verbose=False)
        # Kích thước đầu vào đặt lại theo từng ảnh trước khi detect.
        self._net = cv2.FaceDetectorYN.create(str(model), "", (320, 320), score_threshold, 0.3, 5000)

    def detect(self, bgr: np.ndarray) -> list[Detection]:
        height, width = bgr.shape[:2]
        self._net.setInputSize((width, height))
        _, rows = self._net.detect(bgr)
        if rows is None:
            return []
        found = []
        for row in rows:
            x, y, w, h = (int(round(v)) for v in row[:4])
            found.append(Detection((x, y, w, h), float(row[14]), row[4:14].reshape(5, 2).astype(np.float64)))
        return found


def _mp_image(bgr: np.ndarray):
    import mediapipe as mp  # nhập chậm, chỉ khi cần

    return mp.Image(image_format=mp.ImageFormat.SRGB, data=cv2.cvtColor(bgr, cv2.COLOR_BGR2RGB))


class BlazeFaceDetector:
    """BlazeFace của MediaPipe: bản short range (mặt lớn, cầm gần) hoặc full range (mặt nhỏ hơn, xa hơn)."""

    def __init__(self, model: ModelFile = BLAZEFACE, name: str = "blazeface", min_confidence: float = 0.5) -> None:
        self.name = name
        from mediapipe.tasks import python as mp_python
        from mediapipe.tasks.python import vision

        options = vision.FaceDetectorOptions(
            base_options=mp_python.BaseOptions(model_asset_path=str(ensure_model(model, verbose=False))),
            min_detection_confidence=min_confidence,
        )
        self._detector = vision.FaceDetector.create_from_options(options)

    def detect(self, bgr: np.ndarray) -> list[Detection]:
        result = self._detector.detect(_mp_image(bgr))
        return [
            Detection(
                (d.bounding_box.origin_x, d.bounding_box.origin_y, d.bounding_box.width, d.bounding_box.height),
                float(d.categories[0].score),
            )
            for d in result.detections
        ]


# Chỉ số điểm trong lưới 478 điểm của MediaPipe: tâm mống mắt (refined) và khoé miệng.
_MP_RIGHT_IRIS = range(468, 473)  # mắt phải của người trong ảnh, nằm bên trái khung hình
_MP_LEFT_IRIS = range(473, 478)
_MP_NOSE_TIP, _MP_MOUTH_RIGHT, _MP_MOUTH_LEFT = 1, 61, 291


class MediaPipeLandmarker:
    name = "mp-landmarker"

    def __init__(self) -> None:
        from mediapipe.tasks import python as mp_python
        from mediapipe.tasks.python import vision

        options = vision.FaceLandmarkerOptions(
            base_options=mp_python.BaseOptions(model_asset_path=str(ensure_model(FACE_LANDMARKER, verbose=False))),
            num_faces=2,
        )
        self._landmarker = vision.FaceLandmarker.create_from_options(options)

    def detect(self, bgr: np.ndarray) -> list[Detection]:
        height, width = bgr.shape[:2]
        found = []
        for mesh in self._landmarker.detect(_mp_image(bgr)).face_landmarks:
            pts = np.array([[p.x * width, p.y * height] for p in mesh], dtype=np.float64)  # (478, 2)
            five = np.array(
                [
                    pts[list(_MP_RIGHT_IRIS)].mean(axis=0),
                    pts[list(_MP_LEFT_IRIS)].mean(axis=0),
                    pts[_MP_NOSE_TIP],
                    pts[_MP_MOUTH_RIGHT],
                    pts[_MP_MOUTH_LEFT],
                ]
            )
            x0, y0 = pts.min(axis=0)
            x1, y1 = pts.max(axis=0)
            found.append(Detection((int(x0), int(y0), int(x1 - x0), int(y1 - y0)), 1.0, five))
        return found


def iou(a: tuple[int, int, int, int], b: tuple[int, int, int, int]) -> float:
    """Tỉ lệ giao trên hợp của hai hộp (x, y, w, h)."""
    ax, ay, aw, ah = a
    bx, by, bw, bh = b
    inter_w = max(0, min(ax + aw, bx + bw) - max(ax, bx))
    inter_h = max(0, min(ay + ah, by + bh) - max(ay, by))
    inter = inter_w * inter_h
    return inter / (aw * ah + bw * bh - inter)


def roll_degrees(landmarks: np.ndarray) -> float:
    """Góc nghiêng của đường nối hai mắt so với phương ngang, độ; dương = mắt trái thấp hơn mắt phải."""
    dx, dy = landmarks[1] - landmarks[0]
    return math.degrees(math.atan2(dy, dx))


def similarity_transform(src: np.ndarray, dst: np.ndarray) -> np.ndarray:
    """Phép đồng dạng (xoay + tỉ lệ đều + tịnh tiến) bình phương tối thiểu từ src sang dst, ma trận 2 x 3.

    Umeyama 1991: không có méo theo chiều nào, nên khuôn mặt không bị bóp dẹt khi căn chỉnh.
    """
    src_mean, dst_mean = src.mean(axis=0), dst.mean(axis=0)
    src_c, dst_c = src - src_mean, dst - dst_mean
    u, s, vt = np.linalg.svd(dst_c.T @ src_c / len(src))
    flip = np.eye(2)
    if np.linalg.det(u) * np.linalg.det(vt) < 0:  # loại phép phản chiếu
        flip[1, 1] = -1
    rotation = u @ flip @ vt
    scale = (s * np.diag(flip)).sum() / src_c.var(axis=0).sum()
    translation = dst_mean - scale * rotation @ src_mean
    return np.hstack([scale * rotation, translation[:, None]])


def align_face(bgr: np.ndarray, landmarks: np.ndarray, size: int = 112) -> np.ndarray:
    """Cắt và xoay khuôn mặt về mẫu ArcFace: hai mắt nằm ngang, cách đều, mặt chiếm trọn khung size x size."""
    matrix = similarity_transform(landmarks.astype(np.float64), ARCFACE_112 * size / 112)
    return cv2.warpAffine(bgr, matrix, (size, size), flags=cv2.INTER_LINEAR, borderMode=cv2.BORDER_REPLICATE)


@dataclass(frozen=True)
class Quality:
    blur: float  # phương sai Laplacian: càng nhỏ càng mờ
    brightness: float  # trung bình mức xám 0..255
    contrast: float  # độ lệch chuẩn mức xám


# Vùng «lõi» mặt trên crop 112 x 112 (từ ngang lông mày đến dưới miệng, giữa hai má). Đo cả crop thì cạnh
# viền thẻ, chữ trên thẻ hoặc mép nền lọt vào làm phương sai Laplacian tăng vọt dù mặt vẫn mờ.
FACE_CORE = (slice(36, 100), slice(26, 86))


def face_quality(aligned_bgr: np.ndarray, *, core: bool = True) -> Quality:
    """Chỉ số chất lượng đo trên vùng lõi của crop căn chỉnh 112 x 112, để mọi ảnh cùng kích thước thì mới so được.

    core=False đo cả crop (chỉ để so sánh trong bài: cách đo này bị cạnh viền thẻ làm sai lệch).
    """
    gray = cv2.cvtColor(aligned_bgr, cv2.COLOR_BGR2GRAY)
    if core:
        gray = gray[FACE_CORE]
    return Quality(
        blur=float(cv2.Laplacian(gray, cv2.CV_64F).var()),
        brightness=float(gray.mean()),
        contrast=float(gray.std()),
    )


@dataclass(frozen=True)
class FaceCheck:
    """Kết quả kiểm ảnh chân dung: là đầu vào cho người duyệt, không phải quyết định xác minh."""

    status: str  # "face_checked" hoặc "needs_review" theo KycStatus của loạt bài
    reasons: tuple[str, ...]
    detection: Detection | None
    quality: Quality | None
    roll: float | None


def to_bgr(image: np.ndarray) -> np.ndarray:
    """Chấp nhận ảnh xám (H, W), BGR (H, W, 3) hoặc BGRA (H, W, 4); ảnh rỗng hoặc dtype lạ thì ném ValueError."""
    if image.size == 0 or image.dtype != np.uint8 or image.ndim not in (2, 3):
        raise ValueError(f"cần ảnh uint8 không rỗng, nhận shape {image.shape} dtype {image.dtype}")
    if image.ndim == 2:
        return cv2.cvtColor(image, cv2.COLOR_GRAY2BGR)
    if image.shape[2] == 4:
        return cv2.cvtColor(image, cv2.COLOR_BGRA2BGR)
    if image.shape[2] != 3:
        raise ValueError(f"cần 1, 3 hoặc 4 kênh, nhận {image.shape[2]}")
    return image


def check_portrait(image: np.ndarray, detector: Detector) -> FaceCheck:
    """Một ảnh có đúng một mặt đủ lớn, nằm trọn trong khung, thẳng, rõ và đủ sáng thì mới đi tiếp.

    Còn lại đưa sang người duyệt kèm lý do; hàm không bao giờ tự kết luận «giả mạo» hay «sai người».
    """
    bgr = to_bgr(image)
    height, width = bgr.shape[:2]
    faces = detector.detect(bgr)
    reasons: list[str] = []
    if not faces:
        return FaceCheck("needs_review", ("no_face",), None, None, None)
    if len(faces) > 1:
        reasons.append("multiple_faces")
    face = max(faces, key=lambda f: f.box[2] * f.box[3])  # lấy mặt lớn nhất để đo tiếp
    x, y, w, h = face.box
    if min(w, h) < MIN_FACE_PX:
        reasons.append("face_too_small")
    if x < 0 or y < 0 or x + w > width or y + h > height:  # YuNet vẫn trả hộp thò ra ngoài khung
        reasons.append("face_cut_off")
    if face.landmarks is None:
        return FaceCheck("needs_review", (*reasons, "no_landmarks"), face, None, None)
    roll = roll_degrees(face.landmarks)
    if abs(roll) > MAX_ROLL_DEG:
        reasons.append("tilted")
    quality = face_quality(align_face(bgr, face.landmarks))
    if quality.blur < MIN_BLUR_VAR:
        reasons.append("blurry")
    if quality.brightness < MIN_BRIGHTNESS:
        reasons.append("too_dark")
    if quality.brightness > MAX_BRIGHTNESS:
        reasons.append("too_bright")
    status = "needs_review" if reasons else "face_checked"
    return FaceCheck(status, tuple(reasons), face, quality, roll)
