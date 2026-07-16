import type { ReflectionNode } from "./model";
import type { ReflectionType } from "./type";

export interface ReflectionProperty extends ReflectionNode {
  type: ReflectionType;
  name: string;
  optional: boolean;
}
