"""Check tracked Markdown links, anchors, task references and whitespace locally."""
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[2]


def anchors(text):
    result = set()
    counts = {}
    for heading in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", text, re.M):
        slug = re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        result.add(f"{slug}-{count}" if count else slug)
    result.update(re.findall(r'<a\s+(?:id|name)="([^"]+)"', text))
    return result


def main():
    tracked = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).decode().split("\0")
    deleted = set(subprocess.check_output(["git", "ls-files", "--deleted", "-z"], cwd=ROOT).decode().split("\0"))
    # Validate the working tree, including deliberate unstaged deletions. Links
    # from remaining documents to removed files must still fail below.
    files = {ROOT / name for name in tracked if name.endswith(".md") and name not in deleted}
    files.update((ROOT / "docs").rglob("*.md"))
    errors = []
    for path in sorted(files):
        if not path.exists():
            errors.append(f"Missing tracked document: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        for target in re.findall(r"(?<!!)\[[^\]\n]+\]\(([^)]+)\)", text):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][\w+.-]*:", target):
                continue
            filename, _, fragment = unquote(target).partition("#")
            destination = (path.parent / filename).resolve() if filename else path
            if not destination.exists():
                errors.append(f"{path.relative_to(ROOT)}: missing link {target}")
            elif fragment and destination.suffix == ".md" and fragment not in anchors(destination.read_text(encoding="utf-8")):
                errors.append(f"{path.relative_to(ROOT)}: missing anchor {target}")
    backlog = (ROOT / "docs/BACKLOG.md").read_text(encoding="utf-8")
    cards = re.findall(r"^### (APP-\d{3})\b", backlog, re.M)
    registered = re.findall(r"^\| (APP-\d{3}) \|", backlog, re.M)
    if len(cards) != len(set(cards)) or set(cards) != set(registered):
        errors.append("Backlog cards and register must contain the same unique task IDs")
    for name in ("ROADMAP.md", "BACKLOG.md", "BASELINE.md", "TESTING.md"):
        for task in set(re.findall(r"APP-\d{3}", (ROOT / "docs" / name).read_text(encoding="utf-8"))):
            if task not in cards:
                errors.append(f"{name}: unknown task {task}")
    for row in re.findall(r"^\| APP-\d{3} \|.*$", backlog, re.M):
        task, dependencies = row.split("|")[1].strip(), row.split("|")[4].strip()
        for dependency in re.findall(r"APP-\d{3}", dependencies):
            if dependency not in cards or dependency >= task:
                errors.append(f"{task}: invalid dependency {dependency}")
    whitespace = subprocess.run(["git", "-c", "core.safecrlf=false", "diff", "--check"], cwd=ROOT)
    for error in errors:
        print(error, file=sys.stderr)
    print(f"Checked {len(files)} Markdown files and {len(cards)} task cards; {len(errors)} errors.")
    return int(bool(errors) or whitespace.returncode != 0)


if __name__ == "__main__":
    sys.exit(main())
