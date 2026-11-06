"""Rule engine chấm điểm rủi ro: mỗi rule là một hàm `(features) -> RuleHit | None`, cấu hình bằng YAML.

Điểm đầu ra là ĐẦU VÀO cho người duyệt (dải "review" = đưa vào hàng đợi), không phải quyết định khóa tài khoản:
engine không có dải nào tự động chặn.

Ba lớp, theo Strategy của bài 19 (hàm là strategy):
  1. Rule      : hàm nhận đặc trưng của MỘT chuyến, trả RuleHit (điểm + lý do) hoặc None.
  2. Aggregator: hàm gộp danh sách RuleHit thành một điểm 0..100 (tổng bị chặn, lớn nhất, noisy-or).
  3. RuleEngine: chạy mọi rule, gộp, gắn dải và giữ lại lý do.
"""

import math
import operator
import re
import string
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from functools import reduce
from pathlib import Path

import polars as pl
import yaml

from features import FEATURES

type Features = Mapping[str, float | None]

OPS: dict[str, Callable[[float, float], bool]] = {
    ">": operator.gt, ">=": operator.ge, "<": operator.lt, "<=": operator.le, "==": operator.eq, "!=": operator.ne,
}
CONDITION_RE = re.compile(r"^\s*([a-z][a-z0-9_]*)\s*(>=|<=|==|!=|>|<)\s*(-?\d+(?:\.\d+)?)\s*$")
RULE_ID_RE = re.compile(r"^[a-z][a-z0-9_]*$")
RULE_KEYS = {"id", "points", "when", "reason"}
TOP_KEYS = {"version", "review_at", "watch_at", "aggregate", "rules"}


# ---- Kiểu dữ liệu ---------------------------------------------------------------------------


@dataclass(frozen=True, slots=True)
class RuleHit:
    """Một rule đã bắn: tên, điểm của rule và câu giải thích đã điền số thật."""

    rule_id: str
    points: int
    reason: str


type Rule = Callable[[Features], RuleHit | None]
type Aggregator = Callable[[Sequence[RuleHit]], float]


@dataclass(frozen=True, slots=True)
class Verdict:
    score: float  # 0..100
    band: str  # "none" | "watch" | "review"; không có dải tự khóa
    hits: tuple[RuleHit, ...]

    @property
    def reasons(self) -> list[str]:
        """Lý do xếp theo điểm giảm dần, kèm tên rule để người duyệt và kỹ sư cùng tra được."""
        return [f"[{h.rule_id}] {h.reason} (rule {h.points})" for h in sorted(self.hits, key=lambda h: -h.points)]


# ---- Aggregator: ba strategy gộp điểm -------------------------------------------------------


def sum_capped(hits: Sequence[RuleHit]) -> float:
    """Cộng điểm các rule, chặn ở 100. Hai rule cùng nhìn một dấu vết bị đếm hai lần."""
    return float(min(100, sum(h.points for h in hits)))


def max_points(hits: Sequence[RuleHit]) -> float:
    """Chỉ lấy rule mạnh nhất; rule bổ sung không làm điểm tăng."""
    return float(max((h.points for h in hits), default=0))


def noisy_or(hits: Sequence[RuleHit]) -> float:
    """Coi điểm rule / 100 là xác suất, các rule độc lập: 1 - tích(1 - p). Giả định độc lập sai khi hai rule nhìn cùng một
    dấu vết: noisy-or vẫn tính hai lần, chỉ chậm hơn `sum` và không vượt 100."""
    return 100.0 * (1.0 - math.prod(1.0 - h.points / 100.0 for h in hits))


AGGREGATORS: dict[str, Aggregator] = {"sum": sum_capped, "max": max_points, "noisy_or": noisy_or}


# ---- Cấu hình -------------------------------------------------------------------------------


