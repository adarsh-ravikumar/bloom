import type { ReflectionNode } from "./model";
import type { ReflectionProperty } from "./property";

export interface ReflectionInterface extends ReflectionNode {
  name: string;
  properties: ReflectionProperty[];
}
