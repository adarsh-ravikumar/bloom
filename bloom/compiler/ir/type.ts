import type { ReflectionNode } from "./model";
import type { ReflectionPrimitive } from "./primitive";

export type ReflectionType =
  | ReflectionPrimitive
  | ReflectionReference
  | ReflectionArray
  | ReflectionNullable
  | ReflectionLiteralUnion
  | ReflectionError;

export interface ReflectionError {
  kind: "error";
}

export interface ReflectionReference extends ReflectionNode {
  kind: "reference";
  symbol: string;
}

export interface ReflectionArray extends ReflectionNode {
  kind: "array";
  element: ReflectionType;
}

export interface ReflectionNullable extends ReflectionNode {
  kind: "nullable";
  type: ReflectionType;
}

export interface ReflectionLiteralUnion {
  kind: "literal";
  values: (string | number | boolean)[];
}

