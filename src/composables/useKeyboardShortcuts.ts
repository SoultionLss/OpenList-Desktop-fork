export function useKeyboardShortcuts() {
  const handleKeydown = (event: KeyboardEvent, callbacks: Record<string, () => void>) => {
    const { metaKey, ctrlKey, shiftKey, key } = event
    const modifier = metaKey || ctrlKey

    if (modifier && key === 'k' && callbacks.search) {
      event.preventDefault()
      callbacks.search()
    }

    if (modifier && shiftKey && key === 'L' && callbacks.logs) {
      event.preventDefault()
      callbacks.logs()
    }

    if (modifier && shiftKey && key === 'M' && callbacks.metrics) {
      event.preventDefault()
      callbacks.metrics()
    }

    if (modifier && key === '1' && callbacks.dashboard) {
      event.preventDefault()
      callbacks.dashboard()
    }

    if (key === 'F11' && callbacks.fullscreen) {
      event.preventDefault()
      callbacks.fullscreen()
    }
  }

  return {
    handleKeydown
  }
}