class ConfigError(ValueError):
    """Cấu hình sai; `errors` liệt kê MỌI lỗi một lượt chứ không dừng ở lỗi đầu."""

    def __init__(self, errors: Sequence[str]) -> None:
        super().__init__("; ".join(errors))
        self.errors = list(errors)


@dataclass(frozen=True, slots=True)
class Condition:
    feature: str
    op: str
    value: float

    def holds(self, x: float | None) -> bool:
        """None và NaN nghĩa là «không biết»: điều kiện không thỏa, rule không bắn trên dữ liệu thiếu.

        (NaN phải loại riêng: `nan != 1` là True, và polars xếp NaN lớn hơn mọi số.)"""
        return x is not None and not math.isnan(x) and OPS[self.op](x, self.value)


@dataclass(frozen=True, slots=True)
class RuleSpec:
    id: str
    points: int
    when: tuple[Condition, ...]
    reason: str


@dataclass(frozen=True, slots=True)
class EngineConfig:
    version: int
    review_at: float
    watch_at: float
    aggregate: str
    rules: tuple[RuleSpec, ...]


def _parse_condition(text: object, where: str, errors: list[str]) -> Condition | None:
    m = CONDITION_RE.match(text) if isinstance(text, str) else None
    if m is None:
        errors.append(f"{where}: điều kiện {text!r} không có dạng «tên_đặc_trưng toán_tử số»")
        return None
    feature, op, value = m.group(1), m.group(2), float(m.group(3))
    if feature not in FEATURES:
        errors.append(f"{where}: đặc trưng {feature!r} không có trong FEATURES")
        return None
    return Condition(feature, op, value)


def _check_reason(reason: object, feats: set[str], where: str, errors: list[str]) -> None:
    if not isinstance(reason, str) or not reason.strip():
        errors.append(f"{where}: reason phải là chuỗi không rỗng")
        return
    try:
        parsed = list(string.Formatter().parse(reason))
    except ValueError as e:
        errors.append(f"{where}: reason sai cú pháp ({e})")
        return
    fields = [name for _, name, _, _ in parsed if name is not None]
    if any(spec and ("{" in spec or "}" in spec) for _, _, spec, _ in parsed):
        errors.append(f"{where}: reason không được lồng trường trong định dạng (ví dụ {{x:{{y}}}})")
        return
    bad = [name for name in fields if name not in feats]
    for name in bad:
        # chỉ cho tên đặc trưng trần, và chỉ đặc trưng nằm trong `when` (đảm bảo luôn có giá trị lúc điền)
        errors.append(f"{where}: reason dùng {{{name}}} nhưng đó không phải đặc trưng trong when")
    if not bad:
        try:
            reason.format_map({f: 1.0 for f in FEATURES})
        except (ValueError, TypeError, KeyError, IndexError, AttributeError) as e:
            errors.append(f"{where}: reason có định dạng không điền được ({type(e).__name__}: {e})")


class _StrictLoader(yaml.SafeLoader):
    """SafeLoader (không dựng được đối tượng Python tùy ý) thêm một việc: từ chối khóa trùng.

    `yaml.safe_load` âm thầm lấy khóa cuối cùng khi một mapping có hai khóa trùng tên."""


def _construct_mapping(loader: _StrictLoader, node: yaml.MappingNode, deep: bool = False):
    seen: set = set()
    for key_node, _ in node.value:
        key = loader.construct_object(key_node, deep=deep)
        try:
            hash(key)
        except TypeError:
            raise yaml.constructor.ConstructorError(None, None, "khóa không hashable", key_node.start_mark) from None
        if key in seen:
            raise yaml.constructor.ConstructorError(None, None, f"khóa trùng {key!r}", key_node.start_mark)
        seen.add(key)
    return yaml.SafeLoader.construct_mapping(loader, node, deep)


_StrictLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, _construct_mapping)
SUPPORTED_VERSION = 1


