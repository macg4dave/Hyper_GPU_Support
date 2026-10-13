"""Isolated scan checks; no product execution or UI wording assertions."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
import json

spec = importlib.util.spec_from_file_location("copy_scan", Path(__file__).with_name("check-ui-copy.py"))
scan = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scan)


class ScanTests(unittest.TestCase):
    def test_comments_raw_strings_and_rust_tests(self):
        source = '// "x y"\nlet a = r#"{x} \\"#;\nlet b = "x\\ny";\n#[cfg(test)]\nmod tests { let c = "y z"; }'
        self.assertEqual(list(scan.literals(source)), [(2, '{x} \\'), (3, 'x\ny')])

    def test_enforced_copy_fails_pending_is_advisory_and_missing_path_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "baseline").mkdir()
            (root / "baseline/a.toml").write_text('[messages.x]\ntext = "x y"', encoding="utf-8")
            (root / "source.rs").write_text('let a = "x y";', encoding="utf-8")
            policy = {"schema": 1, "baseline": "baseline", "paths": [{"path": "source.rs", "mode": "enforce", "reason": "Synthetic migrated site"}]}
            path = root / "policy.json"
            path.write_text(json.dumps(policy), encoding="utf-8")
            errors, _ = scan.run(root, path)
            self.assertEqual(len(errors), 1)
            policy["paths"][0]["mode"] = "pending"
            path.write_text(json.dumps(policy), encoding="utf-8")
            errors, advisory = scan.run(root, path)
            self.assertFalse(errors)
            self.assertEqual(advisory["source.rs"]["candidates"], 1)
            (root / "source.rs").unlink()
            self.assertEqual(len(scan.run(root, path)[0]), 1)


if __name__ == "__main__":
    unittest.main()
