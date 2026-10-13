"""Explicit LANG-004 resource-edit proof; restores original bytes in finally.

Only contributor resource files/build outputs and local evidence are affected.
Tests construct headless components; this never starts the live app or backend.
Run alone so another build/editor does not race the deliberate resource edits.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def main():
    directory = ROOT / "resources/i18n/en-GB"
    paths = [directory / f"{feature}.toml" for feature in ("common", "operations", "credentials")]
    extra = directory / "rebuild_probe.toml"
    if extra.exists() or any(path.is_symlink() or not path.is_file() for path in paths):
        raise RuntimeError("Prepared regular resource files required; probe must not already exist")
    original = {path: path.read_bytes() for path in paths}
    evidence = ROOT / "local/evidence" / ("lang-004-rebuild-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ"))
    evidence.mkdir(parents=True)
    report = {"scope": "Resource-only build proof; no live GUI/native/guest/protected effects", "original_hashes": {path.name: hashlib.sha256(data).hexdigest() for path, data in original.items()}, "steps": []}

    def run(name, arguments, expected_failure=None):
        print(f"{name}: {' '.join(arguments)}", flush=True)
        result = subprocess.run(arguments, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace")
        (evidence / f"{len(report['steps']):02d}.log").write_text(result.stdout, encoding="utf-8")
        passed = (result.returncode == 0) if expected_failure is None else (result.returncode != 0 and expected_failure in result.stdout)
        report["steps"].append({"name": name, "command": arguments, "exit_code": result.returncode, "expected_failure": expected_failure, "result": "Passed" if passed else "Failed"})
        (evidence / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        if not passed:
            raise RuntimeError(f"{name} failed; inspect {evidence}")

    def edit(path, key, text):
        source = path.read_text(encoding="utf-8")
        pattern = r'(^\[messages\."' + re.escape(key) + r'"\]\ntext = ).*'
        source, count = re.subn(pattern, lambda match: match[1] + json.dumps(text, ensure_ascii=False), source, flags=re.M)
        if count != 1:
            raise RuntimeError(f"Expected one prepared text field for {key}")
        path.write_text(source, encoding="utf-8")

    check = ["cargo", "check", "--locked", "--bin", "hyper-gpu-support"]
    try:
        run("Original bindings", ["cargo", "test", "--locked", "--test", "ui_text"])
        edit(paths[1], "operations.verify.title", 'λ "x" \\{{root.action("verify")}}\n{{literal}}')
        edit(paths[1], "operations.apply.power_consent", '{final_power}\nλ "x" \\ {{literal}}')
        edit(paths[0], "common.state.unknown", 'λ "u"\n{{literal}}')
        edit(paths[2], "credentials.guest_sign_in.message", '{vm_name}\nλ "x" \\ {{literal}}')
        run("TOML-only Rust rebuild and data insertion", ["cargo", "test", "--locked", "--test", "ui_text", "catalogue_active_bindings"])
        run("TOML-only Slint rebuild and two instances", ["cargo", "test", "--locked", "--bin", "hyper-gpu-support", "gui::catalogue_tests"])
        extra.write_text('schema = 1\nlocale = "en-GB"\nfeature = "rebuild_probe"\nmessages = {}\n', encoding="utf-8")
        run("Unknown file addition invalidates build", check, "unknown file")
        extra.unlink()
        run("Unknown file removal rebuilds", check)
        paths[2].unlink()
        run("Required file deletion invalidates build", check, "missing required file")
        paths[2].write_bytes(original[paths[2]])
        edit(paths[1], "operations.verify.title", "{")
        run("Malformed template blocks generation", check, "unmatched opening brace")
    finally:
        for path, data in original.items():
            path.write_bytes(data)
        if extra.exists():
            extra.unlink()
        report["restored_hashes"] = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
        report["restored"] = report["restored_hashes"] == report["original_hashes"]
        (evidence / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    run("Restored verbatim Rust bindings", ["cargo", "test", "--locked", "--test", "ui_text"])
    run("Restored Slint bindings", ["cargo", "test", "--locked", "--bin", "hyper-gpu-support", "gui::catalogue_tests"])
    print(f"Passed; original resource bytes restored. Evidence: {evidence}")


if __name__ == "__main__":
    main()
