"""Isolated documentation-plan checks; no product or Git mutation."""
import contextlib
import importlib.util
import io
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "check_docs", Path(__file__).with_name("check-docs.py")
)
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


class PlanChecks(unittest.TestCase):
    def check_plan(self, dependencies, *, roadmap="LANG-001", extra="", card_dependencies=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            docs.mkdir()
            rows = []
            cards = []
            for task, deps in dependencies.items():
                rows.append(f"| {task} | Ready | Not run | {', '.join(deps) or 'None'} | L1 |")
                declared = (card_dependencies or {}).get(task, deps)
                cards.append(f"### {task} - Work\n\nDependencies: {', '.join(declared) or 'None'}.\n")
            (docs / "backlog.md").write_text(
                "# Backlog\n" + "\n".join(rows + cards) + extra, encoding="utf-8"
            )
            (docs / "roadmap.md").write_text("# Roadmap\n" + roadmap, encoding="utf-8")
            output = io.StringIO()
            with patch.object(CHECK, "ROOT", root), patch.object(
                CHECK.subprocess, "check_output", return_value=b""
            ), patch.object(
                CHECK.subprocess, "run", return_value=SimpleNamespace(returncode=0)
            ), contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
                result = CHECK.main()
            return result, output.getvalue()

    def test_forward_dependency_and_completed_app_context_are_valid(self):
        result, output = self.check_plan(
            {"LANG-001": ["LANG-013"], "LANG-013": []},
            roadmap="LANG-001 and LANG-013; APP-999 is historical.",
        )
        self.assertEqual(result, 0, output)

    def test_dependency_cycle_fails(self):
        result, output = self.check_plan(
            {"LANG-001": ["LANG-002"], "LANG-002": ["LANG-001"]}
        )
        self.assertEqual(result, 1)
        self.assertIn("cyclic task dependency", output)

    def test_missing_dependency_fails(self):
        result, output = self.check_plan({"LANG-001": ["LANG-099"]})
        self.assertEqual(result, 1)
        self.assertIn("invalid dependency LANG-099", output)

    def test_unknown_roadmap_task_fails(self):
        result, output = self.check_plan({"LANG-001": []}, roadmap="LANG-099")
        self.assertEqual(result, 1)
        self.assertIn("unknown task LANG-099", output)

    def test_card_register_disagreement_fails(self):
        result, output = self.check_plan(
            {"LANG-001": [], "LANG-002": ["LANG-001"]},
            card_dependencies={"LANG-002": []},
        )
        self.assertEqual(result, 1)
        self.assertIn("card and register dependencies differ", output)

    def test_duplicate_register_fails(self):
        result, output = self.check_plan(
            {"LANG-001": []}, extra="\n| LANG-001 | Ready | Not run | None | L1 |\n"
        )
        self.assertEqual(result, 1)
        self.assertIn("same unique task IDs", output)

    def test_broken_link_still_fails(self):
        result, output = self.check_plan(
            {"LANG-001": []}, roadmap="LANG-001 [missing](missing.md)"
        )
        self.assertEqual(result, 1)
        self.assertIn("missing link missing.md", output)

    def test_empty_active_plan_fails(self):
        result, output = self.check_plan({}, roadmap="No tasks")
        self.assertEqual(result, 1)
        self.assertIn("same unique task IDs", output)


if __name__ == "__main__":
    unittest.main()
