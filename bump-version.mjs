import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const version = process.argv[2]?.trim();

if (!version || !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("Usage: node bump-version.mjs <version>");
  console.error("Example: node bump-version.mjs 0.1.1");
  process.exit(1);
}

const root = process.cwd();

updateJson("package.json", (json) => {
  json.version = version;
});

updateJson("src-tauri/tauri.conf.json", (json) => {
  json.version = version;
});

updateTomlPackageVersion("src-tauri/Cargo.toml", "esnip");
updateTomlPackageVersion("src-tauri/Cargo.lock", "esnip");

console.log(`Version bumped to ${version}`);

function updateJson(relativePath, mutator) {
  const path = resolve(root, relativePath);
  const json = JSON.parse(readFileSync(path, "utf8"));
  mutator(json);
  writeFileSync(path, `${JSON.stringify(json, null, 2)}\n`);
}

function updateTomlPackageVersion(relativePath, packageName) {
  const path = resolve(root, relativePath);
  const text = readFileSync(path, "utf8");
  const result = relativePath.endsWith("Cargo.lock")
    ? replaceCargoLockPackageVersion(text, packageName)
    : replaceCargoTomlPackageVersion(text, packageName);

  if (!result.found) {
    throw new Error(`Failed to update ${relativePath}: package ${packageName} not found`);
  }

  writeFileSync(path, result.text);
}

function replaceCargoTomlPackageVersion(text, packageName) {
  let found = false;
  const nextText = text.replace(/(^\[package\]\r?\n[\s\S]*?)(?=^\[|(?![\s\S]))/m, (block) => {
    if (!block.match(new RegExp(`^name\\s*=\\s*"${escapeRegExp(packageName)}"\\s*$`, "m"))) {
      return block;
    }

    found = true;
    return replaceVersionLine(block);
  });

  return { text: nextText, found };
}

function replaceCargoLockPackageVersion(text, packageName) {
  let found = false;
  const nextText = text.replace(/(^\[\[package\]\]\r?\n[\s\S]*?)(?=^\[\[package\]\]|(?![\s\S]))/gm, (block) => {
    if (!block.match(new RegExp(`^name\\s*=\\s*"${escapeRegExp(packageName)}"\\s*$`, "m"))) {
      return block;
    }

    found = true;
    return replaceVersionLine(block);
  });

  return { text: nextText, found };
}

function replaceVersionLine(block) {
  return block.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
