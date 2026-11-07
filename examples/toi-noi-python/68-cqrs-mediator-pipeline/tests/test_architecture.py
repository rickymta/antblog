"""Kiểm kiến trúc bằng import-linter: hợp đồng đúng với code thật, và PHẢI vỡ khi ta cố ý phá.

Test «phải vỡ» chạy trên bản sao trong thư mục tạm, không đụng code thật.
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

LAB = Path(__file__).resolve().parent.parent


def lint(workdir: Path) -> subprocess.CompletedProcess[str]:
    env = {**os.environ, "PYTHONPATH": str(workdir / "src")}  # import-linter phân tích bản sao
    exe = Path(sys.executable).parent / "lint-imports"
    return subprocess.run(
        [str(exe)], cwd=workdir, env=env, capture_output=True, text=True, check=False
    )


@pytest.fixture
def copy_of_lab(tmp_path: Path) -> Path:
    shutil.copytree(LAB / "src", tmp_path / "src", ignore=shutil.ignore_patterns("__pycache__"))
    shutil.copy(LAB / "pyproject.toml", tmp_path / "pyproject.toml")
    return tmp_path


def test_real_code_keeps_both_contracts() -> None:
    result = lint(LAB)
    assert result.returncode == 0, result.stdout
    assert "Contracts: 3 kept, 0 broken." in result.stdout


def test_domain_importing_infrastructure_breaks_layers(copy_of_lab: Path) -> None:
    target = copy_of_lab / "src/kyc_service/domain/kyc.py"
    target.write_text(
        "from kyc_service.infrastructure.memory_repository import InMemoryKycRepository  # noqa\n"
        + target.read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    result = lint(copy_of_lab)
    assert result.returncode == 1
    assert "Bốn lớp: api > infrastructure > application > domain BROKEN" in result.stdout
    assert "kyc_service.domain.kyc -> kyc_service.infrastructure.memory_repository" in result.stdout


def test_application_importing_fastapi_breaks_core_contract(copy_of_lab: Path) -> None:
    target = copy_of_lab / "src/kyc_service/application/handlers.py"
    target.write_text(
        "import fastapi  # noqa\n" + target.read_text(encoding="utf-8"), encoding="utf-8"
    )
    result = lint(copy_of_lab)
    assert result.returncode == 1
    assert "Lõi không import framework web BROKEN" in result.stdout


def test_domain_importing_pydantic_breaks_its_own_contract(copy_of_lab: Path) -> None:
    # Application được dùng pydantic (behavior validation), domain thì không.
    target = copy_of_lab / "src/kyc_service/domain/errors.py"
    target.write_text(
        "import pydantic  # noqa\n" + target.read_text(encoding="utf-8"), encoding="utf-8"
    )
    result = lint(copy_of_lab)
    assert result.returncode == 1
    assert "Domain không import pydantic BROKEN" in result.stdout


def test_domain_importing_application_breaks_layers(copy_of_lab: Path) -> None:
    # Lõi trong cùng cũng không được import lớp ngoài nó một bậc.
    target = copy_of_lab / "src/kyc_service/domain/errors.py"
    target.write_text(
        "from kyc_service.application.errors import PortFailure  # noqa\n"
        + target.read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    result = lint(copy_of_lab)
    assert result.returncode == 1
    assert "kyc_service.domain.errors -> kyc_service.application.errors" in result.stdout


def test_new_module_outside_the_four_layers_breaks_exhaustive_contract(copy_of_lab: Path) -> None:
    # Module mới ở gốc package (ví dụ workers.py) mà chưa xếp lớp: không được lọt khỏi kiểm tra.
    (copy_of_lab / "src/kyc_service/workers.py").write_text(
        "from kyc_service import api, infrastructure  # noqa\n", encoding="utf-8"
    )
    result = lint(copy_of_lab)
    assert result.returncode == 1
    assert "kyc_service.workers" in result.stdout
