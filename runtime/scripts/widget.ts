import { mkdirSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";

const name = process.argv[2];

if (!name) {
  console.error("Usage: bun run widget <name>");
  process.exit(1);
}

const dir = join("src", "widgets", name);

if (existsSync(dir)) {
  console.error(`Widget "${name}" already exists.`);
  process.exit(1);
}

mkdirSync(dir, { recursive: true });

writeFileSync(
  join(dir, `${name}.ts`),
  `import { Widget } from "@/core/widget";

export class ${pascal(name)} extends Widget {

}
`
);

writeFileSync(
  join(dir, "renderer.ts"),
  `import { Renderer } from "@/core/renderer";

export class ${pascal(name)}Renderer extends Renderer<HTMLDivElement> {
  constructor() {
    super("div");
  }
}
`
);

writeFileSync(join(dir, "style.scss"), "");
writeFileSync(
  join(dir, "index.ts"),
  `export * from "./${name}";
export * from "./renderer";
`
);

console.log(`Widget "${name}" created successfully in "src/widgets/${name}"`);

function pascal(s: string) {
  return s
    .split(/[-_ ]+/)
    .map(x => x[0].toUpperCase() + x.slice(1))
    .join("");
}
