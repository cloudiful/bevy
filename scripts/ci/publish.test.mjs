import test from "node:test";
import assert from "node:assert/strict";

import {
  classifyRegistryInfoResult,
  configuredRegistries,
  formatVerificationError,
  formatVerificationSummary,
  normalizeRequestedRegistries,
  registryRequiresToken,
  registryTokenEnvName,
} from "./publish.mjs";

test("classifyRegistryInfoResult returns exists on success", () => {
  assert.equal(
    classifyRegistryInfoResult({
      status: 0,
      stdout: "cloudiful-bevy-camera #camera helper",
      stderr: "",
    }),
    "exists",
  );
});

test("classifyRegistryInfoResult returns missing for clear not found output", () => {
  assert.equal(
    classifyRegistryInfoResult({
      status: 101,
      stdout: "",
      stderr: "error: could not find package `cloudiful-bevy-camera@0.2.0` in registry `crates-io`",
    }),
    "missing",
  );
});

test("classifyRegistryInfoResult returns error for auth failures", () => {
  assert.equal(
    classifyRegistryInfoResult({
      status: 101,
      stdout: "",
      stderr: "error: failed to query replaced source registry `crates-io`\nCaused by:\n  authentication failed",
    }),
    "error",
  );
});

test("classifyRegistryInfoResult returns error for network failures", () => {
  assert.equal(
    classifyRegistryInfoResult({
      status: 101,
      stdout: "",
      stderr: "error: failed to fetch `kellnr`\nCaused by:\n  [6] Could not resolve hostname",
    }),
    "error",
  );
});

test("formatVerificationError includes crate version and registry context", () => {
  const message = formatVerificationError(
    { name: "cloudiful-bevy-camera", version: "0.2.0" },
    "crates-io",
    {
      status: 101,
      stdout: "",
      stderr: "authentication failed",
    },
  );

  assert.match(message, /cloudiful-bevy-camera 0\.2\.0/);
  assert.match(message, /crates-io/);
  assert.match(message, /authentication failed/);
});

test("formatVerificationSummary includes local version registry state and decision", () => {
  const summary = formatVerificationSummary(
    { name: "cloudiful-bevy-camera", version: "0.2.0" },
    "crates-io",
    "missing",
  );

  assert.match(summary, /\[publish-check\]/);
  assert.match(summary, /crate=cloudiful-bevy-camera/);
  assert.match(summary, /registry=crates-io/);
  assert.match(summary, /local=0\.2\.0/);
  assert.match(summary, /registry_version=missing/);
  assert.match(summary, /decision=publish/);
});

test("registryTokenEnvName normalizes dashed registries", () => {
  assert.equal(registryTokenEnvName("crates-io"), "CARGO_REGISTRIES_CRATES_IO_TOKEN");
});

test("normalizeRequestedRegistries returns all registries by default", () => {
  assert.deepEqual(normalizeRequestedRegistries(), ["crates-io", "kellnr"]);
});

test("configuredRegistries filters to the requested registry", () => {
  assert.deepEqual(
    configuredRegistries({
      registry: "kellnr",
      kellnrToken: "secret",
      cratesIoToken: "unused",
    }),
    [{ registry: "kellnr", token: "secret" }],
  );
});

test("registryRequiresToken only requires auth for Kellnr", () => {
  assert.equal(registryRequiresToken("crates-io"), false);
  assert.equal(registryRequiresToken("kellnr"), true);
});

test("configuredRegistries allows explicit crates-io plans without a token", () => {
  assert.deepEqual(
    configuredRegistries({
      registry: "crates-io",
    }),
    [{ registry: "crates-io", token: undefined }],
  );
});

test("configuredRegistries requires a token for explicit Kellnr plans", () => {
  assert.throws(
    () =>
      configuredRegistries({
        registry: "kellnr",
      }),
    /Expected token for registry: kellnr/,
  );
});
