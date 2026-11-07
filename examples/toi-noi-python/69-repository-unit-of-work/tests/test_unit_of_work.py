"""Unit of Work trên PostgreSQL thật: commit, rollback, không lồng, tách theo ngữ cảnh."""

import asyncio
import contextvars
import threading

import pytest
from sqlalchemy import text
from sqlalchemy.engine import Engine

from kyc_service.application.errors import NestedUnitOfWork, UnitOfWorkNotStarted
from kyc_service.infrastructure.sql.unit_of_work import SqlAlchemyUnitOfWork, make_session_factory

from .test_repository import new_application

pytestmark = pytest.mark.usefixtures("clean")


def rows(engine: Engine) -> list[str]:
    with engine.connect() as conn:
        found = conn.execute(
            text("SELECT application_id FROM lab69.kyc_applications ORDER BY 1")
        ).scalars()
        return list(found)


def test_leaving_the_block_commits(engine: Engine) -> None:
    uow = SqlAlchemyUnitOfWork(make_session_factory(engine))
    with uow.begin():
        uow.applications.add(new_application())
    assert rows(engine) == ["KYC-0001"]


def test_an_exception_rolls_everything_back(engine: Engine) -> None:
    uow = SqlAlchemyUnitOfWork(make_session_factory(engine))
    with pytest.raises(RuntimeError), uow.begin():
        uow.applications.add(new_application("KYC-0001", "DR-001"))
        uow.applications.add(new_application("KYC-0002", "DR-002"))
        raise RuntimeError("lỗi giữa chừng")
    assert rows(engine) == []


def test_session_is_closed_and_connection_returned(engine: Engine) -> None:
    uow = SqlAlchemyUnitOfWork(make_session_factory(engine))
    with uow.begin():
        uow.applications.add(new_application())
    with pytest.raises(RuntimeError), uow.begin():
        raise RuntimeError("lỗi")
    assert engine.pool.checkedout() == 0  # type: ignore[attr-defined]
    with pytest.raises(UnitOfWorkNotStarted):
        _ = uow.session


def test_begin_cannot_be_nested(engine: Engine) -> None:
    uow = SqlAlchemyUnitOfWork(make_session_factory(engine))
    with uow.begin(), pytest.raises(NestedUnitOfWork), uow.begin():
        pass


def test_one_uow_object_gives_each_thread_its_own_session(engine: Engine) -> None:
    uow = SqlAlchemyUnitOfWork(make_session_factory(engine))
    both_inside = threading.Barrier(2, timeout=10)
    sessions: dict[str, int] = {}
    errors: list[BaseException] = []

    def worker(app_id: str, driver: str) -> None:
        try:
            with uow.begin():
                sessions[app_id] = id(uow.session)
                both_inside.wait()  # cả hai thread cùng đang trong transaction của mình
                uow.applications.add(new_application(app_id, driver))
        except BaseException as exc:
            errors.append(exc)

    threads = [
        threading.Thread(target=worker, args=("KYC-0001", "DR-001")),
        threading.Thread(target=worker, args=("KYC-0002", "DR-002")),
    ]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    assert errors == []
    assert sessions["KYC-0001"] != sessions["KYC-0002"]
    assert rows(engine) == ["KYC-0001", "KYC-0002"]


def test_task_created_inside_begin_shares_the_session_but_a_bare_thread_does_not(
    engine: Engine,
) -> None:
    uow = SqlAlchemyUnitOfWork(make_session_factory(engine))

    async def inner() -> int:
        return id(uow.session)

    async def scenario() -> tuple[int, int, int]:
        with uow.begin():
            outer = id(uow.session)
            return (
                outer,
                await asyncio.create_task(inner()),  # chép Context: cùng Session
                await asyncio.to_thread(lambda: id(uow.session)),  # cũng chép Context
            )

    outer, via_task, via_thread_pool = asyncio.run(scenario())
    assert outer == via_task == via_thread_pool
    seen: list[object] = []

    def bare_thread() -> None:
        try:
            seen.append(uow.session)
        except UnitOfWorkNotStarted as exc:
            seen.append(type(exc).__name__)

    with uow.begin():
        t = threading.Thread(target=bare_thread)  # Thread mới không thừa kế Context
        t.start()
        t.join()
    assert seen == ["UnitOfWorkNotStarted"]


def test_leaving_begin_in_another_context_fails_but_still_closes_the_session(
    engine: Engine,
) -> None:
    uow = SqlAlchemyUnitOfWork(make_session_factory(engine))
    entered, leaving = contextvars.copy_context(), contextvars.copy_context()
    block = uow.begin()
    entered.run(block.__enter__)
    with pytest.raises(ValueError, match="different Context"):
        leaving.run(block.__exit__, None, None, None)
    assert engine.pool.checkedout() == 0  # type: ignore[attr-defined]
