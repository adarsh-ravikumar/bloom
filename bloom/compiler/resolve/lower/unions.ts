import type { Node, Type } from "ts-morph";
import type { ResolverContext } from "../context";
import type { ReflectionError, ReflectionLiteralUnion, ReflectionNullable } from "bloom/compiler/ir/type";
import { lowerNullable } from "./nullable";
import { lowerLiteralUnion } from "./literal";
import { emit, unsupportedType } from "bloom/compiler/diagnostics";
import { lowerError } from "./error";

export function isNullable(type: Type): boolean {
  if (!type.isUnion())
    return false;

  const members = type.getUnionTypes();

  return (
    members.length === 2 &&
    members.some(t => t.isNull() || t.isUndefined())
  );
}

export function isLiteral(type: Type): boolean {
  return (
    type.isStringLiteral() ||
    type.isNumberLiteral() ||
    type.isBooleanLiteral()
  );
}

export function isLiteralUnion(type: Type): boolean {
  if (!type.isUnion()) return false;

  const members = type.getUnionTypes();

  return !(
    members.some(t => !isLiteral(t))
  );
}

export function lowerUnion
  (ctx: ResolverContext, node: Node, type: Type)
  : ReflectionNullable | ReflectionLiteralUnion | ReflectionError {

  if (isNullable(type)) return lowerNullable(ctx, node, type);
  if (isLiteralUnion(type)) return lowerLiteralUnion(type.getUnionTypes());
  if (isLiteral(type)) return lowerLiteralUnion([type]);

  emit(ctx, unsupportedType(type.getText()), node);
  return lowerError();
}
