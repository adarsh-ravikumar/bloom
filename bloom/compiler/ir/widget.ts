import type { ReflectionCommand } from "./command";
import type { ReflectionEvent } from "./event";
import type { ReflectionNode } from "./model";
import type { ReflectionType } from "./type";

export interface ReflectionWidget extends ReflectionNode {
  name: string,
  props: ReflectionType;
  renderer: ReflectionType;
  behaviour: ReflectionType;
  commands: ReflectionCommand[];
  events: ReflectionEvent[];
}
