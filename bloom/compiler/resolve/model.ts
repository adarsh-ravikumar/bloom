import type { Diagnostic } from "../diagnostics";
import type { ReflectionCommand } from "../ir/command";
import type { ReflectionEvent } from "../ir/event";
import type { ReflectionGraph } from "../ir/model";
import type { ReflectionType } from "../ir/type";

export interface WidgetInstantiation {
  props: ReflectionType;
  renderer: ReflectionType;
  behaviour: ReflectionType;
  commands: ReflectionCommand[];
  events: ReflectionEvent[];
}

export interface CompilationResult {
  reflection: ReflectionGraph;
  diagnostics: Diagnostic[];
}
