from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class EmbedRequest(_message.Message):
    __slots__ = ("image",)
    IMAGE_FIELD_NUMBER: _ClassVar[int]
    image: bytes
    def __init__(self, image: _Optional[bytes] = ...) -> None: ...

class EmbedResponse(_message.Message):
    __slots__ = ("embedding", "model_version")
    EMBEDDING_FIELD_NUMBER: _ClassVar[int]
    MODEL_VERSION_FIELD_NUMBER: _ClassVar[int]
    embedding: _containers.RepeatedScalarFieldContainer[float]
    model_version: str
    def __init__(self, embedding: _Optional[_Iterable[float]] = ..., model_version: _Optional[str] = ...) -> None: ...

class CompareRequest(_message.Message):
    __slots__ = ("image_a", "image_b")
    IMAGE_A_FIELD_NUMBER: _ClassVar[int]
    IMAGE_B_FIELD_NUMBER: _ClassVar[int]
    image_a: bytes
    image_b: bytes
    def __init__(self, image_a: _Optional[bytes] = ..., image_b: _Optional[bytes] = ...) -> None: ...

class CompareResponse(_message.Message):
    __slots__ = ("score", "model_version")
    SCORE_FIELD_NUMBER: _ClassVar[int]
    MODEL_VERSION_FIELD_NUMBER: _ClassVar[int]
    score: float
    model_version: str
    def __init__(self, score: _Optional[float] = ..., model_version: _Optional[str] = ...) -> None: ...