def parse_config(text: str) -> EngineConfig:
    """YAML -> EngineConfig đã kiểm hợp lệ; sai thì ConfigError liệt kê các lỗi tìm được (không bao giờ ngoại lệ lạ).

    Đọc bằng lớp con của `SafeLoader` (như `yaml.safe_load`, thêm kiểm khóa trùng): thẻ `!!python/object` bị từ chối.
    `yaml.load` không có Loader ném TypeError, `FullLoader` từ chối thẻ đó, chỉ `Loader` và `UnsafeLoader` chạy mã."""
    try:
        return _parse(text)
    except ConfigError:
        raise
    except (TypeError, KeyError, ValueError, AttributeError, IndexError) as e:  # lưới an toàn cuối cùng
        raise ConfigError([f"cấu hình sai cấu trúc ({type(e).__name__}: {e})"]) from e


def _parse(text: str) -> EngineConfig:
    errors: list[str] = []
    try:
        raw = yaml.load(text, Loader=_StrictLoader)  # noqa: S506 - _StrictLoader là lớp con của SafeLoader
    except yaml.YAMLError as e:
        raise ConfigError([f"YAML không đọc được: {type(e).__name__}: {str(e).splitlines()[0] if str(e) else ''}"]) from e
    if not isinstance(raw, dict):
        raise ConfigError(["gốc của file phải là một mapping"])
    for key in sorted(set(raw) - TOP_KEYS, key=str):
        errors.append(f"khóa không biết ở gốc: {key!r}")
    aggregate = raw.get("aggregate")
    if not isinstance(aggregate, str) or aggregate not in AGGREGATORS:
        errors.append(f"aggregate {aggregate!r} không thuộc {sorted(AGGREGATORS)}")
    scores: dict[str, float] = {}
    for key in ("review_at", "watch_at"):
        v = raw.get(key)
        if isinstance(v, bool) or not isinstance(v, (int, float)) or not 0 < v <= 100:
            errors.append(f"{key} phải là số trong (0, 100], nhận {v!r}")
        else:
            scores[key] = float(v)
    if len(scores) == 2 and scores["watch_at"] >= scores["review_at"]:
        errors.append("watch_at phải nhỏ hơn review_at")
    version = raw.get("version")
    if version != SUPPORTED_VERSION or isinstance(version, bool):
        errors.append(f"version phải là {SUPPORTED_VERSION}, nhận {version!r}")
    rules_raw = raw.get("rules")
    specs: list[RuleSpec] = []
    if not isinstance(rules_raw, list) or not rules_raw:
        errors.append("rules phải là danh sách không rỗng")
        rules_raw = []
    seen: set[str] = set()
    for i, r in enumerate(rules_raw):
        n_before = len(errors)
        rid = r.get("id") if isinstance(r, dict) else None
        where = f"rules[{i}]" + (f" ({rid})" if isinstance(rid, str) else "")
        if not isinstance(r, dict):
            errors.append(f"{where}: mỗi rule phải là một mapping")
            continue
        for key in sorted(set(r) - RULE_KEYS, key=str):
            errors.append(f"{where}: khóa không biết {key!r} (gõ nhầm?)")
        if not isinstance(rid, str) or not RULE_ID_RE.match(rid):
            errors.append(f"{where}: id phải khớp [a-z][a-z0-9_]*")
        elif rid in seen:
            errors.append(f"{where}: id trùng")
        if isinstance(rid, str):
            seen.add(rid)
        points = r.get("points")
        if isinstance(points, bool) or not isinstance(points, int) or not 1 <= points <= 100:
            errors.append(f"{where}: points phải là số nguyên 1..100, nhận {points!r}")
        when_raw = r.get("when")
        conds: list[Condition] = []
        if not isinstance(when_raw, list) or not when_raw:
            errors.append(f"{where}: when phải là danh sách điều kiện không rỗng")
        else:
            for c in when_raw:
                cond = _parse_condition(c, where, errors)
                if cond is not None:
                    conds.append(cond)
        _check_reason(r.get("reason"), {c.feature for c in conds}, where, errors)
        if len(errors) == n_before:
            specs.append(RuleSpec(rid, points, tuple(conds), r["reason"]))
    if errors:
        raise ConfigError(errors)
    return EngineConfig(version, scores["review_at"], scores["watch_at"], aggregate, tuple(specs))


