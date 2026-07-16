import { Project } from "ts-morph";

import type { DiscoveryResult } from "../discovery/model";

import type { ResolverContext } from "./context";

import { parse } from "./parse";
import { index } from "./indexer";
import { resolveWidgets } from "./widgets";
import { resolveSymbols } from "./symbols";

import type { CompilationResult } from "./model";
import { resolveCommands } from "./commands";
import { resolveEvents } from "./events";

export async function buildReflectionGraph(
  discovery: DiscoveryResult,
): Promise<CompilationResult> {

  const ctx: ResolverContext = {
    discovery,
    project: undefined as unknown as Project,
    sourceFiles: [],
    index: {
      classes: new Map(),
      interfaces: new Map(),
      enums: new Map(),
      aliases: new Map(),
      declarations: new Map(),
    },
    reflection: {
      widgets: [],
      interfaces: [],
      enums: [],
    },
    resolvedSymbols: new Set(),
    diagnostics: []
  };

  await parse(ctx);
  await index(ctx);

  await resolveWidgets(ctx);

  await resolveCommands(ctx);
  await resolveEvents(ctx);

  await resolveSymbols(ctx);

  return {
    reflection: ctx.reflection,
    diagnostics: ctx.diagnostics
  };
}
