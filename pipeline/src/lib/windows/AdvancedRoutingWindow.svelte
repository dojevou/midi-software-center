<script lang="ts">
  // ============================================================================
  // Advanced Routing Window - Visual MIDI/Audio/CV Routing System
  // ============================================================================
  // Features:
  // - Visual routing diagram with drag-and-drop connections
  // - Matrix view alternative for quick routing
  // - Advanced filtering and transformation per route
  // - Multi-output routing with velocity/channel splitting
  // - Real-time visual feedback during playback
  // - Save/load routing presets
  // ============================================================================

  import { onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import type { Writable, Readable } from 'svelte/store';

  // ============================================================================
  // Type Definitions
  // ============================================================================

  export type SignalType = 'MIDI' | 'Audio' | 'CV';
  export type MessageType = 'NoteOn' | 'NoteOff' | 'CC' | 'PitchBend' | 'AfterTouch' | 'ProgramChange';

  export interface Node {
    id: string;
    name: string;
    type: 'track' | 'instrument' | 'output';
    signalType: SignalType;
    position: { x: number; y: number };
    color: string;
    isActive: boolean;
  }

  export interface RouteFilter {
    messageTypes: MessageType[];
    channelRange: { min: number; max: number };
    velocityRange: { min: number; max: number };
    noteRange: { min: number; max: number };
  }

  export interface RouteTransform {
    transpose: number;
    velocityScale: number;
    channelRemap: number | null;
  }

  export interface Route {
    id: string;
    sourceId: string;
    destinationId: string;
    enabled: boolean;
    filter: RouteFilter;
    transform: RouteTransform;
    isActive: boolean; // Real-time activity indicator
  }

  export interface RoutingPreset {
    name: string;
    description: string;
    routes: Route[];
    createdAt: string;
  }

  // ============================================================================
  // Props
  // ============================================================================

  export let visible: boolean = true;
  export let onClose: (() => void) | undefined = undefined;

  // ============================================================================
  // Constants
  // ============================================================================

  const SIGNAL_COLORS = {
    MIDI: '#a855f7', // Purple
    Audio: '#22c55e', // Green
    CV: '#f97316', // Orange
  } as const;

  const NODE_WIDTH = 120;
  const NODE_HEIGHT = 60;
  const CANVAS_PADDING = 50;
  const GRID_SIZE = 20;

  const DEFAULT_FILTER: RouteFilter = {
    messageTypes: ['NoteOn', 'NoteOff', 'CC', 'PitchBend'],
    channelRange: { min: 1, max: 16 },
    velocityRange: { min: 0, max: 127 },
    noteRange: { min: 0, max: 127 },
  };

  const DEFAULT_TRANSFORM: RouteTransform = {
    transpose: 0,
    velocityScale: 1.0,
    channelRemap: null,
  };

  // ============================================================================
  // State Management
  // ============================================================================

  let viewMode: 'diagram' | 'matrix' = 'diagram';
  let selectedRoute: Route | null = null;
  let hoveredNodeId: string | null = null;
  let isDraggingConnection = false;
  let dragStartNodeId: string | null = null;
  let dragEndPosition: { x: number; y: number } | null = null;
  let showPresetDialog = false;
  let presetName = '';
  let presetDescription = '';

  // Canvas references
  let canvasContainer: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let canvasWidth = 1200;
  let canvasHeight = 800;

  // Stores
  const nodes: Writable<Node[]> = writable([]);
  const routes: Writable<Route[]> = writable([]);
  const presets: Writable<RoutingPreset[]> = writable([]);

  // Derived stores
  const trackNodes: Readable<Node[]> = derived(nodes, $nodes =>
    $nodes.filter(n => n.type === 'track')
  );

  const instrumentNodes: Readable<Node[]> = derived(nodes, $nodes =>
    $nodes.filter(n => n.type === 'instrument')
  );

  const outputNodes: Readable<Node[]> = derived(nodes, $nodes =>
    $nodes.filter(n => n.type === 'output')
  );

  const destinationNodes: Readable<Node[]> = derived(nodes, $nodes =>
    $nodes.filter(n => n.type === 'instrument' || n.type === 'output')
  );

  const activeRoutes: Readable<Route[]> = derived(routes, $routes =>
    $routes.filter(r => r.enabled && r.isActive)
  );

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    initializeCanvas();
    loadSampleData();
    loadPresetsFromStorage();
    startRenderLoop();
    window.addEventListener('resize', handleResize);
  });

  onDestroy(() => {
    window.removeEventListener('resize', handleResize);
  });

  // ============================================================================
  // Initialization
  // ============================================================================

  function initializeCanvas(): void {
    if (!canvas) return;
    ctx = canvas.getContext('2d');
    handleResize();
  }

  function handleResize(): void {
    if (!canvasContainer || !canvas) return;
    const rect = canvasContainer.getBoundingClientRect();
    canvasWidth = rect.width;
    canvasHeight = rect.height;
    canvas.width = canvasWidth;
    canvas.height = canvasHeight;
  }

  function loadSampleData(): void {
    // Create sample track nodes (left side)
    const trackCount = 8;
    const trackSpacing = (canvasHeight - 2 * CANVAS_PADDING) / (trackCount - 1);
    const sampleTracks: Node[] = Array.from({ length: trackCount }, (_, i) => ({
      id: `track-${i}`,
      name: `Track ${i + 1}`,
      type: 'track' as const,
      signalType: 'MIDI' as const,
      position: {
        x: CANVAS_PADDING,
        y: CANVAS_PADDING + i * trackSpacing,
      },
      color: SIGNAL_COLORS.MIDI,
      isActive: false,
    }));

    // Create sample instrument nodes (right side)
    const instrumentCount = 6;
    const instrumentSpacing = (canvasHeight - 2 * CANVAS_PADDING) / (instrumentCount - 1);
    const sampleInstruments: Node[] = Array.from({ length: instrumentCount }, (_, i) => ({
      id: `instrument-${i}`,
      name: `Synth ${i + 1}`,
      type: 'instrument' as const,
      signalType: i < 4 ? ('MIDI' as const) : ('Audio' as const),
      position: {
        x: canvasWidth - CANVAS_PADDING - NODE_WIDTH,
        y: CANVAS_PADDING + i * instrumentSpacing,
      },
      color: i < 4 ? SIGNAL_COLORS.MIDI : SIGNAL_COLORS.Audio,
      isActive: false,
    }));

    // Create sample output nodes
    const sampleOutputs: Node[] = [
      {
        id: 'output-master',
        name: 'Master Out',
        type: 'output',
        signalType: 'Audio',
        position: {
          x: canvasWidth - CANVAS_PADDING - NODE_WIDTH,
          y: canvasHeight - CANVAS_PADDING - NODE_HEIGHT,
        },
        color: SIGNAL_COLORS.Audio,
        isActive: false,
      },
    ];

    nodes.set([...sampleTracks, ...sampleInstruments, ...sampleOutputs]);

    // Create sample routes
    const sampleRoutes: Route[] = [
      createRoute('track-0', 'instrument-0'),
      createRoute('track-1', 'instrument-1'),
      createRoute('track-2', 'instrument-2'),
    ];

    routes.set(sampleRoutes);
  }

  function loadPresetsFromStorage(): void {
    try {
      const stored = localStorage.getItem('routing-presets');
      if (stored) {
        presets.set(JSON.parse(stored));
      }
    } catch (error) {
      console.error('Failed to load presets:', error);
    }
  }

  function savePresetsToStorage(presetsList: RoutingPreset[]): void {
    try {
      localStorage.setItem('routing-presets', JSON.stringify(presetsList));
    } catch (error) {
      console.error('Failed to save presets:', error);
    }
  }

  // ============================================================================
  // Route Management
  // ============================================================================

  function createRoute(sourceId: string, destinationId: string): Route {
    return {
      id: `route-${Date.now()}-${Math.random()}`,
      sourceId,
      destinationId,
      enabled: true,
      filter: { ...DEFAULT_FILTER },
      transform: { ...DEFAULT_TRANSFORM },
      isActive: false,
    };
  }

  function addRoute(sourceId: string, destinationId: string): void {
    // Check if route already exists
    const existing = $routes.find(
      r => r.sourceId === sourceId && r.destinationId === destinationId
    );
    if (existing) {
      console.warn('Route already exists');
      return;
    }

    const newRoute = createRoute(sourceId, destinationId);
    routes.update(r => [...r, newRoute]);
    selectedRoute = newRoute;
  }

  function removeRoute(routeId: string): void {
    routes.update(r => r.filter(route => route.id !== routeId));
    if (selectedRoute?.id === routeId) {
      selectedRoute = null;
    }
  }

  function toggleRoute(routeId: string): void {
    routes.update(r =>
      r.map(route =>
        route.id === routeId ? { ...route, enabled: !route.enabled } : route
      )
    );
  }

  function updateRouteFilter(routeId: string, filter: Partial<RouteFilter>): void {
    routes.update(r =>
      r.map(route =>
        route.id === routeId
          ? { ...route, filter: { ...route.filter, ...filter } }
          : route
      )
    );
  }

  function updateRouteTransform(routeId: string, transform: Partial<RouteTransform>): void {
    routes.update(r =>
      r.map(route =>
        route.id === routeId
          ? { ...route, transform: { ...route.transform, ...transform } }
          : route
      )
    );
  }

  function getRoutesForNode(nodeId: string): Route[] {
    return $routes.filter(r => r.sourceId === nodeId || r.destinationId === nodeId);
  }

  function getRoutesBetween(sourceId: string, destinationId: string): Route[] {
    return $routes.filter(r => r.sourceId === sourceId && r.destinationId === destinationId);
  }

  // ============================================================================
  // Canvas Rendering
  // ============================================================================

  function startRenderLoop(): void {
    function render() {
      if (viewMode === 'diagram') {
        renderDiagram();
      }
      requestAnimationFrame(render);
    }
    requestAnimationFrame(render);
  }

  function renderDiagram(): void {
    if (!ctx) return;

    // Clear canvas
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, canvasWidth, canvasHeight);

    // Draw grid
    drawGrid();

    // Draw connection lines (behind nodes)
    drawConnections();

    // Draw drag line if dragging
    if (isDraggingConnection && dragStartNodeId && dragEndPosition) {
      drawDragLine(dragStartNodeId, dragEndPosition);
    }

    // Draw nodes (on top)
    for (const node of $nodes) {
      drawNode(node, node.id === hoveredNodeId);
    }
  }

  function drawGrid(): void {
    if (!ctx) return;

    ctx.strokeStyle = '#2a2a2a';
    ctx.lineWidth = 1;

    // Vertical lines
    for (let x = 0; x < canvasWidth; x += GRID_SIZE) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvasHeight);
      ctx.stroke();
    }

    // Horizontal lines
    for (let y = 0; y < canvasHeight; y += GRID_SIZE) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(canvasWidth, y);
      ctx.stroke();
    }
  }

  function drawNode(node: Node, isHovered: boolean): void {
    if (!ctx) return;

    const { x, y } = node.position;

    // Draw node background
    ctx.fillStyle = node.isActive ? '#374151' : '#1f2937';
    ctx.strokeStyle = isHovered ? '#ffffff' : node.color;
    ctx.lineWidth = isHovered ? 3 : 2;

    ctx.beginPath();
    ctx.roundRect(x, y, NODE_WIDTH, NODE_HEIGHT, 8);
    ctx.fill();
    ctx.stroke();

    // Draw node label
    ctx.fillStyle = '#ffffff';
    ctx.font = '14px Inter, system-ui, sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(node.name, x + NODE_WIDTH / 2, y + NODE_HEIGHT / 2);

    // Draw signal type indicator
    ctx.fillStyle = node.color;
    ctx.font = '10px Inter, system-ui, sans-serif';
    ctx.fillText(node.signalType, x + NODE_WIDTH / 2, y + NODE_HEIGHT - 12);

    // Draw connection points
    const leftPoint = { x: x, y: y + NODE_HEIGHT / 2 };
    const rightPoint = { x: x + NODE_WIDTH, y: y + NODE_HEIGHT / 2 };

    if (node.type === 'track') {
      drawConnectionPoint(rightPoint, node.color);
    } else {
      drawConnectionPoint(leftPoint, node.color);
    }
  }

  function drawConnectionPoint(point: { x: number; y: number }, color: string): void {
    if (!ctx) return;

    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(point.x, point.y, 6, 0, Math.PI * 2);
    ctx.fill();
  }

  function drawConnections(): void {
    if (!ctx) return;

    for (const route of $routes) {
      const source = $nodes.find(n => n.id === route.sourceId);
      const destination = $nodes.find(n => n.id === route.destinationId);

      if (!source || !destination) continue;

      const startPoint = {
        x: source.position.x + NODE_WIDTH,
        y: source.position.y + NODE_HEIGHT / 2,
      };

      const endPoint = {
        x: destination.position.x,
        y: destination.position.y + NODE_HEIGHT / 2,
      };

      const isHighlighted =
        hoveredNodeId === source.id || hoveredNodeId === destination.id;

      drawConnection(startPoint, endPoint, route, isHighlighted);
    }
  }

  function drawConnection(
    start: { x: number; y: number },
    end: { x: number; y: number },
    route: Route,
    isHighlighted: boolean
  ): void {
    if (!ctx) return;

    const source = $nodes.find(n => n.id === route.sourceId);
    if (!source) return;

    // Determine line style
    const color = source.color;
    const alpha = route.enabled ? (route.isActive ? 1.0 : 0.6) : 0.2;
    const lineWidth = isHighlighted ? 4 : (route.isActive ? 3 : 2);

    ctx.strokeStyle = color + Math.floor(alpha * 255).toString(16).padStart(2, '0');
    ctx.lineWidth = lineWidth;

    // Draw bezier curve
    const controlPoint1X = start.x + (end.x - start.x) * 0.5;
    const controlPoint2X = start.x + (end.x - start.x) * 0.5;

    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.bezierCurveTo(
      controlPoint1X,
      start.y,
      controlPoint2X,
      end.y,
      end.x,
      end.y
    );
    ctx.stroke();

    // Draw glow effect for active routes
    if (route.isActive && route.enabled) {
      ctx.shadowBlur = 10;
      ctx.shadowColor = color;
      ctx.stroke();
      ctx.shadowBlur = 0;
    }

    // Draw arrowhead
    drawArrowhead(end, color, alpha);
  }

  function drawDragLine(
    nodeId: string,
    endPosition: { x: number; y: number }
  ): void {
    if (!ctx) return;

    const node = $nodes.find(n => n.id === nodeId);
    if (!node) return;

    const startPoint = {
      x: node.position.x + NODE_WIDTH,
      y: node.position.y + NODE_HEIGHT / 2,
    };

    ctx.strokeStyle = node.color + '80'; // 50% opacity
    ctx.lineWidth = 2;
    ctx.setLineDash([5, 5]);

    ctx.beginPath();
    ctx.moveTo(startPoint.x, startPoint.y);
    ctx.lineTo(endPosition.x, endPosition.y);
    ctx.stroke();

    ctx.setLineDash([]);
  }

  function drawArrowhead(
    point: { x: number; y: number },
    color: string,
    alpha: number
  ): void {
    if (!ctx) return;

    const size = 8;
    ctx.fillStyle = color + Math.floor(alpha * 255).toString(16).padStart(2, '0');

    ctx.beginPath();
    ctx.moveTo(point.x, point.y);
    ctx.lineTo(point.x - size, point.y - size / 2);
    ctx.lineTo(point.x - size, point.y + size / 2);
    ctx.closePath();
    ctx.fill();
  }

  // ============================================================================
  // Mouse Interaction
  // ============================================================================

  function handleCanvasMouseDown(event: MouseEvent): void {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Check if clicking on a node
    for (const node of $nodes) {
      if (isPointInNode(x, y, node)) {
        if (node.type === 'track') {
          startDraggingConnection(node.id, { x, y });
        }
        return;
      }
    }

    // Check if clicking on a connection
    for (const route of $routes) {
      if (isPointNearConnection(x, y, route)) {
        selectedRoute = route;
        return;
      }
    }

    selectedRoute = null;
  }

  function handleCanvasMouseMove(event: MouseEvent): void {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Update drag position
    if (isDraggingConnection) {
      dragEndPosition = { x, y };
    }

    // Update hovered node
    let foundHover = false;
    for (const node of $nodes) {
      if (isPointInNode(x, y, node)) {
        hoveredNodeId = node.id;
        foundHover = true;
        break;
      }
    }

    if (!foundHover) {
      hoveredNodeId = null;
    }

    // Update cursor
    if (canvas) {
      if (isDraggingConnection || hoveredNodeId) {
        canvas.style.cursor = 'pointer';
      } else {
        canvas.style.cursor = 'default';
      }
    }
  }

  function handleCanvasMouseUp(event: MouseEvent): void {
    if (!isDraggingConnection || !dragStartNodeId) return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Check if released on a destination node
    for (const node of $nodes) {
      if (
        (node.type === 'instrument' || node.type === 'output') &&
        isPointInNode(x, y, node)
      ) {
        addRoute(dragStartNodeId, node.id);
        break;
      }
    }

    stopDraggingConnection();
  }

  function handleCanvasContextMenu(event: MouseEvent): void {
    event.preventDefault();

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Check if right-clicking on a connection
    for (const route of $routes) {
      if (isPointNearConnection(x, y, route)) {
        if (confirm(`Delete route from ${getNodeName(route.sourceId)} to ${getNodeName(route.destinationId)}?`)) {
          removeRoute(route.id);
        }
        return;
      }
    }
  }

  function handleCanvasDoubleClick(event: MouseEvent): void {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Check if double-clicking on a node
    for (const node of $nodes) {
      if (isPointInNode(x, y, node)) {
        // Open node properties (could be expanded)
        console.log('Edit node properties:', node);
        return;
      }
    }
  }

  function startDraggingConnection(nodeId: string, position: { x: number; y: number }): void {
    isDraggingConnection = true;
    dragStartNodeId = nodeId;
    dragEndPosition = position;
  }

  function stopDraggingConnection(): void {
    isDraggingConnection = false;
    dragStartNodeId = null;
    dragEndPosition = null;
  }

  function isPointInNode(x: number, y: number, node: Node): boolean {
    return (
      x >= node.position.x &&
      x <= node.position.x + NODE_WIDTH &&
      y >= node.position.y &&
      y <= node.position.y + NODE_HEIGHT
    );
  }

  function isPointNearConnection(x: number, y: number, route: Route): boolean {
    const source = $nodes.find(n => n.id === route.sourceId);
    const destination = $nodes.find(n => n.id === route.destinationId);

    if (!source || !destination) return false;

    const startPoint = {
      x: source.position.x + NODE_WIDTH,
      y: source.position.y + NODE_HEIGHT / 2,
    };

    const endPoint = {
      x: destination.position.x,
      y: destination.position.y + NODE_HEIGHT / 2,
    };

    // Simple distance check (could be improved with proper bezier distance)
    const distance = distanceToLineSegment(x, y, startPoint, endPoint);
    return distance < 10;
  }

  function distanceToLineSegment(
    px: number,
    py: number,
    p1: { x: number; y: number },
    p2: { x: number; y: number }
  ): number {
    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    const lengthSquared = dx * dx + dy * dy;

    if (lengthSquared === 0) {
      return Math.sqrt((px - p1.x) ** 2 + (py - p1.y) ** 2);
    }

    let t = ((px - p1.x) * dx + (py - p1.y) * dy) / lengthSquared;
    t = Math.max(0, Math.min(1, t));

    const projX = p1.x + t * dx;
    const projY = p1.y + t * dy;

    return Math.sqrt((px - projX) ** 2 + (py - projY) ** 2);
  }

  function getNodeName(nodeId: string): string {
    const node = $nodes.find(n => n.id === nodeId);
    return node ? node.name : 'Unknown';
  }

  // ============================================================================
  // Matrix View
  // ============================================================================

  function setMatrixRoute(sourceId: string, destinationId: string): void {
    // Remove existing routes from this source
    routes.update(r => r.filter(route => route.sourceId !== sourceId));

    // Add new route
    addRoute(sourceId, destinationId);
  }

  function getMatrixRoute(sourceId: string, destinationId: string): Route | undefined {
    return $routes.find(
      r => r.sourceId === sourceId && r.destinationId === destinationId && r.enabled
    );
  }

  // ============================================================================
  // Preset Management
  // ============================================================================

  function savePreset(): void {
    if (!presetName.trim()) {
      alert('Please enter a preset name');
      return;
    }

    const preset: RoutingPreset = {
      name: presetName.trim(),
      description: presetDescription.trim(),
      routes: $routes.map(r => ({ ...r })),
      createdAt: new Date().toISOString(),
    };

    presets.update(p => [...p, preset]);
    savePresetsToStorage([...$presets, preset]);

    presetName = '';
    presetDescription = '';
    showPresetDialog = false;
  }

  function loadPreset(preset: RoutingPreset): void {
    if (confirm(`Load preset "${preset.name}"? This will replace current routing.`)) {
      routes.set(preset.routes.map(r => ({ ...r, isActive: false })));
      selectedRoute = null;
    }
  }

  function deletePreset(presetName: string): void {
    if (confirm(`Delete preset "${presetName}"?`)) {
      const filtered = $presets.filter(p => p.name !== presetName);
      presets.set(filtered);
      savePresetsToStorage(filtered);
    }
  }

  function exportPreset(preset: RoutingPreset): void {
    const json = JSON.stringify(preset, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `routing-preset-${preset.name.toLowerCase().replace(/\s+/g, '-')}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function importPreset(event: Event): void {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const preset = JSON.parse(e.target?.result as string) as RoutingPreset;
        presets.update(p => [...p, preset]);
        savePresetsToStorage([...$presets, preset]);
        alert(`Imported preset "${preset.name}"`);
      } catch (error) {
        console.error('Failed to import preset:', error);
        alert('Failed to import preset. Invalid file format.');
      }
    };
    reader.readAsText(file);
  }

  // ============================================================================
  // Utility Functions
  // ============================================================================

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }

  function simulateActivity(): void {
    // Simulate real-time activity for demo purposes
    routes.update(r =>
      r.map(route => ({
        ...route,
        isActive: route.enabled && Math.random() > 0.7,
      }))
    );

    nodes.update(n =>
      n.map(node => ({
        ...node,
        isActive: Math.random() > 0.8,
      }))
    );
  }

  // Simulate activity every 500ms for demo
  let activityInterval: number;
  onMount(() => {
    activityInterval = window.setInterval(simulateActivity, 500);
  });

  onDestroy(() => {
    if (activityInterval) {
      clearInterval(activityInterval);
    }
  });
</script>

<!-- ============================================================================
     Template
     ============================================================================ -->

{#if visible}
  <div class="advanced-routing-window">
    <!-- Header -->
    <div class="header">
      <h1 class="title">Advanced Routing</h1>
      <div class="header-controls">
        <button
          class="view-toggle"
          class:active={viewMode === 'diagram'}
          on:click={() => (viewMode = 'diagram')}
        >
          Diagram
        </button>
        <button
          class="view-toggle"
          class:active={viewMode === 'matrix'}
          on:click={() => (viewMode = 'matrix')}
        >
          Matrix
        </button>
        <button class="preset-btn" on:click={() => (showPresetDialog = true)}>
          Presets
        </button>
        <button class="close-btn" on:click={onClose}>✕</button>
      </div>
    </div>

    <!-- Main Content -->
    <div class="content">
      {#if viewMode === 'diagram'}
        <!-- Diagram View -->
        <div class="diagram-view">
          <div class="canvas-container" bind:this={canvasContainer}>
            <canvas
              bind:this={canvas}
              on:mousedown={handleCanvasMouseDown}
              on:mousemove={handleCanvasMouseMove}
              on:mouseup={handleCanvasMouseUp}
              on:contextmenu={handleCanvasContextMenu}
              on:dblclick={handleCanvasDoubleClick}
            />
          </div>

          <!-- Properties Panel -->
          {#if selectedRoute}
            <div class="properties-panel">
              <div class="properties-header">
                <h3>Route Properties</h3>
                <button
                  class="close-properties"
                  on:click={() => (selectedRoute = null)}
                >
                  ✕
                </button>
              </div>

              <div class="properties-content">
                <!-- Route Info -->
                <div class="property-group">
                  <h4>Route</h4>
                  <div class="route-info">
                    <span class="route-label">Source:</span>
                    <span class="route-value">{getNodeName(selectedRoute.sourceId)}</span>
                  </div>
                  <div class="route-info">
                    <span class="route-label">Destination:</span>
                    <span class="route-value">{getNodeName(selectedRoute.destinationId)}</span>
                  </div>
                  <div class="route-info">
                    <label class="toggle-label">
                      <input
                        type="checkbox"
                        checked={selectedRoute.enabled}
                        on:change={() => toggleRoute(selectedRoute.id)}
                      />
                      <span>Enabled</span>
                    </label>
                  </div>
                </div>

                <!-- Filter Settings -->
                <div class="property-group">
                  <h4>Filter</h4>

                  <!-- Message Types -->
                  <div class="filter-section">
                    <label>Message Types:</label>
                    <div class="checkbox-group">
                      {#each ['NoteOn', 'NoteOff', 'CC', 'PitchBend', 'AfterTouch', 'ProgramChange'] as msgType}
                        <label class="checkbox-label">
                          <input
                            type="checkbox"
                            checked={selectedRoute.filter.messageTypes.includes(msgType)}
                            on:change={(e) => {
                              const checked = e.currentTarget.checked;
                              const current = selectedRoute.filter.messageTypes;
                              const updated = checked
                                ? [...current, msgType]
                                : current.filter(t => t !== msgType);
                              updateRouteFilter(selectedRoute.id, { messageTypes: updated });
                            }}
                          />
                          <span>{msgType}</span>
                        </label>
                      {/each}
                    </div>
                  </div>

                  <!-- Channel Range -->
                  <div class="filter-section">
                    <label>Channel Range:</label>
                    <div class="range-inputs">
                      <input
                        type="number"
                        min="1"
                        max="16"
                        value={selectedRoute.filter.channelRange.min}
                        on:input={(e) =>
                          updateRouteFilter(selectedRoute.id, {
                            channelRange: {
                              ...selectedRoute.filter.channelRange,
                              min: parseInt(e.currentTarget.value),
                            },
                          })}
                      />
                      <span>to</span>
                      <input
                        type="number"
                        min="1"
                        max="16"
                        value={selectedRoute.filter.channelRange.max}
                        on:input={(e) =>
                          updateRouteFilter(selectedRoute.id, {
                            channelRange: {
                              ...selectedRoute.filter.channelRange,
                              max: parseInt(e.currentTarget.value),
                            },
                          })}
                      />
                    </div>
                  </div>

                  <!-- Velocity Range -->
                  <div class="filter-section">
                    <label>Velocity Range:</label>
                    <div class="range-inputs">
                      <input
                        type="number"
                        min="0"
                        max="127"
                        value={selectedRoute.filter.velocityRange.min}
                        on:input={(e) =>
                          updateRouteFilter(selectedRoute.id, {
                            velocityRange: {
                              ...selectedRoute.filter.velocityRange,
                              min: parseInt(e.currentTarget.value),
                            },
                          })}
                      />
                      <span>to</span>
                      <input
                        type="number"
                        min="0"
                        max="127"
                        value={selectedRoute.filter.velocityRange.max}
                        on:input={(e) =>
                          updateRouteFilter(selectedRoute.id, {
                            velocityRange: {
                              ...selectedRoute.filter.velocityRange,
                              max: parseInt(e.currentTarget.value),
                            },
                          })}
                      />
                    </div>
                  </div>

                  <!-- Note Range -->
                  <div class="filter-section">
                    <label>Note Range:</label>
                    <div class="range-inputs">
                      <input
                        type="number"
                        min="0"
                        max="127"
                        value={selectedRoute.filter.noteRange.min}
                        on:input={(e) =>
                          updateRouteFilter(selectedRoute.id, {
                            noteRange: {
                              ...selectedRoute.filter.noteRange,
                              min: parseInt(e.currentTarget.value),
                            },
                          })}
                      />
                      <span>to</span>
                      <input
                        type="number"
                        min="0"
                        max="127"
                        value={selectedRoute.filter.noteRange.max}
                        on:input={(e) =>
                          updateRouteFilter(selectedRoute.id, {
                            noteRange: {
                              ...selectedRoute.filter.noteRange,
                              max: parseInt(e.currentTarget.value),
                            },
                          })}
                      />
                    </div>
                  </div>
                </div>

                <!-- Transform Settings -->
                <div class="property-group">
                  <h4>Transform</h4>

                  <!-- Transpose -->
                  <div class="transform-section">
                    <label>Transpose (semitones):</label>
                    <input
                      type="number"
                      min="-48"
                      max="48"
                      value={selectedRoute.transform.transpose}
                      on:input={(e) =>
                        updateRouteTransform(selectedRoute.id, {
                          transpose: parseInt(e.currentTarget.value),
                        })}
                    />
                  </div>

                  <!-- Velocity Scale -->
                  <div class="transform-section">
                    <label>Velocity Scale:</label>
                    <input
                      type="number"
                      min="0"
                      max="2"
                      step="0.1"
                      value={selectedRoute.transform.velocityScale}
                      on:input={(e) =>
                        updateRouteTransform(selectedRoute.id, {
                          velocityScale: parseFloat(e.currentTarget.value),
                        })}
                    />
                  </div>

                  <!-- Channel Remap -->
                  <div class="transform-section">
                    <label>Channel Remap:</label>
                    <select
                      value={selectedRoute.transform.channelRemap ?? ''}
                      on:change={(e) =>
                        updateRouteTransform(selectedRoute.id, {
                          channelRemap: e.currentTarget.value
                            ? parseInt(e.currentTarget.value)
                            : null,
                        })}
                    >
                      <option value="">No Remap</option>
                      {#each Array.from({ length: 16 }, (_, i) => i + 1) as channel}
                        <option value={channel}>Channel {channel}</option>
                      {/each}
                    </select>
                  </div>
                </div>

                <!-- Actions -->
                <div class="property-actions">
                  <button
                    class="delete-route-btn"
                    on:click={() => {
                      removeRoute(selectedRoute.id);
                      selectedRoute = null;
                    }}
                  >
                    Delete Route
                  </button>
                </div>
              </div>
            </div>
          {/if}
        </div>
      {:else}
        <!-- Matrix View -->
        <div class="matrix-view">
          <div class="matrix-container">
            <table class="routing-matrix">
              <thead>
                <tr>
                  <th class="corner-cell">Track → Destination</th>
                  {#each $destinationNodes as dest}
                    <th class="destination-header" style="border-bottom-color: {dest.color}">
                      <div class="header-content">
                        <span class="node-name">{dest.name}</span>
                        <span class="node-type">{dest.signalType}</span>
                      </div>
                    </th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each $trackNodes as track}
                  <tr>
                    <td class="track-header" style="border-left-color: {track.color}">
                      <div class="header-content">
                        <span class="node-name">{track.name}</span>
                        <span class="node-type">{track.signalType}</span>
                      </div>
                    </td>
                    {#each $destinationNodes as dest}
                      <td class="matrix-cell">
                        {#if getMatrixRoute(track.id, dest.id)}
                          <button
                            class="matrix-button active"
                            style="background-color: {track.color}40; border-color: {track.color}"
                            on:click={() => removeRoute(getMatrixRoute(track.id, dest.id).id)}
                            title="Click to remove route"
                          >
                            ✓
                          </button>
                        {:else}
                          <button
                            class="matrix-button"
                            on:click={() => setMatrixRoute(track.id, dest.id)}
                            title="Click to create route"
                          >
                            ○
                          </button>
                        {/if}
                      </td>
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>

          <!-- Matrix Legend -->
          <div class="matrix-legend">
            <h3>Legend</h3>
            <div class="legend-item">
              <span class="legend-icon active">✓</span>
              <span>Active route (click to remove)</span>
            </div>
            <div class="legend-item">
              <span class="legend-icon">○</span>
              <span>No route (click to create)</span>
            </div>
            <div class="legend-note">
              Note: Each track can only route to one destination at a time in matrix view.
              Use diagram view for multi-output routing.
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Preset Dialog -->
    {#if showPresetDialog}
      <div class="preset-dialog-overlay" on:click={() => (showPresetDialog = false)}>
        <div class="preset-dialog" on:click|stopPropagation>
          <div class="preset-dialog-header">
            <h2>Routing Presets</h2>
            <button class="close-btn" on:click={() => (showPresetDialog = false)}>
              ✕
            </button>
          </div>

          <div class="preset-dialog-content">
            <!-- Save New Preset -->
            <div class="preset-section">
              <h3>Save Current Routing</h3>
              <div class="preset-form">
                <input
                  type="text"
                  placeholder="Preset name"
                  bind:value={presetName}
                />
                <textarea
                  placeholder="Description (optional)"
                  bind:value={presetDescription}
                  rows="3"
                />
                <button class="save-preset-btn" on:click={savePreset}>
                  Save Preset
                </button>
              </div>
            </div>

            <!-- Saved Presets -->
            <div class="preset-section">
              <h3>Saved Presets</h3>
              {#if $presets.length === 0}
                <p class="no-presets">No presets saved yet</p>
              {:else}
                <div class="preset-list">
                  {#each $presets as preset}
                    <div class="preset-item">
                      <div class="preset-info">
                        <h4>{preset.name}</h4>
                        {#if preset.description}
                          <p class="preset-description">{preset.description}</p>
                        {/if}
                        <p class="preset-meta">
                          {preset.routes.length} routes • {formatDate(preset.createdAt)}
                        </p>
                      </div>
                      <div class="preset-actions">
                        <button
                          class="preset-action-btn load"
                          on:click={() => loadPreset(preset)}
                          title="Load preset"
                        >
                          Load
                        </button>
                        <button
                          class="preset-action-btn export"
                          on:click={() => exportPreset(preset)}
                          title="Export to file"
                        >
                          Export
                        </button>
                        <button
                          class="preset-action-btn delete"
                          on:click={() => deletePreset(preset.name)}
                          title="Delete preset"
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- Import Preset -->
            <div class="preset-section">
              <h3>Import Preset</h3>
              <input
                type="file"
                accept=".json"
                on:change={importPreset}
                class="file-input"
              />
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
{/if}

<!-- ============================================================================
     Styles
     ============================================================================ -->

<style>
  .advanced-routing-window {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: #111827;
    color: #f3f4f6;
    display: flex;
    flex-direction: column;
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    z-index: 1000;
  }

  /* Header */
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    background: #1f2937;
    border-bottom: 1px solid #374151;
  }

  .title {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0;
  }

  .header-controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .view-toggle {
    padding: 0.5rem 1rem;
    background: #374151;
    color: #9ca3af;
    border: 1px solid #4b5563;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .view-toggle:hover {
    background: #4b5563;
    color: #f3f4f6;
  }

  .view-toggle.active {
    background: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }

  .preset-btn {
    padding: 0.5rem 1rem;
    background: #059669;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background 0.2s;
  }

  .preset-btn:hover {
    background: #047857;
  }

  .close-btn {
    padding: 0.5rem 0.75rem;
    background: #ef4444;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 1.25rem;
    line-height: 1;
    transition: background 0.2s;
  }

  .close-btn:hover {
    background: #dc2626;
  }

  /* Content */
  .content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* Diagram View */
  .diagram-view {
    flex: 1;
    display: flex;
    position: relative;
  }

  .canvas-container {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  /* Properties Panel */
  .properties-panel {
    width: 350px;
    background: #1f2937;
    border-left: 1px solid #374151;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .properties-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid #374151;
  }

  .properties-header h3 {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0;
  }

  .close-properties {
    padding: 0.25rem 0.5rem;
    background: transparent;
    color: #9ca3af;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1.25rem;
    line-height: 1;
    transition: all 0.2s;
  }

  .close-properties:hover {
    background: #374151;
    color: #f3f4f6;
  }

  .properties-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .property-group {
    margin-bottom: 2rem;
  }

  .property-group h4 {
    font-size: 0.875rem;
    font-weight: 600;
    text-transform: uppercase;
    color: #9ca3af;
    margin: 0 0 1rem 0;
    letter-spacing: 0.05em;
  }

  .route-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid #374151;
  }

  .route-info:last-child {
    border-bottom: none;
  }

  .route-label {
    font-size: 0.875rem;
    color: #9ca3af;
  }

  .route-value {
    font-size: 0.875rem;
    color: #f3f4f6;
    font-weight: 500;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .toggle-label input[type='checkbox'] {
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .filter-section,
  .transform-section {
    margin-bottom: 1rem;
  }

  .filter-section label,
  .transform-section label {
    display: block;
    font-size: 0.875rem;
    color: #d1d5db;
    margin-bottom: 0.5rem;
  }

  .checkbox-group {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .checkbox-label input[type='checkbox'] {
    width: 14px;
    height: 14px;
    cursor: pointer;
  }

  .range-inputs {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .range-inputs input {
    flex: 1;
    padding: 0.5rem;
    background: #374151;
    color: #f3f4f6;
    border: 1px solid #4b5563;
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .range-inputs span {
    font-size: 0.875rem;
    color: #9ca3af;
  }

  .transform-section input,
  .transform-section select {
    width: 100%;
    padding: 0.5rem;
    background: #374151;
    color: #f3f4f6;
    border: 1px solid #4b5563;
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .property-actions {
    margin-top: 2rem;
    padding-top: 1rem;
    border-top: 1px solid #374151;
  }

  .delete-route-btn {
    width: 100%;
    padding: 0.75rem;
    background: #ef4444;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background 0.2s;
  }

  .delete-route-btn:hover {
    background: #dc2626;
  }

  /* Matrix View */
  .matrix-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 2rem;
    overflow: auto;
  }

  .matrix-container {
    flex: 1;
    overflow: auto;
    margin-bottom: 2rem;
  }

  .routing-matrix {
    width: 100%;
    border-collapse: collapse;
    background: #1f2937;
  }

  .routing-matrix th,
  .routing-matrix td {
    padding: 1rem;
    text-align: center;
    border: 1px solid #374151;
  }

  .corner-cell {
    background: #111827;
    font-weight: 600;
    font-size: 0.875rem;
    text-align: left;
    color: #9ca3af;
  }

  .destination-header {
    background: #1f2937;
    font-weight: 500;
    font-size: 0.875rem;
    border-bottom-width: 3px;
  }

  .track-header {
    background: #1f2937;
    font-weight: 500;
    font-size: 0.875rem;
    text-align: left;
    border-left-width: 3px;
  }

  .header-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
  }

  .track-header .header-content {
    align-items: flex-start;
  }

  .node-name {
    font-size: 0.875rem;
    color: #f3f4f6;
  }

  .node-type {
    font-size: 0.75rem;
    color: #9ca3af;
  }

  .matrix-cell {
    background: #111827;
  }

  .matrix-button {
    width: 40px;
    height: 40px;
    background: #374151;
    color: #9ca3af;
    border: 2px solid #4b5563;
    border-radius: 50%;
    cursor: pointer;
    font-size: 1.25rem;
    line-height: 1;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .matrix-button:hover {
    background: #4b5563;
    transform: scale(1.1);
  }

  .matrix-button.active {
    background-color: rgba(168, 85, 247, 0.25);
    border-color: #a855f7;
    color: #a855f7;
  }

  .matrix-button.active:hover {
    background-color: rgba(168, 85, 247, 0.4);
  }

  .matrix-legend {
    background: #1f2937;
    padding: 1.5rem;
    border-radius: 8px;
    border: 1px solid #374151;
  }

  .matrix-legend h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.75rem;
    font-size: 0.875rem;
  }

  .legend-icon {
    width: 40px;
    height: 40px;
    background: #374151;
    color: #9ca3af;
    border: 2px solid #4b5563;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.25rem;
  }

  .legend-icon.active {
    background-color: rgba(168, 85, 247, 0.25);
    border-color: #a855f7;
    color: #a855f7;
  }

  .legend-note {
    margin-top: 1rem;
    padding: 0.75rem;
    background: #374151;
    border-left: 3px solid #f59e0b;
    font-size: 0.875rem;
    color: #d1d5db;
    border-radius: 4px;
  }

  /* Preset Dialog */
  .preset-dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }

  .preset-dialog {
    width: 90%;
    max-width: 700px;
    max-height: 85vh;
    background: #1f2937;
    border-radius: 12px;
    border: 1px solid #374151;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .preset-dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid #374151;
  }

  .preset-dialog-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0;
  }

  .preset-dialog-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .preset-section {
    margin-bottom: 2rem;
  }

  .preset-section:last-child {
    margin-bottom: 0;
  }

  .preset-section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
    color: #9ca3af;
  }

  .preset-form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .preset-form input,
  .preset-form textarea {
    padding: 0.75rem;
    background: #374151;
    color: #f3f4f6;
    border: 1px solid #4b5563;
    border-radius: 6px;
    font-size: 0.875rem;
    font-family: inherit;
  }

  .save-preset-btn {
    padding: 0.75rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background 0.2s;
  }

  .save-preset-btn:hover {
    background: #2563eb;
  }

  .no-presets {
    text-align: center;
    color: #9ca3af;
    font-size: 0.875rem;
    padding: 2rem;
  }

  .preset-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .preset-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 1rem;
    background: #374151;
    border-radius: 8px;
    border: 1px solid #4b5563;
  }

  .preset-info {
    flex: 1;
  }

  .preset-info h4 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 0.5rem 0;
  }

  .preset-description {
    font-size: 0.875rem;
    color: #d1d5db;
    margin: 0 0 0.5rem 0;
  }

  .preset-meta {
    font-size: 0.75rem;
    color: #9ca3af;
    margin: 0;
  }

  .preset-actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .preset-action-btn {
    padding: 0.5rem 0.75rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .preset-action-btn.load {
    background: #3b82f6;
    color: white;
  }

  .preset-action-btn.load:hover {
    background: #2563eb;
  }

  .preset-action-btn.export {
    background: #059669;
    color: white;
  }

  .preset-action-btn.export:hover {
    background: #047857;
  }

  .preset-action-btn.delete {
    background: #ef4444;
    color: white;
  }

  .preset-action-btn.delete:hover {
    background: #dc2626;
  }

  .file-input {
    padding: 0.75rem;
    background: #374151;
    color: #f3f4f6;
    border: 1px solid #4b5563;
    border-radius: 6px;
    font-size: 0.875rem;
    cursor: pointer;
    width: 100%;
  }

  .file-input::-webkit-file-upload-button {
    padding: 0.5rem 1rem;
    background: #4b5563;
    color: #f3f4f6;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
    margin-right: 1rem;
  }

  .file-input::-webkit-file-upload-button:hover {
    background: #6b7280;
  }

  /* Scrollbar Styling */
  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  ::-webkit-scrollbar-track {
    background: #1f2937;
  }

  ::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
  }

  ::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }
</style>