def load_config(path: Path) -> EngineConfig:
    return parse_config(path.read_text(encoding="utf-8"))


# ---- Rule và engine -------------------------------------------------------------------------


def build_rule(spec: RuleSpec) -> Rule:
    """Closure giữ `spec` (bài 19): trả hàm `(features) -> RuleHit | None`.

    Thiếu KHÓA đặc trưng thì `features[...]` ném KeyError (lỗi lược đồ, phải thấy ngay); giá trị None chỉ làm rule không bắn.
    """

    def rule(features: Features) -> RuleHit | None:
        # lấy MỌI giá trị trước khi so: `all()` dừng ở điều kiện sai đầu tiên sẽ không bao giờ đụng tới khóa thứ hai,
        # nên một khóa bị đổi tên sẽ làm rule im lặng thay vì báo lỗi lược đồ
        values = [features[c.feature] for c in spec.when]
        if all(c.holds(v) for c, v in zip(spec.when, values, strict=True)):
            return RuleHit(spec.id, spec.points, spec.reason.format_map(features))
        return None

    return rule


class RuleEngine:
    """Chạy mọi rule trên đặc trưng của một chuyến, gộp điểm, gắn dải. Rule là callable bất kỳ, không nhất thiết từ YAML."""

    def __init__(self, rules: Sequence[Rule], aggregate: Aggregator, *, review_at: float, watch_at: float) -> None:
        self.rules = tuple(rules)
        self.aggregate = aggregate
        self.review_at = review_at
        self.watch_at = watch_at

    @classmethod
    def from_config(cls, cfg: EngineConfig) -> "RuleEngine":
        return cls([build_rule(s) for s in cfg.rules], AGGREGATORS[cfg.aggregate], review_at=cfg.review_at, watch_at=cfg.watch_at)

    def score(self, features: Features) -> Verdict:
        hits = tuple(h for rule in self.rules if (h := rule(features)) is not None)
        score = self.aggregate(hits)
        band = "review" if score >= self.review_at else "watch" if score >= self.watch_at else "none"
        return Verdict(score, band, hits)


# ---- Đường theo lô (polars): CÙNG định nghĩa, để đo trên hàng trăm nghìn chuyến ----------------


def condition_expr(c: Condition) -> pl.Expr:
    known = pl.col(c.feature).cast(pl.Float64).is_not_nan().fill_null(False)  # None và NaN: không biết
    return (OPS[c.op](pl.col(c.feature), c.value).fill_null(False)) & known


def rule_flags(df: pl.DataFrame, specs: Sequence[RuleSpec]) -> pl.DataFrame:
    """Một cột boolean mỗi rule (tên cột = id rule): rule có bắn trên chuyến đó không."""
    return df.select([pl.all_horizontal([condition_expr(c) for c in s.when]).alias(s.id) for s in specs])


def score_frame(flags: pl.DataFrame, specs: Sequence[RuleSpec], aggregate: str) -> pl.Series:
    """Điểm 0..100 của từng chuyến từ bảng cờ rule, theo đúng ba aggregator ở trên."""
    pts = [pl.col(s.id).cast(pl.Float64) * s.points for s in specs]
    match aggregate:
        case "sum":
            expr = pl.sum_horizontal(pts).clip(upper_bound=100.0)
        case "max":
            expr = pl.max_horizontal(pts)
        case "noisy_or":
            miss = [1.0 - p / 100.0 for p in pts]
            expr = 100.0 * (1.0 - reduce(operator.mul, miss))
        case _:
            raise ValueError(f"aggregate không biết: {aggregate}")
    return flags.select(expr.alias("score"))["score"]
