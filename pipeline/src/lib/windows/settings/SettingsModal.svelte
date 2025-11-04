<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    confirm: string | null;
    cancel: void;
  }>();

  export let title: string = 'Confirm';
  export let message: string = '';
  export let confirmText: string = 'OK';
  export let cancelText: string = 'Cancel';
  export let showInput: boolean = false;
  export let inputPlaceholder: string = '';
  export let inputValue: string = '';
  export let inputType: string = 'text';
  export let showCancel: boolean = true;
  export let dangerous: boolean = false;

  let localInputValue: string = inputValue;
  let isValid: boolean = true;

  function handleConfirm() {
    if (showInput) {
      if (!localInputValue.trim()) {
        isValid = false;
        return;
      }
      dispatch('confirm', localInputValue);
    } else {
      dispatch('confirm', null);
    }
  }

  function handleCancel() {
    dispatch('cancel');
  }

  function handleInputChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localInputValue = target.value;
    isValid = true;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleCancel();
    } else if (event.key === 'Enter' && !showInput) {
      handleConfirm();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="modal-overlay" on:click={handleCancel}>
  <div class="modal-dialog" on:click|stopPropagation>
    <div class="modal-header">
      <h2 class="modal-title">{title}</h2>
      <button class="close-btn" on:click={handleCancel} aria-label="Close">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      {#if message}
        <p class="modal-message">{message}</p>
      {/if}

      {#if showInput}
        <input
          type={inputType}
          class="modal-input"
          class:invalid={!isValid}
          placeholder={inputPlaceholder}
          value={localInputValue}
          on:input={handleInputChange}
          autofocus
        />
        {#if !isValid}
          <p class="error-message">This field is required</p>
        {/if}
      {/if}
    </div>

    <div class="modal-footer">
      {#if showCancel}
        <button class="btn btn-secondary" on:click={handleCancel}>
          {cancelText}
        </button>
      {/if}
      <button
        class="btn"
        class:btn-primary={!dangerous}
        class:btn-danger={dangerous}
        on:click={handleConfirm}
      >
        {confirmText}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .modal-dialog {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    min-width: 400px;
    max-width: 600px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .modal-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    margin: 0;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary, #a0a0a0);
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    background: var(--bg-hover, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
  }

  .modal-body {
    padding: 1.5rem;
  }

  .modal-message {
    color: var(--text-primary, #e0e0e0);
    margin: 0 0 1rem 0;
    line-height: 1.5;
  }

  .modal-input {
    width: 100%;
    padding: 0.75rem;
    background: var(--bg-primary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    color: var(--text-primary, #e0e0e0);
    font-size: 0.95rem;
    transition: all 0.2s;
  }

  .modal-input:focus {
    outline: none;
    border-color: var(--accent-color, #4a9eff);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .modal-input.invalid {
    border-color: var(--error-color, #ff4444);
  }

  .error-message {
    color: var(--error-color, #ff4444);
    font-size: 0.875rem;
    margin: 0.5rem 0 0 0;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1.5rem;
    border-top: 1px solid var(--border-color, #333);
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-secondary {
    background: var(--bg-primary, #252525);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #333);
  }

  .btn-secondary:hover {
    background: var(--bg-hover, #2a2a2a);
    border-color: var(--border-hover, #444);
  }

  .btn-primary {
    background: var(--accent-color, #4a9eff);
    color: #fff;
  }

  .btn-primary:hover {
    background: #3a8eef;
  }

  .btn-danger {
    background: var(--error-color, #ff4444);
    color: #fff;
  }

  .btn-danger:hover {
    background: #ee3333;
  }
</style>
