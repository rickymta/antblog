"""Test mediator thuần: dispatch theo kiểu và thứ tự pipeline, không cần domain KYC."""

from dataclasses import dataclass
from typing import Any

import pytest

from kyc_service.application.mediator import (
    Behavior,
    DuplicateHandler,
    HandlerNotFound,
    Mediator,
    Next,
)
from kyc_service.application.messages import Command, Message, Query


@dataclass(frozen=True)
class Ping(Query[str]):
    text: str


@dataclass(frozen=True)
class LoudPing(Ping):  # kiểu con của Ping, KHÔNG có handler riêng
    pass


@dataclass(frozen=True)
class Bump(Command[int]):
    by: int


class PingHandler:
    def handle(self, message: Ping) -> str:
        return f"pong:{message.text}"


class BumpHandler:
    def __init__(self, trace: list[str]) -> None:
        self._trace = trace

    def handle(self, message: Bump) -> int:
        self._trace.append("handler")
        return message.by + 1


def recorder(name: str, trace: list[str]) -> Behavior:
    def behavior(message: Message[Any], next_step: Next) -> Any:
        trace.append(f"{name}:vào")
        try:
            return next_step(message)
        finally:
            trace.append(f"{name}:ra")

    return behavior


def test_dispatch_by_exact_type() -> None:
    mediator = Mediator()
    mediator.register(Ping, PingHandler())
    assert mediator.send(Ping("a")) == "pong:a"
    with pytest.raises(HandlerNotFound, match="LoudPing"):
        mediator.send(LoudPing("a"))  # không rơi về handler của lớp cha


def test_unregistered_and_duplicate() -> None:
    mediator = Mediator()
    with pytest.raises(HandlerNotFound):
        mediator.send(Ping("a"))
    mediator.register(Ping, PingHandler())
    with pytest.raises(DuplicateHandler):
        mediator.register(Ping, PingHandler())


def test_first_behavior_wraps_outermost() -> None:
    trace: list[str] = []
    mediator = Mediator([recorder("A", trace), recorder("B", trace), recorder("C", trace)])
    mediator.register(Bump, BumpHandler(trace))
    assert mediator.send(Bump(1)) == 2
    assert trace == ["A:vào", "B:vào", "C:vào", "handler", "C:ra", "B:ra", "A:ra"]


def test_behavior_can_short_circuit() -> None:
    trace: list[str] = []

    def block(message: Message[Any], next_step: Next) -> Any:
        return -1  # không gọi next: handler không chạy

    mediator = Mediator([recorder("A", trace), block])
    mediator.register(Bump, BumpHandler(trace))
    assert mediator.send(Bump(1)) == -1
    assert trace == ["A:vào", "A:ra"]


def test_each_layer_keeps_its_own_next() -> None:
    # Bắt lỗi kinh điển: viết lambda trong vòng for làm mọi lớp gọi cùng một `next`.
    seen: list[str] = []

    def tag(name: str) -> Behavior:
        def behavior(message: Message[Any], next_step: Next) -> Any:
            seen.append(name)
            return next_step(message)

        return behavior

    mediator = Mediator([tag("1"), tag("2"), tag("3")])
    mediator.register(Ping, PingHandler())
    mediator.send(Ping("x"))
    assert seen == ["1", "2", "3"]
