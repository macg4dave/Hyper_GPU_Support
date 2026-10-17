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
    def check_plan(self, dependencies, *, roadmap=None, extra="", card_dependencies=None, qa_layout=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            docs.mkdir()
            rows = []
            cards = []
            for task, deps in dependencies.items():
                if qa_layout:
                    rows.append(f"| {task} | Prepare | Work | Ready | {', '.join(deps) or 'None'} | Not run |")
                else:
                    rows.append(f"| {task} | Ready | Not run | {', '.join(deps) or 'None'} | L1 |")
                declared = (card_dependencies or {}).get(task, deps)
                cards.append(f"### {task} - Work\n\nDependencies: {', '.join(declared) or 'None'}.\n")
            (docs / "backlog.md").write_text(
                "# Backlog\n"
                + ("| ID | Stage | Task | Implementation | Dependencies | Validation |\n" if qa_layout
                   else "| ID | Implementation | Validation | Dependencies | Stage |\n")
                + "\n".join(rows + cards) + extra, encoding="utf-8"
            )
            (docs / "roadmap.md").write_text("# Roadmap\n" + (roadmap if roadmap is not None else next(iter(dependencies), "No tasks")), encoding="utf-8")
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

    def test_qa_plan_accepts_historical_language_context(self):
        result, output = self.check_plan(
            {"QA-001": [], "QA-002": ["QA-001"]},
            roadmap="QA-001 and QA-002; LANG-006 is historical.",
            qa_layout=True,
        )
        self.assertEqual(result, 0, output)

    def test_unknown_qa_roadmap_task_fails(self):
        result, output = self.check_plan({"QA-001": []}, roadmap="QA-099")
        self.assertEqual(result, 1)
        self.assertIn("unknown task QA-099", output)

    def test_qa_dependency_cycle_fails(self):
        result, output = self.check_plan(
            {"QA-001": ["QA-002"], "QA-002": ["QA-001"]}
        )
        self.assertEqual(result, 1)
        self.assertIn("cyclic task dependency", output)

    def test_qa_missing_dependency_fails(self):
        result, output = self.check_plan({"QA-001": ["QA-099"]})
        self.assertEqual(result, 1)
        self.assertIn("invalid dependency QA-099", output)

    def test_qa_card_register_disagreement_fails(self):
        result, output = self.check_plan(
            {"QA-001": [], "QA-002": ["QA-001"]},
            card_dependencies={"QA-002": []},
        )
        self.assertEqual(result, 1)
        self.assertIn("card and register dependencies differ", output)

    def test_qa_duplicate_register_fails(self):
        result, output = self.check_plan(
            {"QA-001": []}, extra="\n| QA-001 | Ready | Not run | None | Prepare |\n"
        )
        self.assertEqual(result, 1)
        self.assertIn("same unique task IDs", output)

    def test_empty_plan_requires_explicit_status(self):
        result, output = self.check_plan({}, roadmap="No tasks")
        self.assertEqual(result, 1)
        self.assertIn("Empty backlog must explicitly state", output)

    def test_optimization_plan_accepts_independent_work_and_forward_dependencies(self):
        result, output = self.check_plan(
            {"OPT-001": [], "OPT-002": [], "OPT-008": ["OPT-001", "OPT-002", "OPT-004"], "OPT-004": []},
            roadmap="OPT-001, OPT-002, OPT-004 and OPT-008; QA-014 and LANG-006 are historical.",
            qa_layout=True,
        )
        self.assertEqual(result, 0, output)
        self.assertIn("4 task cards; 0 errors", output)

    def test_optimization_unknown_tasks_and_dependencies_fail(self):
        cases = [
            ({"OPT-001": []}, {"roadmap": "OPT-099"}, "unknown task OPT-099"),
            ({"OPT-001": ["OPT-099"]}, {}, "invalid dependency OPT-099"),
        ]
        for dependencies, options, expected in cases:
            with self.subTest(expected=expected):
                result, output = self.check_plan(dependencies, **options)
                self.assertEqual(result, 1, output)
                self.assertIn(expected, output)

    def test_optimization_cycles_duplicates_and_dependency_disagreement_fail(self):
        cases = [
            ({"OPT-001": ["OPT-002"], "OPT-002": ["OPT-001"]}, {}, "cyclic task dependency"),
            ({"OPT-001": []}, {"extra": "\n| OPT-001 | Ready | Not run | None | R0 |\n"}, "same unique task IDs"),
            ({"OPT-001": [], "OPT-002": ["OPT-001"]}, {"card_dependencies": {"OPT-002": []}}, "card and register dependencies differ"),
        ]
        for dependencies, options, expected in cases:
            with self.subTest(expected=expected):
                result, output = self.check_plan(dependencies, **options)
                self.assertEqual(result, 1, output)
                self.assertIn(expected, output)

    def test_explicitly_empty_backlog_is_valid(self):
        result, output = self.check_plan({}, extra="\nNo open tasks.\n")
        self.assertEqual(result, 0, output)


if __name__ == "__main__":
    unittest.main()
