import { type InterfaceDeclaration, type PropertySignature, ts } from "ts-morph";

import type { ReflectionInterface } from "../../ir/interface";
import { lowerProperty } from "./property";
import type { ResolverContext } from "../context";

export function lowerInterface(ctx: ResolverContext, declaration: InterfaceDeclaration): ReflectionInterface {
  return {
    name: declaration.getName(),
    properties:
      [...flattenProperties(declaration).values()]
        .map((val) => lowerProperty(ctx, val)),
  };
}

function flattenProperties
  (declaration: InterfaceDeclaration, visited = new Set<string>())
  : Map<string, PropertySignature> {

  const name = declaration.getName();
  if (visited.has(name))
    return new Map();

  visited.add(name);

  const properties = new Map<string, PropertySignature>();

  for (const base of declaration.getBaseDeclarations()) {
    if (!base.isKind(ts.SyntaxKind.InterfaceDeclaration))
      continue;

    for (const [name, property] of flattenProperties(base, visited)) {
      properties.set(name, property);
    }

  }

  for (const property of declaration.getProperties()) {
    properties.set(property.getName(), property);
  }

  return properties;
}
