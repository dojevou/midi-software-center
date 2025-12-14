import { writable, derived, get } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

export type DragSource =
  | 'file-browser'
  | 'database-list'
  | 'sequencer-track'
  | 'mixer-channel'
  | 'piano-roll-note'
  | 'loop-browser'
  | 'external';

export type DropTarget =
  | 'sequencer-track'
  | 'mixer-channel'
  | 'piano-roll'
  | 'trash'
  | 'folder'
  | 'window-dock';

export interface DragData {
  type: DragDataType;
  source: DragSource;
  payload: unknown;
  preview?: HTMLElement;
}

export type DragDataType =
  | 'midi-file'
  | 'midi-files'
  | 'audio-file'
  | 'track'
  | 'clip'
  | 'note'
  | 'notes'
  | 'channel'
  | 'window';

export interface MidiFileDragPayload {
  fileId: number;
  filename: string;
  filepath: string;
  bpm?: number;
  key?: string;
}

export interface TrackDragPayload {
  trackId: number;
  channel: number;
  name: string;
}

export interface NoteDragPayload {
  noteIds: number[];
  originalPositions: { pitch: number; tick: number }[];
}

export interface DropZone {
  id: string;
  element: HTMLElement;
  accepts: DragDataType[];
  target: DropTarget;
  onDrop: (data: DragData, position: { x: number; y: number }) => void;
  onDragOver?: (data: DragData) => boolean;
  priority?: number;
}

export interface DnDState {
  isDragging: boolean;
  dragData: DragData | null;
  dropZones: Map<string, DropZone>;
  activeDropZone: string | null;
  cursorPosition: { x: number; y: number };
  dragOffset: { x: number; y: number };
}

// ============================================================================
// STORE IMPLEMENTATION
// ============================================================================

const initialState: DnDState = {
  isDragging: false,
  dragData: null,
  dropZones: new Map(),
  activeDropZone: null,
  cursorPosition: { x: 0, y: 0 },
  dragOffset: { x: 0, y: 0 },
};

function createDnDStore() {
  const { subscribe, set, update } = writable<DnDState>(initialState);

  let previewElement: HTMLElement | null = null;
  let boundMouseMove: ((e: MouseEvent) => void) | null = null;
  let boundMouseUp: ((e: MouseEvent) => void) | null = null;

  const store = {
    subscribe,

    // Register a drop zone
    registerDropZone(zone: DropZone) {
      update(state => {
        const zones = new Map(state.dropZones);
        zones.set(zone.id, zone);
        return { ...state, dropZones: zones };
      });

      return () => store.unregisterDropZone(zone.id);
    },

    // Unregister a drop zone
    unregisterDropZone(id: string) {
      update(state => {
        const zones = new Map(state.dropZones);
        zones.delete(id);
        return { ...state, dropZones: zones };
      });
    },

    // Start dragging
    startDrag(data: DragData, event: MouseEvent) {
      const offset = { x: 0, y: 0 };

      // Create preview element using safe DOM methods
      if (data.preview) {
        previewElement = data.preview.cloneNode(true) as HTMLElement;
      } else {
        previewElement = createDefaultPreview(data);
      }

      previewElement.classList.add('dnd-preview');
      previewElement.style.position = 'fixed';
      previewElement.style.pointerEvents = 'none';
      previewElement.style.zIndex = '100000';
      document.body.appendChild(previewElement);

      update(state => ({
        ...state,
        isDragging: true,
        dragData: data,
        cursorPosition: { x: event.clientX, y: event.clientY },
        dragOffset: offset,
      }));

      store.updatePreviewPosition(event.clientX, event.clientY);

      // Create bound handlers
      boundMouseMove = (e: MouseEvent) => store.handleMouseMove(e);
      boundMouseUp = (e: MouseEvent) => store.handleMouseUp(e);

      // Add document-level listeners
      document.addEventListener('mousemove', boundMouseMove);
      document.addEventListener('mouseup', boundMouseUp);
      document.body.style.cursor = 'grabbing';
      document.body.classList.add('is-dragging');
    },

    // Update drag position
    handleMouseMove(event: MouseEvent) {
      const state = get({ subscribe });
      if (!state.isDragging) {return;}

      update(s => ({ ...s, cursorPosition: { x: event.clientX, y: event.clientY } }));

      // Update preview position
      if (previewElement) {
        previewElement.style.left = `${event.clientX + 12}px`;
        previewElement.style.top = `${event.clientY + 12}px`;
      }

      // Find active drop zone
      const activeZone = findActiveDropZone(state.dropZones, state.dragData!, event);
      update(s => ({ ...s, activeDropZone: activeZone?.id || null }));

      // Update drop zone highlighting
      state.dropZones.forEach((zone, id) => {
        if (id === activeZone?.id) {
          zone.element.classList.add('drop-zone-active');
        } else {
          zone.element.classList.remove('drop-zone-active');
        }
      });
    },

    // Handle drop
    handleMouseUp(event: MouseEvent) {
      const state = get({ subscribe });
      if (!state.isDragging || !state.dragData) {return;}

      // Clean up preview
      if (previewElement) {
        previewElement.remove();
        previewElement = null;
      }

      // Execute drop if over valid zone
      if (state.activeDropZone) {
        const zone = state.dropZones.get(state.activeDropZone);
        if (zone) {
          zone.onDrop(state.dragData, { x: event.clientX, y: event.clientY });
        }
      }

      // Clean up all drop zones
      state.dropZones.forEach(zone => {
        zone.element.classList.remove('drop-zone-active');
      });

      // Remove listeners
      if (boundMouseMove) {
        document.removeEventListener('mousemove', boundMouseMove);
        boundMouseMove = null;
      }
      if (boundMouseUp) {
        document.removeEventListener('mouseup', boundMouseUp);
        boundMouseUp = null;
      }
      document.body.style.cursor = '';
      document.body.classList.remove('is-dragging');

      // Reset state
      set({ ...initialState, dropZones: state.dropZones });
    },

    updatePreviewPosition(x: number, y: number) {
      if (previewElement) {
        previewElement.style.left = `${x + 12}px`;
        previewElement.style.top = `${y + 12}px`;
      }
    },

    // Cancel drag
    cancelDrag() {
      if (previewElement) {
        previewElement.remove();
        previewElement = null;
      }

      const state = get({ subscribe });
      state.dropZones.forEach(zone => {
        zone.element.classList.remove('drop-zone-active');
      });

      if (boundMouseMove) {
        document.removeEventListener('mousemove', boundMouseMove);
        boundMouseMove = null;
      }
      if (boundMouseUp) {
        document.removeEventListener('mouseup', boundMouseUp);
        boundMouseUp = null;
      }

      document.body.style.cursor = '';
      document.body.classList.remove('is-dragging');

      set({ ...initialState, dropZones: state.dropZones });
    },
  };

  return store;
}

