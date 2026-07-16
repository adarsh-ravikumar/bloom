import { BloomError } from "./lib/errors";
import { dev } from "./lib/commands/dev";

try {
  await dev();
}
catch (error) {
  if (error instanceof BloomError) {
    console.error(error.message);
    process.exit(1);
  }

  throw error;
}
