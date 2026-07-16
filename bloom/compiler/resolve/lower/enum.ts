import type { EnumDeclaration, EnumMember } from "ts-morph";

import type { ReflectionEnum, ReflectionEnumValue } from "../../ir/enum";

function lowerEnumValue(member: EnumMember): ReflectionEnumValue {
  return {
    name: member.getName(),
    value: member.getValue(),
  };
}

export function lowerEnum(declaration: EnumDeclaration): ReflectionEnum {
  return {
    name: declaration.getName(),
    values: declaration
      .getMembers()
      .map(lowerEnumValue),
  };
}
