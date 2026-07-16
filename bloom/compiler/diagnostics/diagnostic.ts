export enum DiagnosticLevel {
  Error,
  Warning,
  Note,
}

export interface DiagnosticLocation {
  file: string;
  line: number;
  column: number;
}

export interface Diagnostic {
  level: DiagnosticLevel;
  code: string;
  title: string;
  message: string;
  hint?: string;
  location?: DiagnosticLocation;
}
