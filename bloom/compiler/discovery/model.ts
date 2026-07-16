export type WidgetSource =
  | "builtin"
  | "user";

export interface DiscoveredSourceFile {
  path: string;
  source: WidgetSource;
}

export interface DiscoveredWidget {
  name: string;
  file: DiscoveredSourceFile;
}

export interface DiscoveryResult {
  widgets: DiscoveredWidget[];
}
