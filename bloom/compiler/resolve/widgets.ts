import type { ClassDeclaration, DecoratableNode, Type } from "ts-morph";

import { lowerType } from "./lower/type";
import type { WidgetInstantiation } from "./model";

import type { ResolverContext } from "./context";
import { emit, invalidWidgetBase, invalidWidgetSpecialization } from "../diagnostics";

export function hasDecorator
  (method: DecoratableNode, name: string)
  : boolean {
  return method.getDecorator(name) !== undefined;
}

function findWidget(ctx: ResolverContext, cls: ClassDeclaration, type: Type | undefined): Type | undefined {
  if (type == undefined) {
    return;
  }

  const symbol = type.getSymbol();
  if (symbol?.getName() === "Widget") return type;

  const bases = type.getBaseTypes();
  if (bases.length !== 1) {
    emit(ctx, invalidWidgetBase(type.getText()), cls);
    return;
  }

  return findWidget(ctx, cls, bases[0]);
}

export function resolveWidgetInstantiation
  (ctx: ResolverContext, cls: ClassDeclaration)
  : WidgetInstantiation | undefined {
  const widget = findWidget(ctx, cls, cls.getType());

  if (!widget) return;

  const [props, renderer, behaviour] = widget.getTypeArguments();

  if (!props || !renderer || !behaviour) {
    emit(ctx, invalidWidgetSpecialization(widget.getText()), cls);
    return;
  }

  return {
    props: lowerType(ctx, cls, props),
    renderer: lowerType(ctx, cls, renderer),
    behaviour: lowerType(ctx, cls, behaviour),
    commands: [],
    events: [],
  };
}

export async function resolveWidgets(ctx: ResolverContext) {
  for (const cls of ctx.index.classes.values()) {
    if (!hasDecorator(cls, "Bloom")) continue;

    ctx.reflection.widgets.push({
      name: cls.getNameOrThrow(),
      ...resolveWidgetInstantiation(ctx, cls),
    });
  }
}
