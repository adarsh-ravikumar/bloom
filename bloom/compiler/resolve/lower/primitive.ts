import type { ReflectionPrimitive } from "../../ir/primitive";

export function lowerPrimitive(primitive: ReflectionPrimitive["primitive"]): ReflectionPrimitive {
  return {
    kind: "primitive",
    primitive,
  };
}
