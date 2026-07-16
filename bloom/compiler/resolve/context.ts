import type {
  Project,
  SourceFile,
  ClassDeclaration,
  InterfaceDeclaration,
  EnumDeclaration,
  TypeAliasDeclaration,
} from "ts-morph";

import type { DiscoveryResult } from "../discovery/model";
import type { ReflectionGraph } from "../ir/model";
import type { Diagnostic } from "../diagnostics/diagnostic";

export interface ResolverIndex {
  classes: Map<string, ClassDeclaration>;
  interfaces: Map<string, InterfaceDeclaration>;
  enums: Map<string, EnumDeclaration>;
  aliases: Map<string, TypeAliasDeclaration>;
  declarations: Map<string, Node>;
}

export interface ResolverContext {
  discovery: DiscoveryResult;
  project: Project;
  sourceFiles: SourceFile[]
  index: ResolverIndex;
  reflection: ReflectionGraph;
  resolvedSymbols: Set<string>;
  diagnostics: Diagnostic[];
}
