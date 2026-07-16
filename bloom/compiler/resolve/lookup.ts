import type { ClassDeclaration, EnumDeclaration, InterfaceDeclaration } from "ts-morph";
import type { ResolverContext } from "./context";

export function lookupClass
  (ctx: ResolverContext, symbol: string)
  : ClassDeclaration | undefined {
  return ctx.index.classes.get(symbol);
}

export function lookupInterface
  (ctx: ResolverContext, symbol: string)
  : InterfaceDeclaration | undefined {
  return ctx.index.interfaces.get(symbol);
}

export function lookupEnum
  (ctx: ResolverContext, symbol: string)
  : EnumDeclaration | undefined {
  return ctx.index.enums.get(symbol);
}
