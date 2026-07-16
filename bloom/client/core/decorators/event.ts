export function Event(): MethodDecorator {
  return (_, __, descriptor) => descriptor;
}