// Helper function - creates preview using safe DOM methods (no innerHTML)
function createDefaultPreview(data: DragData): HTMLElement {
  const el = document.createElement('div');
  el.className = 'dnd-default-preview';
  el.style.background = 'var(--bg-secondary, #2a2a2a)';
  el.style.border = '1px solid var(--accent-color, #007bff)';
  el.style.borderRadius = '4px';
  el.style.padding = '8px 12px';
  el.style.fontSize = '12px';
  el.style.color = 'var(--text-primary, #fff)';
  el.style.boxShadow = '0 4px 12px rgba(0,0,0,0.3)';
  el.style.maxWidth = '200px';
  el.style.overflow = 'hidden';
  el.style.textOverflow = 'ellipsis';
  el.style.whiteSpace = 'nowrap';
  el.style.display = 'flex';
  el.style.alignItems = 'center';
  el.style.gap = '6px';

  const icon = document.createElement('span');
  const text = document.createElement('span');

  switch (data.type) {
    case 'midi-file': {
      const filePayload = data.payload as MidiFileDragPayload;
      icon.textContent = '\uD83C\uDFB5';
      text.textContent = filePayload.filename;
      break;
    }
    case 'midi-files': {
      const files = data.payload as MidiFileDragPayload[];
      icon.textContent = '\uD83C\uDFB5';
      text.textContent = `${files.length} files`;
      break;
    }
    case 'track': {
      const trackPayload = data.payload as TrackDragPayload;
      icon.textContent = '\uD83C\uDFB8';
      text.textContent = trackPayload.name;
      break;
    }
    case 'clip':
      icon.textContent = '\uD83C\uDFAC';
      text.textContent = 'Clip';
      break;
    case 'note':
    case 'notes':
      icon.textContent = '\uD83C\uDFB9';
      text.textContent = data.type === 'note' ? 'Note' : 'Notes';
      break;
    default:
      icon.textContent = '\uD83D\uDCE6';
      text.textContent = `Dragging ${data.type}`;
  }

  el.appendChild(icon);
  el.appendChild(text);

  return el;
}

function findActiveDropZone(
  zones: Map<string, DropZone>,
  dragData: DragData,
  event: MouseEvent
): DropZone | null {
  const candidates: DropZone[] = [];

  zones.forEach(zone => {
    // Check if zone accepts this drag type
    if (!zone.accepts.includes(dragData.type)) {return;}

    // Check if cursor is over element
    const rect = zone.element.getBoundingClientRect();
    if (
      event.clientX >= rect.left &&
      event.clientX <= rect.right &&
      event.clientY >= rect.top &&
      event.clientY <= rect.bottom
    ) {
      // Check custom onDragOver validation
      if (zone.onDragOver && !zone.onDragOver(dragData)) {return;}

      candidates.push(zone);
    }
  });

  // Sort by priority (higher priority wins) or z-index
  candidates.sort((a, b) => (b.priority || 0) - (a.priority || 0));

  return candidates[0] || null;
}

export const dndStore = createDnDStore();

// Derived stores
export const isDragging = derived(dndStore, $dnd => $dnd.isDragging);
export const dragType = derived(dndStore, $dnd => $dnd.dragData?.type || null);
