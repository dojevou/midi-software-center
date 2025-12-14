import { dndStore } from '$lib/stores/dndStore';
import type { DragData, DragDataType, DropTarget } from '$lib/stores/dndStore';

export interface DroppableOptions {
  accepts: DragDataType[];
  target: DropTarget;
  onDrop: (data: DragData, position: { x: number; y: number }) => void;
  onDragOver?: (data: DragData) => boolean;
  onDragEnter?: (data: DragData) => void;
  onDragLeave?: () => void;
  disabled?: boolean;
  priority?: number;
}

export function droppable(node: HTMLElement, options: DroppableOptions) {
  const zoneId = `drop-zone-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

  let unregister: (() => void) | null = null;

  function register() {
    if (options.disabled) {return;}

    unregister = dndStore.registerDropZone({
      id: zoneId,
      element: node,
      accepts: options.accepts,
      target: options.target,
      onDrop: options.onDrop,
      onDragOver: options.onDragOver,
      priority: options.priority,
    });
  }

  register();

  return {
    update(newOptions: DroppableOptions) {
      unregister?.();
      options = newOptions;
      register();
    },
    destroy() {
      unregister?.();
    },
  };
}
