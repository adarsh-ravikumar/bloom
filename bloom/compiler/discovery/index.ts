import { discoverWidgets } from "./widgets";
import type { DiscoveryResult } from "./model";
import type { ProjectConfig } from "scripts/lib/pipeline/project";

export async function discover(config: ProjectConfig): Promise<DiscoveryResult> {
  const widgets = await discoverWidgets(config.widgets.builtin, config.widgets.user);
  return { widgets };
}
