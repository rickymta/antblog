"""Embedding cho bài 51 và dữ liệu phụ.

PHƯƠNG ÁN EMBEDDING: lab này tự chứa, không phụ thuộc lab 49. Embedding là PCA ("eigenfaces") 128 chiều
fit trên 200 ảnh Olivetti (ảnh 0-4 của mỗi người), rồi chuẩn hóa L2. Đây KHÔNG phải mô hình nhận diện
khuôn mặt hiện đại (InsightFace/ArcFace cho vector 512 chiều tách người tốt hơn nhiều). Nó đủ để có vector
có cấu trúc thật cho bài toán tìm kiếm; độ chính xác nhận diện trong bài là của PCA, không phải của hệ thống thật.

Distractor (tài khoản nền) là vector TỔNG HỢP lấy mẫu từ Gaussian có cùng trung bình/hiệp phương sai với
embedding thật, chỉ để đủ quy mô đo thời gian và recall của chỉ mục. Chúng không đại diện cho người thật.
"""

import numpy as np
from sklearn.datasets import fetch_olivetti_faces
from sklearn.decomposition import PCA

DIM = 128


def load_embeddings() -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    """Trả về (gallery, gallery_person, query, query_person); vector float32 đã chuẩn hóa L2.

    Olivetti (~1,3 MB, tải lần đầu vào ~/scikit_learn_data): 40 người x 10 ảnh 64 x 64. Ảnh 0-4 mỗi người là
    5 "tài khoản" đã đăng ký, ảnh 5-9 là ảnh của lần đăng ký mới cần kiểm tra trùng.
    """
    data = fetch_olivetti_faces(shuffle=False)
    x = data.data.astype(np.float32)
    person = data.target
    in_gallery = np.arange(len(x)) % 10 < 5
    pca = PCA(n_components=DIM, random_state=0).fit(x[in_gallery])
    emb = pca.transform(x).astype(np.float32)
    emb /= np.linalg.norm(emb, axis=1, keepdims=True)
    return emb[in_gallery], person[in_gallery], emb[~in_gallery], person[~in_gallery]


def synthetic_distractors(real: np.ndarray, n: int, seed: int = 51) -> np.ndarray:
    """n vector nền: Gaussian cùng trung bình và hiệp phương sai với `real`, chuẩn hóa L2."""
    rng = np.random.default_rng(seed)
    mean = real.mean(axis=0)
    values, vectors = np.linalg.eigh(np.cov(real.T))
    root = vectors * np.sqrt(np.clip(values, 0, None))
    z = mean + rng.standard_normal((n, real.shape[1])).astype(np.float32) @ root.T.astype(np.float32)
    z = z.astype(np.float32)
    z /= np.linalg.norm(z, axis=1, keepdims=True)
    return z
