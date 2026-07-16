import { Logger } from "./logger"
import type { CompilationResult } from "bloom/compiler/resolve/model";
import type { ProjectConfig } from "./pipeline/project";

export interface BloomContext {
  logger: Logger;
  project?: ProjectConfig,
  compileResult?: CompilationResult,
}

export function createContext(): BloomContext {
  return {
    logger: new Logger(),
  };
}
