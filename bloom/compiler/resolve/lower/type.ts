import type { Node, Type } from "ts-morph";

import type { ReflectionType } from "../../ir/type";

import { lowerPrimitive } from "./primitive";
import { lowerReference } from "./reference";
import { lowerArray } from "./array";
import type { ResolverContext } from "../context";
import { isLiteral, lowerUnion } from "./unions";

export function lowerType(ctx: ResolverContext, node: Node, type: Type): ReflectionType {
  if (type.isString()) return lowerPrimitive("string");
  if (type.isNumber()) return lowerPrimitive("number");
  if (type.isBoolean()) return lowerPrimitive("boolean");
  if (type.isNull()) return lowerPrimitive("null");
  if (type.isVoid()) return lowerPrimitive("void");

  if (type.isEnum()) return lowerReference(ctx, node, type);

  if (type.isArray()) return lowerArray(ctx, node, type);
  if (type.isUnion() || isLiteral(type)) return lowerUnion(ctx, node, type)


  return lowerReference(ctx, node, type);
}

