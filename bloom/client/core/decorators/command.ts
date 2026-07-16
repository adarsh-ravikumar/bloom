export function Command(): MethodDecorator {
  return (_, __, descriptor) => descriptor;
}
