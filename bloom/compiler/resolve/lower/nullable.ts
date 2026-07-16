import type { ReflectionNullable } from "../../ir/type";

import { lowerType } from "./type";

import type { ResolverContext } from "../context";

import type { Node, Type } from "ts-morph";

export function lowerNullable
  (ctx: ResolverContext, node: Node, type: Type)
  : ReflectionNullable {

  const [a, b] =
    type.getUnionTypes();

  const inner =
    a.isNull() || a.isUndefined()
      ? b
      : a;

  return {
    kind: "nullable",
    type: lowerType(ctx, node, inner!),
  };
}
