"""Unit of Work: một Session và một transaction cho mỗi command.

`begin()` mở Session mới, mở transaction (`session.begin()`) và đặt (Session, kho) vào một
`ContextVar`. Phạm vi của nó là **Context**, không phải thread hay task:

- Mỗi request đồng thời có Context riêng (FastAPI chạy endpoint đồng bộ trong thread pool, mỗi lần
  gọi trên một bản sao Context), nên mỗi request có Session riêng dù dùng chung một đối tượng UoW.
- Task tạo BÊN TRONG `begin()` (`asyncio.create_task`, `asyncio.to_thread`) chép Context nên DÙNG
  CHUNG Session đó; Session đồng bộ không an toàn khi chia cho thread khác.
- `threading.Thread` tự mở bên trong `begin()` không thừa kế Context: gọi `uow.applications` ở đó
  ném `UnitOfWorkNotStarted`.
- Vào `begin()` ở Context này mà thoát ở Context khác là `ValueError` (token của ContextVar).

Thoát khối bình thường là commit; có exception là rollback.
"""

from collections.abc import Iterator
from contextlib import contextmanager
from contextvars import ContextVar

from sqlalchemy.engine import Engine
from sqlalchemy.orm import Session, sessionmaker

from kyc_service.application.errors import NestedUnitOfWork, UnitOfWorkNotStarted
from kyc_service.application.ports import KycRepository
from kyc_service.infrastructure.sql.repository import SqlAlchemyKycRepository


def make_session_factory(engine: Engine) -> sessionmaker[Session]:
    # expire_on_commit=False: không có dữ liệu hết hạn sau commit (ta không dùng lại hàng ORM).
    return sessionmaker(engine, expire_on_commit=False)


class SqlAlchemyUnitOfWork:
    def __init__(self, session_factory: sessionmaker[Session]) -> None:
        self._factory = session_factory
        # (Session, kho) của transaction đang mở trong Context này; None nếu chưa mở. Tài liệu
        # contextvars khuyên tạo ContextVar ở cấp module, vì biến không bị giải phóng cùng đối tượng
        # chứa nó. Lab giữ một biến cho MỖI đối tượng UoW để hai UoW (hai cơ sở dữ liệu, hoặc hai
        # UoW trong một test) không dùng chung giá trị; ứng dụng chỉ tạo một UoW lúc khởi động.
        self._current: ContextVar[tuple[Session, SqlAlchemyKycRepository] | None] = ContextVar(
            "kyc_uow_current", default=None
        )

    def _open(self) -> tuple[Session, SqlAlchemyKycRepository]:
        current = self._current.get()
        if current is None:
            raise UnitOfWorkNotStarted("dùng kho ngoài khối `with uow.begin()`")
        return current

    @property
    def session(self) -> Session:
        return self._open()[0]

    @property
    def applications(self) -> KycRepository:
        return self._open()[1]

    @contextmanager
    def begin(self) -> Iterator[None]:
        if self._current.get() is not None:
            raise NestedUnitOfWork("begin() lồng nhau: mỗi command chỉ có một transaction")
        session = self._factory()
        token = self._current.set((session, SqlAlchemyKycRepository(session)))
        try:
            with session.begin():  # commit khi thoát bình thường, rollback khi có exception
                yield
        finally:
            try:
                self._current.reset(token)  # ValueError nếu thoát ở Context khác với lúc vào
            finally:
                session.close()  # dù reset lỗi, Session vẫn phải đóng để trả kết nối
