#!/usr/bin/env bun

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import {
  intro,
  outro,
  text,
  select,
  isCancel,
  cancel
} from "@clack/prompts";

import color from "picocolors";

intro(color.magenta("Bloom Widget Generator"));

const name = await text({
  message: "Widget name",
  placeholder: "button",
});

if (isCancel(name)) {
  cancel("Cancelled.");
  process.exit(0);
}

const kind = await select({
  message: "Widget type",
  options: [
    {
      value: "primitive",
      label: "Primitive Widget",
      hint: "Owns a View + Behaviour"
    },
    {
      value: "derived",
      label: "Derived Primitive",
      hint: "Extends an existing primitive"
    },
    {
      value: "composite",
      label: "Composite Widget",
      hint: "Composes existing widgets"
    }
  ]
});

if (isCancel(kind)) {
  cancel("Cancelled.");
  process.exit(0);
}

let baseWidget = "";

if (kind === "composite" || kind == "derived") {
  const result = await text({
    message: "Base widget",
    placeholder: "Container"
  });

  if (isCancel(result)) {
    cancel("Cancelled.");
    process.exit(0);
  }

  baseWidget = result;
}

const widgetName = name.toString();
const className = pascal(widgetName);

const dir = join("src", "widgets", widgetName);

if (existsSync(dir)) {
  console.error(`Widget "${widgetName}" already exists.`);
  process.exit(1);
}

mkdirSync(dir);

switch (kind) {
  case "primitive":
    createPrimitive(widgetName, className, dir);
    break;

  case "derived":
    createDerivedPrimitive(
      widgetName,
      className,
      baseWidget,
      dir
    );
    break;

  case "composite":
    createComposite(
      widgetName,
      className,
      baseWidget,
      dir
    );
    break;
}

outro(color.green(`Created ${widgetName}`));

function createPrimitive(
  name: string,
  className: string,
  dir: string
) {
  writeFileSync(
    join(dir, `${name}.ts`),
    `import { Widget } from "@/core/widget";
import { ${className}View } from "./view";
import { ${className}Behaviour } from "./behaviour";

export class ${className} extends Widget<${className}View, ${className}Behaviour> {
  constructor() {
    super(${className}View, ${className}Behaviour);
  }

  public override compose(): void {
    super.compose();
  }

  public override update(): void {
    super.update();
  }
}
`);

  writeFileSync(
    join(dir, "view.ts"),
    `import "./style.scss";
import { View } from "@/core/view";

export class ${className}View extends View<HTMLDivElement> {
  constructor() {
    super("div");
  }
}
`);

  writeFileSync(
    join(dir, "behaviour.ts"),
    `import { Behaviour, type BehaviourHandler } from "@/core/behaviour";

export class ${className}Behaviour extends Behaviour {
  constructor() {
    super();
  }

  public onClick(
    handler: BehaviourHandler<"click", HTMLDivElement>
  ): void {
    this.addEvent("click", handler);
  }
}
`);

  writeFileSync(join(dir, "style.scss"), "");

  createIndex(name, dir);
}

function createComposite(
  name: string,
  className: string,
  base: string,
  dir: string
) {
  const baseClass = pascal(base);

  writeFileSync(
    join(dir, `${name}.ts`),
    `import { ${baseClass} } from "@/widgets/${base.toLowerCase()}";

export class ${className} extends ${baseClass} {
  constructor() {
    super();
  }

  public override compose(): void {
    super.compose();
  }

  public override update(): void {
    super.update();
  }
}
`);

  createIndex(name, dir);
}

function createDerivedPrimitive(
  name: string,
  className: string,
  base: string,
  dir: string
) {
  const baseClass = pascal(base);

  writeFileSync(
    join(dir, `${name}.ts`),
    `import { Widget } from "@/core/widget";
import { ${className}View } from "./view";
import { ${className}Behaviour } from "./behaviour";

export class ${className} extends Widget<${className}View, ${className}Behaviour> {
  constructor() {
    super(${className}View, ${className}Behaviour);
  }

  public override compose(): void {
    super.compose();
  }

  public override update(): void {
    super.update();
  }
}
`);

  writeFileSync(
    join(dir, "view.ts"),
    `import "./style.scss";
import { ${baseClass}View } from "@/widgets/${base}/view";

export class ${className}View extends ${baseClass}View {
  constructor() {
    super();
  }
}
`);

  writeFileSync(
    join(dir, "behaviour.ts"),
    `import { ${baseClass}Behaviour } from "@/widgets/${base}/behaviour";

export class ${className}Behaviour extends ${baseClass}Behaviour {
  constructor() {
    super();
  }
}
`);

  writeFileSync(
    join(dir, "style.scss"),
    `/* ${className} styles */`
  );

  createIndex(name, dir);
}

function createIndex(
  name: string,
  dir: string
) {
  writeFileSync(
    join(dir, "index.ts"),
    `export * from "./${name}";
`);
}

function pascal(str: string) {
  return str
    .split(/[-_ ]+/)
    .map(x => x.charAt(0).toUpperCase() + x.slice(1))
    .join("");
}
