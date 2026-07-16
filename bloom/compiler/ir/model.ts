import type { ReflectionWidget } from "./widget";
import type { ReflectionInterface } from "./interface";
import type { ReflectionEnum } from "./enum";

export interface ReflectionNode { };

export interface ReflectionGraph {
  widgets: ReflectionWidget[];
  interfaces: ReflectionInterface[];
  enums: ReflectionEnum[];
}
