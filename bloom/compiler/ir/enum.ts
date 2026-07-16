export interface ReflectionEnum {
  name: string;
  values: ReflectionEnumValue[];
}

export interface ReflectionEnumValue {
  name: string;
  value: string | number;
}
