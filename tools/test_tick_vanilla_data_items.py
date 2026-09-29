import unittest

from tick_vanilla_data_items import tick

LINES = [
    "- [ ] Audit vanilla asset resource `a/one.json` for coverage.",
    "- [ ] Audit vanilla asset resource `a/two.json` for coverage.",
    "- [x] Audit vanilla asset resource `a/three.json` for coverage.",
]


class TickTest(unittest.TestCase):
    def test_only_listed_lines_are_ticked(self):
        out, ticked = tick(LINES, {"a/one.json", "a/three.json"})
        self.assertEqual(ticked, ["a/one.json"])
        self.assertTrue(out[0].startswith("- [x] "))
        self.assertTrue(out[1].startswith("- [ ] "))
        self.assertEqual(out[2], LINES[2])

    def test_prefix_paths_do_not_match(self):
        _, ticked = tick(LINES, {"a/one"})
        self.assertEqual(ticked, [])


if __name__ == "__main__":
    unittest.main()
