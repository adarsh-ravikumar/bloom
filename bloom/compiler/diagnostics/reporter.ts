import type { LogChannel } from "scripts/lib/logger";
import { DiagnosticLevel } from "./diagnostic";
import type { Diagnostic } from "./diagnostic";

import { format } from "./formatter";

export function report
  (channel: LogChannel, diagnostics: Diagnostic[])
  : void {

  for (const diagnostic of diagnostics) {
    const message = format(diagnostic);

    switch (diagnostic.level) {
      case DiagnosticLevel.Error:
        channel.raw(message);
        break;

      case DiagnosticLevel.Warning:
        channel.raw(message);
        break;

      case DiagnosticLevel.Note:
        channel.raw(message);
        break;
    }
  }
}
