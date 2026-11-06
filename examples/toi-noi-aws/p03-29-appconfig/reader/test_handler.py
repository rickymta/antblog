"""Kiểm logic báo giá và rollout trên máy, không cần AWS: python3 -m unittest discover -s reader"""

import json
import pathlib
import unittest

import handler

ROOT = pathlib.Path(__file__).resolve().parent.parent
TABLE = json.loads((ROOT / "config" / "pricing-table.json").read_text())
FLAGS = json.loads((ROOT / "flags" / "pricing-flags.json").read_text())
FLAG_VALUE = FLAGS["values"]["peak-pricing-v2"]


class QuoteTests(unittest.TestCase):
    def test_base_quote_for_five_km_bike(self):
        event = {"city": "ha-noi", "customerId": "c-1", "vehicle": "bike", "km": 5}
        self.assertEqual(handler.quote(TABLE, FLAG_VALUE, event)["total"], 25500)

    def test_flag_off_for_other_city(self):
        event = {"city": "da-nang", "customerId": "c-1", "vehicle": "bike", "km": 5}
        self.assertFalse(handler.quote(TABLE, FLAG_VALUE, event)["peakPricingV2"])

    def test_full_rollout_applies_multiplier(self):
        flag = {**FLAG_VALUE, "rolloutPercent": 100}
        event = {"city": "ho-chi-minh", "customerId": "c-1", "vehicle": "bike", "km": 5}
        self.assertEqual(handler.quote(TABLE, flag, event)["total"], round(25500 * 1.2))

    def test_zero_rollout_applies_nobody(self):
        flag = {**FLAG_VALUE, "rolloutPercent": 0}
        event = {"city": "ho-chi-minh", "customerId": "c-1", "vehicle": "bike", "km": 5}
        self.assertFalse(handler.quote(TABLE, flag, event)["peakPricingV2"])

    def test_rollout_is_close_to_five_percent_and_stable(self):
        ids = [f"c-{n}" for n in range(10000)]
        hits = sum(handler.in_rollout(i, 5) for i in ids)
        self.assertTrue(400 <= hits <= 600, hits)
        self.assertEqual(handler.in_rollout("c-42", 5), handler.in_rollout("c-42", 5))


if __name__ == "__main__":
    unittest.main()
