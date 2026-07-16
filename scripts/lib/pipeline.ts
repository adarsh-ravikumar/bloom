import type { BloomContext } from "./context";

export interface Stage {
  readonly name: string;
  readonly run: (ctx: BloomContext) => Promise<void>;
}

export class Pipeline {
  private readonly stages: Stage[] = [];

  public add(
    name: string,
    run: Stage["run"],
  ) {
    this.stages.push({
      name,
      run,
    });
  }

  public async run(ctx: BloomContext) {
    for (const stage of this.stages) {
      using task =
        ctx.logger.bloom.task(stage.name);

      await stage.run(ctx);
    }
  }
}
