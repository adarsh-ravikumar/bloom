import {
  DiagnosticLevel,
  type Diagnostic,
} from "./diagnostic";

export enum DiagnosticCode {
  UnsupportedType = "BLM0001",
  UnknownReference = "BLM0002",
  DuplicateWidget = "BLM0003",
  DuplicateInterface = "BLM0004",
  CyclicInheritance = "BLM0005",
  UnsupportedGeneric = "BLM0006",
  UnsupportedDecorator = "BLM0007",
  InvalidWidgetBase = "BLM0008",
  InvalidWidgetSpecialization = "BLM0009",
}

export function unsupportedType(type: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.UnsupportedType,
    title: "Unsupported Type",
    message: `Bloom does not support the type '${type}'.`,
    hint: "Use a supported primitive, array, interface or enum.",
  };
}

export function unknownReference(symbol: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.UnknownReference,
    title: "Unknown Reference",
    message: `Unable to resolve '${symbol}'.`,
    hint: "Ensure the referenced declaration exists.",
  };
}

export function duplicateWidget(widget: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.DuplicateWidget,
    title: "Duplicate Widget",
    message: `Widget '${widget}' has already been declared.`,
  };
}

export function invalidWidgetBase(widget: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.InvalidWidgetBase,
    title: "Invalid Widget Base",
    message: `'${widget}' must inherit from Widget`,
    hint: "Widgets mut directly inherit from Widget<P, R, B>"
  };
}

export function invalidWidgetSpecialization(widget: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.InvalidWidgetSpecialization,
    title: "Invalid Widget Specialization",
    message: `Widget '${widget}' does not correctly specialize Widget.`,
    hint: "Expected Widget<Props, Renderer, Behaviour>.",
  };

}

export function duplicateInterface(iface: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.DuplicateInterface,
    title: "Duplicate Interface",
    message: `Interface '${iface}' has already been declared.`,
  };
}

export function cyclicInheritance(iface: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.CyclicInheritance,
    title: "Cyclic Inheritance",
    message: `Interface '${iface}' participates in an inheritance cycle.`,
  };
}

export function unsupportedGeneric(type: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.UnsupportedGeneric,
    title: "Unsupported Generic",
    message: `Generic type '${type}' is not supported.`,
  };
}

export function unsupportedDecorator(decorator: string): Diagnostic {
  return {
    level: DiagnosticLevel.Error,
    code: DiagnosticCode.UnsupportedDecorator,
    title: "Unsupported Decorator",
    message: `Decorator '${decorator}' cannot be reflected.`,
  };
}
