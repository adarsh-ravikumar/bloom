import type { ReflectionLiteralUnion } from "../../ir/type";

import type { Type } from "ts-morph";

function lowerLiteral
  (type: Type)
  : string | number | boolean {

  if (type.isStringLiteral()) return type.getLiteralValue() as string;
  if (type.isNumberLiteral()) return type.getLiteralValue() as number;
  if (type.isBooleanLiteral()) return type.getText() == "true";

  throw new Error(
    "Invalid literal.",
  );
}

export function lowerLiteralUnion
  (types: Type[])
  : ReflectionLiteralUnion {
  return {
    kind: "literal",
    values: types.map((v) => lowerLiteral(v))
  }
}

