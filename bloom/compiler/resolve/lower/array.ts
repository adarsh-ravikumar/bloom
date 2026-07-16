import type { Node, Type } from "ts-morph";

import type { ReflectionArray, ReflectionError } from "../../ir/type";

import { lowerType } from "./type";
import type { ResolverContext } from "../context";
import { emit, unsupportedType } from "bloom/compiler/diagnostics";
import { lowerError } from "./error";


export function lowerArray(ctx: ResolverContext, node: Node, type: Type): ReflectionArray | ReflectionError {
  const element = type.getArrayElementType();

  if (!element) {
    emit(ctx, unsupportedType(type.getText()), node);
    return lowerError();
  }

  return {
    kind: "array",
    element: lowerType(ctx, node, element),
  };
}
