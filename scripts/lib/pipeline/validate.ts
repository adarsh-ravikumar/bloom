import type { BloomContext } from "../context";
import { commandExists, directoryExists, fileExists } from "../fs";

const REQUIRED_FILES = [
  "package.json",
  "proj.bloom.yaml",
  "src/main.rs",
  "src/app.rs",
  "tsconfig.json",
  "vite.config.ts",
  "bloom/host/tauri/Cargo.toml",
  "bloom/host/tauri/tauri.conf.json",
];

const REQUIRED_TOOLCHAIN = [
  "bun",
  "bunx",
  "rustc",
  "cargo",
  "git"
];

const REQUIRED_DIRECTORIES = [
  "bloom",
  "bloom/client",
  "bloom/client/widgets",
  "bloom/host",
  "bloom/host/tauri",
  "src",
  "scripts"
];

export async function validate(_: BloomContext) {
  await check(directoryExists, REQUIRED_DIRECTORIES);
  await check(fileExists, REQUIRED_FILES);
  await check(commandExists, REQUIRED_TOOLCHAIN);
}

export async function check(func: (_: string) => Promise<void>, items: string[]) {
  for (const item of items) {
    await func(item);
  }
}
