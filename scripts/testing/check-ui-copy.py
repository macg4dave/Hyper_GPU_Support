"""Focused copy guard plus advisory lexical findings; not inventory reconciliation.

Rust test modules, test fixtures, generated output and comments are outside scope.
Known baseline literals in enforced paths are errors. Other lexical candidates are
advisory because strings may be protocols, metadata or application identities.
"""
import argparse
import json
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[2]
POLICY = ROOT / "tests/fixtures/ui-text/validator/copy-scan.json"
# Rust raw/basic strings and Slint basic strings. Character literals are skipped.
TOKEN = re.compile(r'//[^\n]*|/\*.*?\*/|r(?P<hash>\#*)".*?"(?P=hash)|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'', re.S)


def literals(source, rust=True):
    if rust:
        # Current production files keep test-only modules after this attribute.
        # This is deliberately advisory lexical scanning, not a Rust parser.
        source = re.split(r"#\[cfg\(test\)\]", source, maxsplit=1)[0]
    for match in TOKEN.finditer(source):
        token = match.group()
        if token.startswith(("//", "/*", "'")):
            continue
        line = source.count("\n", 0, match.start()) + 1
        if token.startswith("r"):
            hashes = match.group("hash")
            value = token[2 + len(hashes):-(1 + len(hashes))]
        else:
            # Decode only understood escapes. Unknown syntax remains advisory.
            value = token[1:-1]
            value = re.sub(r"\\u\{([0-9a-fA-F]+)\}", lambda m: chr(int(m[1], 16)), value)
            value = re.sub(r"\\([nrt\\\"])", lambda m: {"n": "\n", "r": "\r", "t": "\t", "\\": "\\", '"': '"'}[m[1]], value)
        yield line, value


def run(root=ROOT, policy_path=POLICY):
    policy = json.loads(policy_path.read_text(encoding="utf-8"))
    if policy.get("schema") != 1:
        raise ValueError("unsupported copy scan policy")
    known = set()
    for path in sorted((root / policy["baseline"]).glob("*.toml")):
        document = tomllib.loads(path.read_text(encoding="utf-8"))
        known.update(entry["text"] for entry in document["messages"].values())
    if not known:
        raise ValueError("copy scan baseline is missing or empty")
    failures, advisories = [], {}
    for entry in policy["paths"]:
        path = root / entry["path"]
        if entry["mode"] not in ("enforce", "pending") or not entry["reason"].strip():
            raise ValueError("invalid reviewed path policy")
        if not path.is_file():
            failures.append(f'{entry["path"]}: reviewed source path missing')
            continue
        count = 0
        for line, value in literals(path.read_text(encoding="utf-8"), path.suffix == ".rs"):
            if value in known and entry["mode"] == "enforce":
                failures.append(f'{entry["path"]}:{line}: hard-coded active baseline copy')
            if value in known or re.search(r"[A-Za-z]{2,}\s+[A-Za-z]{2,}", value):
                count += 1
        advisories[entry["path"]] = {"candidates": count, "mode": entry["mode"], "reason": entry["reason"]}
    return sorted(failures), advisories


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="include advisory path counts")
    args = parser.parse_args()
    try:
        failures, advisories = run()
    except (OSError, ValueError, KeyError) as error:
        print(f"Copy scan failed: {error}", file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps({"errors": failures, "advisory": advisories}, indent=2))
    else:
        print(f"Reviewed {len(advisories)} paths; {len(failures)} hard-coded copy errors. Lexical candidates remain advisory; Rust tests are excluded.")
        for failure in failures:
            print(failure, file=sys.stderr)
    return int(bool(failures))


if __name__ == "__main__":
    sys.exit(main())
