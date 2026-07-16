import type { ResolverContext } from "./context";

import type { ReflectionReference, ReflectionType } from "../ir/type";

import { lookupEnum, lookupInterface } from "./lookup";

import { lowerInterface } from "./lower/interface";
import { lowerEnum } from "./lower/enum";

function visitType(ctx: ResolverContext, type: ReflectionType): void {
  switch (type.kind) {
    case "primitive":
      return;

    case "literal":
      return;

    case "reference":
      visitReference(ctx, type);
      return;

    case "array":
      visitType(ctx, type.element);
      return;

    case "nullable":
      visitType(ctx, type.type);
      return;
  }
}

function visitReference(ctx: ResolverContext, reference: ReflectionReference) {
  if (ctx.resolvedSymbols.has(reference.symbol)) return;
  ctx.resolvedSymbols.add(reference.symbol);

  const iface = lookupInterface(ctx, reference.symbol);

  if (iface) {
    const reflection = lowerInterface(ctx, iface);
    ctx.reflection.interfaces.push(reflection);

    for (const property of reflection.properties) visitType(ctx, property.type);

    return;
  }

  const enumeration = lookupEnum(ctx, reference.symbol);


  if (enumeration) {
    ctx.reflection.enums.push(lowerEnum(enumeration));
    return;
  }
}

export async function resolveSymbols(ctx: ResolverContext): Promise<void> {
  for (const widget of ctx.reflection.widgets) {
    visitType(ctx, widget.props);
    visitType(ctx, widget.renderer);
    visitType(ctx, widget.behaviour);

    for (const command of widget.commands) {
      for (const parameter of command.parameters) {
        visitType(ctx, parameter.type);
      }
    }
  }
}
