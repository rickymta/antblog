from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class ScoreRequest(_message.Message):
    __slots__ = ("trip_id", "customer_id", "driver_id", "device_id", "distance_km")
    TRIP_ID_FIELD_NUMBER: _ClassVar[int]
    CUSTOMER_ID_FIELD_NUMBER: _ClassVar[int]
    DRIVER_ID_FIELD_NUMBER: _ClassVar[int]
    DEVICE_ID_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_KM_FIELD_NUMBER: _ClassVar[int]
    trip_id: str
    customer_id: str
    driver_id: str
    device_id: str
    distance_km: str
    def __init__(self, trip_id: _Optional[str] = ..., customer_id: _Optional[str] = ..., driver_id: _Optional[str] = ..., device_id: _Optional[str] = ..., distance_km: _Optional[str] = ...) -> None: ...

class ScoreResponse(_message.Message):
    __slots__ = ("score", "reasons", "model_version")
    SCORE_FIELD_NUMBER: _ClassVar[int]
    REASONS_FIELD_NUMBER: _ClassVar[int]
    MODEL_VERSION_FIELD_NUMBER: _ClassVar[int]
    score: float
    reasons: _containers.RepeatedScalarFieldContainer[str]
    model_version: str
    def __init__(self, score: _Optional[float] = ..., reasons: _Optional[_Iterable[str]] = ..., model_version: _Optional[str] = ...) -> None: ...
