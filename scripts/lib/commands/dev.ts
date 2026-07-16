import { Pipeline } from "../pipeline";

import { validate } from "../pipeline/validate";
import { project } from "../pipeline/project";
import { synchronize } from "../pipeline/synchronize";
import { startVite } from "../pipeline/vite";
import { waitForVite } from "../pipeline/wait";
import { startTauri } from "../pipeline/tauri";
import { supervise } from "../pipeline/supervise";
import { createContext } from "../context";

export async function dev() {
  const ctx = createContext();

  const pipeline = new Pipeline();

  pipeline.add("Validate project", validate);
  pipeline.add("Load Project", project);
  pipeline.add("Synchronize project", synchronize);
  // pipeline.add("Start Vite", startVite);
  // pipeline.add("Wait for Vite", waitForVite);
  // pipeline.add("Launch desktop application", startTauri);
  // pipeline.add("Supervise processes", supervise);

  await pipeline.run(ctx);
}
