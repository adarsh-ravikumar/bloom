export interface WidgetOptions {
  displayName?: string;
  category?: string;
  icon?: string;
}

export function Bloom(
  options: WidgetOptions = {},
): ClassDecorator {
  return (_target) => {
    void options;
  };
}
