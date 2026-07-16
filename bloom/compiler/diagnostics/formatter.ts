import type { Diagnostic } from "./diagnostic";

const RESET = "\x1b[0m";

const RED = "\x1b[91m";
const YELLOW = "\x1b[93m";
const BLUE = "\x1b[94m";
const CYAN = "\x1b[96m";
const GRAY = "\x1b[90m";

export function format
  (diagnostic: Diagnostic)
  : string {

  let color = CYAN;
  let kind = "note";

  switch (diagnostic.level) {
    case 0:
      color = RED;
      kind = "error";
      break;

    case 1:
      color = YELLOW;
      kind = "warning";
      break;

    case 2:
      color = BLUE;
      kind = "note";
      break;
  }

  let output =
    `${color}${kind}${RESET} ${GRAY}[${diagnostic.code}]${RESET} ` +
    `${diagnostic.title}\n`

  if (diagnostic.location) {
    output +=
      `${GRAY}--> ${diagnostic.location.file}:${diagnostic.location.line}:${diagnostic.location.column}${RESET}\n\n`;
  }

  output += diagnostic.message

  if (diagnostic.hint) {

    output +=
      `\n\n${CYAN}Hint:${RESET} ${diagnostic.hint}`;

  }

  return output;

}
