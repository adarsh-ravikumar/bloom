import type { Node } from "ts-morph";
import type { ResolverContext } from "../resolve/context";

import type { Diagnostic, DiagnosticLocation } from "./diagnostic";

function location(node: Node): DiagnosticLocation {
  const source = node.getSourceFile();

  const pos =
    source.getLineAndColumnAtPos(
      node.getStart(),
    );

  return {
    file: source.getFilePath(),
    line: pos.line,
    column: pos.column,
  };
}

export function emit
  (ctx: ResolverContext, diagnostic: Diagnostic, node: Node)
  : void {
  diagnostic.location = location(node);
  ctx.diagnostics.push(diagnostic);

}
