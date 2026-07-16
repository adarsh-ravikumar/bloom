import type { ReflectionNode } from "./model";

export interface ReflectionPrimitive extends ReflectionNode {
  kind: "primitive";
  primitive:
  | "string"
  | "number"
  | "boolean"
  | "void"
  | "null";
}
