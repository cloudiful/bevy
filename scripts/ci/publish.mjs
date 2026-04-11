#!/usr/bin/env node

import { appendFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));

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
      name: pkg.name,
      version: pkg.version,
    }))
    .sort((left, right) => left.crate.localeCompare(right.crate));
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

function registryTokenEnvName(registry) {
  return `CARGO_REGISTRIES_${registry.replace(/-/g, "_").toUpperCase()}_TOKEN`;
}

function configuredRegistries({ kellnrToken, cratesIoToken }) {
  return [
    cratesIoToken ? { registry: "crates-io", token: cratesIoToken } : null,
    kellnrToken ? { registry: "kellnr", token: kellnrToken } : null,
  ].filter(Boolean);
}

function runCommandCapture(command, args, { env = {} } = {}) {
  try {
    return {
      status: 0,
      stdout: execFileSync(command, args, {
        cwd: ROOT,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
        env: {
          ...process.env,
          ...env,
        },
      }),
      stderr: "",
    };
  } catch (error) {
    return {
      status: error.status ?? 1,
      stdout: error.stdout?.toString?.() ?? "",
      stderr: error.stderr?.toString?.() ?? error.message,
    };
  }
}

function classifyRegistryInfoResult(result) {
  if (result.status === 0) {
    return "exists";
  }

  const combined = `${result.stdout}\n${result.stderr}`.toLowerCase();
  const missingSignals = [
    "could not find package",
    "could not find `",
    "does not exist in registry",
    "no matching package named",
  ];

  if (missingSignals.some((signal) => combined.includes(signal))) {
    return "missing";
  }

  return "error";
}

function formatVerificationError(crateInfo, registry, result) {
  const details = [result.stderr.trim(), result.stdout.trim()].filter(Boolean).join("\n");
  return details
    ? `Failed to verify ${crateInfo.name} ${crateInfo.version} in ${registry}.\n${details}`
    : `Failed to verify ${crateInfo.name} ${crateInfo.version} in ${registry}.`;
}

function formatVerificationSummary(crateInfo, registry, state) {
  const registryVersion = state === "exists" ? crateInfo.version : state;
  const decision =
    state === "exists" ? "skip" : state === "missing" ? "publish" : "abort";

  return [
    `[publish-check]`,
    `crate=${crateInfo.name}`,
    `registry=${registry}`,
    `local=${crateInfo.version}`,
    `registry_version=${registryVersion}`,
    `decision=${decision}`,
  ].join(" ");
}

function verifyCrateVersionInRegistry(crateInfo, registry, token) {
  const result = runCommandCapture(
    "cargo",
    ["info", "--registry", registry, `${crateInfo.name}@${crateInfo.version}`],
    {
      env: token
        ? {
            [registryTokenEnvName(registry)]: token,
          }
        : {},
    },
  );

  return {
    state: classifyRegistryInfoResult(result),
    result,
  };
}

function buildPlan({ kellnrToken, cratesIoToken }) {
  const crates = workspaceCrates();
  const registries = configuredRegistries({ kellnrToken, cratesIoToken });
  const releaseCrates = [];

  for (const crateInfo of crates) {
    let shouldRelease = false;

    for (const { registry, token } of registries) {
      const verification = verifyCrateVersionInRegistry(crateInfo, registry, token);
      console.log(formatVerificationSummary(crateInfo, registry, verification.state));

      if (verification.state === "error") {
        throw new Error(formatVerificationError(crateInfo, registry, verification.result));
      }

      if (verification.state === "missing") {
        shouldRelease = true;
      }
    }

    if (shouldRelease) {
      releaseCrates.push(crateInfo.crate);
    }
  }

  return {
    hasTestCrates: releaseCrates.length > 0,
    hasPublishCrates: releaseCrates.length > 0,
    testCrates: releaseCrates,
    publishCrates: releaseCrates,
  };
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

function inspectRegistry(crates, { registry, token }) {
  const workspaceByDir = new Map(workspaceCrates().map((crate) => [crate.crate, crate]));

  for (const crate of crates) {
    const crateInfo = workspaceByDir.get(crate);
    if (!crateInfo) {
      throw new Error(`Unknown workspace crate: ${crate}`);
    }

    const verification = verifyCrateVersionInRegistry(crateInfo, registry, token);
    console.log(formatVerificationSummary(crateInfo, registry, verification.state));

    if (verification.state === "error") {
      console.error(formatVerificationError(crateInfo, registry, verification.result));
    }
  }
}

function runInspectPublish(crates, { kellnrToken, cratesIoToken }) {
  inspectRegistry(crates, {
    registry: "crates-io",
    token: cratesIoToken,
  });
  inspectRegistry(crates, {
    registry: "kellnr",
    token: kellnrToken,
  });
}

function runPublishToRegistry(crates, { registry, token }) {
  if (!token) {
    throw new Error(`Expected token for registry: ${registry}`);
  }

  const workspaceByDir = new Map(workspaceCrates().map((crate) => [crate.crate, crate]));

  for (const crate of crates) {
    const crateInfo = workspaceByDir.get(crate);
    if (!crateInfo) {
      throw new Error(`Unknown workspace crate: ${crate}`);
    }

    const verification = verifyCrateVersionInRegistry(crateInfo, registry, token);
    console.log(formatVerificationSummary(crateInfo, registry, verification.state));

    if (verification.state === "exists") {
      continue;
    }

    if (verification.state === "error") {
      throw new Error(formatVerificationError(crateInfo, registry, verification.result));
    }

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
      kellnrToken: args.kellnrToken,
      cratesIoToken: args.cratesIoToken,
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

  if (args.command === "inspectPublish" || args.command === "inspect-publish") {
    runInspectPublish(parseCratesJson(args.cratesJson), {
      kellnrToken: args.kellnrToken,
      cratesIoToken: args.cratesIoToken,
    });
    return;
  }

  throw new Error("Expected command: plan, test, publish, or inspectPublish");
}

export {
  buildPlan,
  classifyRegistryInfoResult,
  configuredRegistries,
  formatVerificationError,
  formatVerificationSummary,
  registryTokenEnvName,
};

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
