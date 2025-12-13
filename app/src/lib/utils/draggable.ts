import { dndStore } from '$lib/stores/dndStore';
import type { DragData } from '$lib/stores/dndStore';

export interface DraggableOptions {
  data: () => DragData;
  disabled?: boolean;
  handle?: string; // CSS selector for drag handle
  onDragStart?: () => void;
  onDragEnd?: () => void;
}

export function draggable(node: HTMLElement, options: DraggableOptions) {
  function getHandle(): HTMLElement {
    if (options.handle) {
      return node.querySelector(options.handle) || node;
    }
    return node;
  }

  function handleMouseDown(event: MouseEvent) {
    if (options.disabled) return;
    if (event.button !== 0) return; // Only left click

    // Check if clicked on handle
    const handle = getHandle();
    if (handle !== node && !handle.contains(event.target as Node)) return;

    event.preventDefault();

    const data = options.data();
    options.onDragStart?.();
    dndStore.startDrag(data, event);

    // One-time cleanup handler
    const cleanup = () => {
      options.onDragEnd?.();
      document.removeEventListener('mouseup', cleanup);
    };
    document.addEventListener('mouseup', cleanup);
  }

  const handle = getHandle();
  handle.style.cursor = options.disabled ? 'default' : 'grab';
  handle.addEventListener('mousedown', handleMouseDown);

  return {
    update(newOptions: DraggableOptions) {
      options = newOptions;
      const h = getHandle();
      h.style.cursor = options.disabled ? 'default' : 'grab';
    },
    destroy() {
      const h = getHandle();
      h.removeEventListener('mousedown', handleMouseDown);
    },
  };
}
