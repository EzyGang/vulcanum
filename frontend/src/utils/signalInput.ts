import type { Signal } from '@preact/signals';

export const textInputHandler = (field: Signal<string>) => (event: Event) => {
  field.value = (event.target as HTMLInputElement | HTMLTextAreaElement).value;
};
