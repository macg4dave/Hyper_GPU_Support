// PreToolUse guard: makes Claude ask the user before it adds lint suppressions,
// ignored tests, placeholder macros, or lint-weakening config/flags.
// It compares before/after text so editing code near existing attributes is not flagged.
import { readFileSync, existsSync } from "node:fs";

const input = JSON.parse(readFileSync(0, "utf8") || "{}");
const tool = input.tool_name ?? "";
const ti = input.tool_input ?? {};
const findings = [];

const count = (text, re) => (text.match(re) ?? []).length;
const grew = (before, after, re) => count(after, re) > count(before, re);

// Lint names inside #[allow(...)] / #[expect(...)] / cfg_attr(..., allow(...)).
function suppressedLints(text) {
  const lints = [];
  const attr = /\b(?:allow|expect)\s*\(([^)]*)\)/g;
  for (const m of text.matchAll(attr)) {
    for (const part of m[1].split(",")) {
      const name = part.trim();
      if (/^[a-z_]+(::[a-z_]+)*$/.test(name)) lints.push(name);
    }
  }
  return lints;
}

function newLints(before, after) {
  const pool = suppressedLints(before);
  const added = [];
  for (const lint of suppressedLints(after)) {
    const i = pool.indexOf(lint);
    if (i >= 0) pool.splice(i, 1);
    else added.push(lint);
  }
  return added;
}

function checkRust(before, after) {
  const added = newLints(before, after);
  // Narrow unsafe_code allows are the project's FFI pattern; require a new SAFETY comment.
  const unsafeOnly = added.length > 0 && added.every((l) => l === "unsafe_code");
  if (unsafeOnly && grew(before, after, /SAFETY:/g)) {
    // accepted
  } else if (added.length) {
    findings.push(`adds lint suppression(s): ${added.join(", ")}`);
  }
  if (grew(before, after, /#\s*\[\s*ignore\b/g)) findings.push("adds #[ignore] to a test");
  if (grew(before, after, /\b(todo|unimplemented)!\s*\(/g)) findings.push("adds todo!/unimplemented!");
}

function checkConfig(before, after) {
  if (grew(before, after, /=\s*["']allow["']|level\s*=\s*["']allow["']/g))
    findings.push('sets a lint to "allow" in config');
  if (grew(before, after, /["']-A\s*[a-z]|--cap-lints|["']-Awarnings/g))
    findings.push("adds lint-capping rustflags");
  if (count(after, /["'](deny|forbid)["']/g) < count(before, /["'](deny|forbid)["']/g))
    findings.push("removes a deny/forbid lint level");
}

function checkFile(path, before, after) {
  const p = (path ?? "").replace(/\\/g, "/").toLowerCase();
  if (p.endsWith(".rs")) checkRust(before, after);
  else if (/(^|\/)(cargo\.toml|\.?clippy\.toml|rustfmt\.toml|\.cargo\/config(\.toml)?)$/.test(p))
    checkConfig(before, after);
}

function readExisting(path) {
  try {
    return path && existsSync(path) ? readFileSync(path, "utf8") : "";
  } catch {
    return "";
  }
}

if (tool === "Edit") {
  checkFile(ti.file_path, ti.old_string ?? "", ti.new_string ?? "");
} else if (tool === "MultiEdit") {
  for (const e of ti.edits ?? []) checkFile(ti.file_path, e.old_string ?? "", e.new_string ?? "");
} else if (tool === "Write") {
  checkFile(ti.file_path, readExisting(ti.file_path), ti.content ?? "");
} else if (tool === "Bash" || tool === "PowerShell") {
  const cmd = ti.command ?? "";
  if (/--cap-lints|RUSTFLAGS[^\n]*-A|cargo\s+clippy[^\n]*\s-A\s|\s-A\s*(clippy|warnings|dead_code|unused)/.test(cmd))
    findings.push("runs cargo/clippy with lints allowed or capped");
  if (/#!?\[\s*(allow|expect)\s*\(/.test(cmd) || /\bcfg_attr\([^)]*\b(allow|expect)\s*\(/.test(cmd))
    findings.push("writes a lint suppression through the shell");
  if (/--no-verify\b/.test(cmd)) findings.push("skips git hooks (--no-verify)");
}

if (findings.length) {
  process.stdout.write(
    JSON.stringify({
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: "ask",
        permissionDecisionReason:
          "Lint guard: " +
          findings.join("; ") +
          ". Fix the underlying issue instead unless the user approves this exception.",
      },
    }),
  );
}
