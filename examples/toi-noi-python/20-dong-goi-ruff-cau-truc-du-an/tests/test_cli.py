import pytest

from toinoi_pricing.cli import main


def test_cli_prints_fare(capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["2.5"]) == 0
    assert capsys.readouterr().out == "2.5 km => 14250 VND\n"


def test_cli_accepts_service_option(capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["5", "--service", "food"]) == 0
    assert capsys.readouterr().out == "5 km => 25500 VND\n"


@pytest.mark.parametrize(
    ("argument", "message"),
    [("100.001", "ngoài miền"), ("-1", "ngoài miền"), ("abc", "không phải số")],
)
def test_cli_rejects_bad_input(
    argument: str, message: str, capsys: pytest.CaptureFixture[str]
) -> None:
    assert main([argument]) == 2
    captured = capsys.readouterr()
    assert captured.out == ""
    assert message in captured.err
