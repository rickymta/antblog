import random
import unicodedata

import pytest

import metrics


def test_cer_kitten_sitting_by_hand():
    # kitten -> sitting: thay k->s, thay e->i, chèn g: S=2, D=0, I=1, N=6
    counts = metrics.char_errors(["kitten"], ["sitting"])
    assert (counts.substitutions, counts.deletions, counts.insertions) == (2, 0, 1)
    assert counts.reference_chars == 6
    assert counts.cer == pytest.approx(0.5)
    assert metrics.cer(["kitten"], ["sitting"]) == pytest.approx(0.5)


def test_missing_diacritics_cost_little_chars_but_most_words():
    ref, hyp = "NGUYỄN VĂN A", "NGUYEN VAN A"
    assert metrics.cer([ref], [hyp]) == pytest.approx(2 / 12)  # hai ký tự mất dấu
    assert metrics.wer([ref], [hyp]) == pytest.approx(2 / 3)  # hai trong ba từ sai


def test_nfd_vs_nfc_is_not_an_ocr_error():
    ref = "NGUYỄN"
    decomposed = unicodedata.normalize("NFD", ref)
    # Ễ (1 mã) thành E + dấu mũ + dấu ngã (3 mã): 1 thay + 2 chèn = 3 phép sửa
    assert metrics.edit_distance(ref, decomposed) == 3
    assert metrics.cer([ref], [decomposed]) == 0.0  # cer() chuẩn hóa NFC hai phía


def test_corpus_cer_is_not_mean_of_cers():
    refs, hyps = ["099085791907", "D"], ["099085791907", "O"]
    assert metrics.cer(refs, hyps) == pytest.approx(1 / 13)
    assert (metrics.cer(refs[:1], hyps[:1]) + metrics.cer(refs[1:], hyps[1:])) / 2 == pytest.approx(0.5)


def test_empty_hypothesis_counts_as_deleting_everything():
    assert metrics.cer(["abc"], [""]) == 1.0
    counts = metrics.char_errors(["abc"], [""])
    assert (counts.hits, counts.deletions) == (0, 3)


def test_cer_can_exceed_one_when_ocr_inserts_a_lot():
    assert metrics.cer(["ab"], ["abxxxx"]) == pytest.approx(2.0)


@pytest.mark.parametrize("seed", range(5))
def test_hand_implementation_matches_jiwer(seed):
    rng = random.Random(seed)
    refs = ["".join(rng.choices("abcdễ 01", k=rng.randint(1, 12))).strip() or "a" for _ in range(60)]
    hyps = ["".join(rng.choices("abcdễ 01", k=rng.randint(0, 12))) for _ in range(60)]
    assert metrics.cer_by_hand(refs, hyps) == pytest.approx(metrics.cer(refs, hyps))
    assert metrics.wer_by_hand(refs, hyps) == pytest.approx(metrics.wer(refs, hyps))
