import type { ReflectionError } from "../../ir/type";

export function lowerError(): ReflectionError {
  return {
    kind: "error",
  };
}
