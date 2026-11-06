import pytest

import evaluation as ev
from postprocess import FieldResult


def field(conf=0.95, value="Nam", issues=()):
    return FieldResult("sex", value, value, conf, issues=issues)


def test_confident_clean_field_is_auto_accepted():
    assert not ev.field_needs_review(field(0.95), 0.8)


def test_threshold_is_inclusive_for_accept():
    assert not ev.field_needs_review(field(0.80), 0.8)
    assert ev.field_needs_review(field(0.79), 0.8)


def test_issue_sends_to_review_even_with_high_confidence():
    assert ev.field_needs_review(field(0.99, issues=("impossible_date",)), 0.0)


def test_missing_value_always_goes_to_review():
    assert ev.field_needs_review(FieldResult("sex", None, None, 0.0), 0.0)


BASE = {"cer_raw_by_field": {"a": 0.10, "b": 0.0}, "page_recall": 0.94, "fields_correct": 213, "fields_total": 250,
        "auto_wrong_fields": 4, "auto_wrong_cards": 1, "card_review_rate": 0.40}


def changed(**kw):
    return {**BASE, **kw}


def test_same_numbers_pass():
    assert ev.check_regression(BASE, BASE) == []


@pytest.mark.parametrize(("cer_a", "expected_failures"), [(0.12, 0), (0.13, 1)])
def test_cer_tolerance_boundary(cer_a, expected_failures):
    current = changed(cer_raw_by_field={"a": cer_a, "b": 0.0})
    assert len(ev.check_regression(current, BASE)) == expected_failures


def test_one_more_wrong_auto_accept_fails_but_fewer_passes():
    assert len(ev.check_regression(changed(auto_wrong_fields=5), BASE)) == 1
    assert ev.check_regression(changed(auto_wrong_fields=3), BASE) == []


def test_one_more_card_slipping_through_fails():
    assert len(ev.check_regression(changed(auto_wrong_cards=2), BASE)) == 1


def test_one_fewer_correct_field_fails_without_tolerance():
    failures = ev.check_regression(changed(fields_correct=212), BASE)
    assert len(failures) == 1 and "số trường đúng" in failures[0]
    assert ev.check_regression(changed(fields_correct=214), BASE) == []


def test_page_recall_drop_beyond_tolerance_fails():
    assert ev.check_regression(changed(page_recall=0.92), BASE) == []
    assert len(ev.check_regression(changed(page_recall=0.91), BASE)) == 1


def test_review_rate_jump_fails():
    assert len(ev.check_regression(changed(card_review_rate=0.51), BASE)) == 1


ENV = {"tesseract": "tesseract 5.5.4", "traineddata_sha256": {"vie": "aa", "eng": "bb"},
       "font": {"name": "Arial.ttf", "sha256": "ff"}}


def test_environment_mismatch_is_reported_separately():
    assert ev.check_environment(ENV, ENV) == []
    other_model = {**ENV, "traineddata_sha256": {"vie": "zz", "eng": "bb"}}
    problems = ev.check_environment(other_model, ENV)
    assert len(problems) == 1 and "traineddata_sha256" in problems[0]
    other_font = {**ENV, "font": {"name": "Arial.ttf", "sha256": "00"}}
    assert len(ev.check_environment(other_font, ENV)) == 1
