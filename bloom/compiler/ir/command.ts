import type { ReflectionProperty } from "./property";

export interface ReflectionCommand {
  name: string;
  parameters: ReflectionProperty[];
}
