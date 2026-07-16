import type { Node, Type } from "ts-morph";

import type { ReflectionError, ReflectionReference } from "../../ir/type";
import { emit, unknownReference } from "bloom/compiler/diagnostics";
import type { ResolverContext } from "../context";
import { lowerError } from "./error";

export function lowerReference
  (ctx: ResolverContext, node: Node, type: Type)
  : ReflectionReference | ReflectionError {
  const symbol = type.getSymbol();


  if (!symbol) {
    emit(ctx, unknownReference(type.getText()), node);
    return lowerError();
  }

  return {
    kind: "reference",
    symbol: symbol.getName(),
  };
}
