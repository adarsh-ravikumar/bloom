import type { ResolverContext } from "./context";

export async function index(
  ctx: ResolverContext,
) {
  for (const file of ctx.sourceFiles) {
    for (const cls of file.getClasses()) {
      const name = cls.getName();
      if (!name) continue;

      ctx.index.classes.set(name, cls);
    }

    for (const iface of file.getInterfaces()) {
      const name = iface.getName();
      ctx.index.interfaces.set(name, iface);
    }

    for (const enumeration of file.getEnums()) {
      const name = enumeration.getName();
      ctx.index.enums.set(name, enumeration);
    }

    for (const alias of file.getTypeAliases()) {
      const name = alias.getName();
      ctx.index.aliases.set(name, alias);
    }
  }
}
