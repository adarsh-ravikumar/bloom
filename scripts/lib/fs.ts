import { $ } from "bun";
import { BloomError } from "./errors";

export async function fileExists(path: string) {
  const result = await $`test -f ${path}`.nothrow();

  if (result.exitCode !== 0)
    throw new BloomError(
      `Missing required file '${path}'.`,
    );
}

export async function directoryExists(path: string) {
  const result = await $`test -d ${path}`.nothrow();

  if (result.exitCode !== 0)
    throw new BloomError(
      `Missing required directory '${path}'.`,
    );
}

export async function commandExists(command: string) {
  const exists = await $`command -v ${command}`.quiet();
  if (!exists)
    throw new BloomError(
      `Missing required tool '${command}'.`
    );
}
