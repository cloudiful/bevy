#!/usr/bin/env node

import { appendFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const SHARED_FILES = new Set([
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain",
  "rust-toolchain.toml",
]);
const SHARED_PREFIXES = [".cargo/"];
const ZERO_SHA = "0000000000000000000000000000000000000000";

function parseArgs(argv) {
  const [command, ...rest] = argv;
  const options = { command };

  for (let index = 0; index < rest.length; index += 2) {
    const key = rest[index];
    const value = rest[index + 1];
    options[key.replace(/^--/, "").replace(/-([a-z])/g, (_, char) => char.toUpperCase())] = value;
  }

  return options;
}

function runGit(args, { allowFailure = false } = {}) {
  try {
    return execFileSync("git", args, {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch (error) {
    if (allowFailure) {
      return null;
    }
    throw new Error(error.stderr?.trim() || error.message);
  }
}

function sectionBody(content, sectionName) {
  const escaped = sectionName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = content.match(new RegExp(`^\\[${escaped}\\]\\s*([\\s\\S]*?)(?=^\\[[^\\]]+\\]\\s*$|$)`, "m"));
  return match?.[1] ?? "";
}

function parsePackageVersion(content) {
  const packageSection = sectionBody(content, "package");
  const versionMatch = packageSection.match(/version\s*=\s*"([^"]+)"/m);
  if (!versionMatch) {
    throw new Error("Could not find [package].version in Cargo.toml");
  }
  return versionMatch[1];
}

function workspaceCrates() {
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }),
  );

  const workspaceMembers = new Set(metadata.workspace_members);
  return metadata.packages
    .filter((pkg) => workspaceMembers.has(pkg.id))
    .map((pkg) => ({
      crate: relative(ROOT, dirname(pkg.manifest_path)),
      version: pkg.version,
    }))
    .sort((left, right) => left.crate.localeCompare(right.crate));
}

function changedFiles(baseSha, headSha, eventName) {
  if (eventName === "workflow_dispatch") {
    return { affectsAll: true, files: [] };
  }

  if (!baseSha || baseSha === ZERO_SHA) {
    return { affectsAll: true, files: [] };
  }

  const files = runGit(["diff", "--name-only", baseSha, headSha])
    .split("\n")
    .map((value) => value.trim())
    .filter(Boolean);

  const affectsAll = files.some(
    (path) => SHARED_FILES.has(path) || SHARED_PREFIXES.some((prefix) => path.startsWith(prefix)),
  );

  return { affectsAll, files };
}

function crateChanged(crateDir, files, affectsAll) {
  if (affectsAll) {
    return true;
  }

  return files.some((path) => path === crateDir || path.startsWith(`${crateDir}/`));
}

function baseVersion(baseSha, crateDir) {
  if (!baseSha || baseSha === ZERO_SHA) {
    return null;
  }

  const content = runGit(["show", `${baseSha}:${crateDir}/Cargo.toml`], {
    allowFailure: true,
  });
  if (content == null) {
    return null;
  }

  try {
    return parsePackageVersion(content);
  } catch {
    return null;
  }
}

function versionChanged(current, previous) {
  return previous == null || current !== previous;
}

function buildPlan({ eventName, baseSha, beforeSha, headSha }) {
  const crates = workspaceCrates();
  const diffBase = baseSha || beforeSha || null;
  const { affectsAll, files } = changedFiles(diffBase, headSha, eventName);

  const testMatrix = [];
  const publishMatrix = [];

  for (const crate of crates) {
    const changed = crateChanged(crate.crate, files, affectsAll);
    if (changed) {
      testMatrix.push({ crate: crate.crate });
    }

    const previousVersion = baseVersion(diffBase, crate.crate);
    if (changed && versionChanged(crate.version, previousVersion)) {
      publishMatrix.push({ crate: crate.crate });
    }
  }

  return {
    hasTestCrates: testMatrix.length > 0,
    hasPublishCrates: publishMatrix.length > 0,
    testCrates: testMatrix.map((entry) => entry.crate),
    publishCrates: publishMatrix.map((entry) => entry.crate),
  };
}

function writeOutputs(outputPath, plan) {
  const lines = [
    `has_test_crates=${plan.hasTestCrates ? "true" : "false"}`,
    `has_publish_crates=${plan.hasPublishCrates ? "true" : "false"}`,
    `test_crates=${JSON.stringify(plan.testCrates)}`,
    `publish_crates=${JSON.stringify(plan.publishCrates)}`,
  ];
  appendFileSync(outputPath, `${lines.join("\n")}\n`, "utf8");
}

function parseCratesJson(input) {
  const crates = JSON.parse(input ?? "[]");
  if (!Array.isArray(crates) || crates.some((item) => typeof item !== "string")) {
    throw new Error("Expected JSON array of crate directory strings");
  }
  return crates;
}

function runCommand(command, args) {
  execFileSync(command, args, {
    cwd: ROOT,
    stdio: "inherit",
  });
}

function runTests(crates) {
  for (const crate of crates) {
    runCommand("cargo", ["test", "--manifest-path", `${crate}/Cargo.toml`]);
    runCommand("cargo", ["check", "--manifest-path", `${crate}/Cargo.toml`, "--examples"]);
  }
}

function runMultiPublish(crates, { kellnrToken, cratesIoToken }) {
  if (!kellnrToken && !cratesIoToken) {
    throw new Error("Expected at least one publish token");
  }

  if (cratesIoToken) {
    runPublishToRegistry(crates, {
      registry: "crates-io",
      token: cratesIoToken,
    });
  }

  if (kellnrToken) {
    runPublishToRegistry(crates, {
      registry: "kellnr",
      token: kellnrToken,
    });
  }
}

function runPublishToRegistry(crates, { registry, token }) {
  if (!token) {
    throw new Error(`Expected token for registry: ${registry}`);
  }

  for (const crate of crates) {
    runCommand("cargo", [
      "publish",
      "--manifest-path",
      `${crate}/Cargo.toml`,
      "--registry",
      registry,
      "--token",
      token,
    ]);
  }
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.command === "plan") {
    const plan = buildPlan({
      eventName: args.eventName,
      baseSha: args.baseSha,
      beforeSha: args.beforeSha,
      headSha: args.headSha,
    });

    if (args.githubOutput) {
      writeOutputs(args.githubOutput, plan);
      return;
    }

    process.stdout.write(`${JSON.stringify(plan, null, 2)}\n`);
    return;
  }

  if (args.command === "test") {
    runTests(parseCratesJson(args.cratesJson));
    return;
  }

  if (args.command === "publish") {
    runMultiPublish(parseCratesJson(args.cratesJson), {
      kellnrToken: args.kellnrToken,
      cratesIoToken: args.cratesIoToken,
    });
    return;
  }

  throw new Error("Expected command: plan, test, or publish");
}

main();
