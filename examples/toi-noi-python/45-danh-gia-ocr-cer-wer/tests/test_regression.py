import evaluation as ev


def test_dataset_size(golden):
    items, results, _ = golden
    assert len(items) == 50
    assert sum(len(r.fields) for r in results) == 250


def test_environment_matches_baseline(golden):
    """Số đo chỉ so được khi cùng Tesseract, cùng mô hình `vie`/`eng`, cùng font vẽ thẻ."""
    problems = ev.check_environment(ev.environment(), ev.load_baseline()["environment"])
    assert problems == [], "\n".join(problems) + "\nxem lại rồi chạy main.py --update-baseline nếu chủ ý"


def test_no_regression_against_baseline(golden):
    items, results, pages = golden
    failures = ev.check_regression(ev.snapshot(items, results, pages), ev.load_baseline())
    assert failures == [], "\n".join(failures)


def test_gate_catches_downscaled_images(golden):
    """Thử phá 1: thu ảnh còn một nửa trước OCR phải làm regression gate fail."""
    items, _, _ = golden
    bad = ev.snapshot(items, ev.run_pipeline(items, transform=ev.downscale_half),
                      ev.page_texts(items, transform=ev.downscale_half))
    failures = ev.check_regression(bad, ev.load_baseline())
    assert any(line.startswith("CER") for line in failures)
    assert any("hàng chờ" in line for line in failures)


def test_gate_catches_broken_postprocessing_on_cards_already_queued(golden):
    """Thử phá 2: hậu xử lý trả None cho các thẻ vốn đã vào hàng chờ. OCR, tỉ lệ hàng chờ và CER
    thô không đổi; chỉ luật «số trường đúng không giảm» thấy được."""
    items, results, pages = golden
    broken = ev.snapshot(items, ev.drop_values(results), pages)
    failures = ev.check_regression(broken, ev.load_baseline())
    assert len(failures) == 1 and "số trường đúng" in failures[0]


def test_stricter_threshold_never_lets_more_wrong_fields_through(golden):
    items, results, _ = golden
    stats = [ev.gate_stats(items, results, t) for t in (0.0, 0.5, 0.7, 0.8, 0.9, 0.95)]
    assert all(a.auto_wrong >= b.auto_wrong for a, b in zip(stats, stats[1:], strict=False))
    assert all(a.field_review_rate <= b.field_review_rate for a, b in zip(stats, stats[1:], strict=False))


def test_page_recall_is_an_upper_bound_of_correct_fields(golden):
    items, results, pages = golden
    hit, total = ev.page_recall(items, pages)
    correct = sum(ev.is_correct(n, f.value, it.truth[n]) for it, r in zip(items, results, strict=True)
                  for n, f in r.fields.items())
    assert total == 250 and hit >= correct
