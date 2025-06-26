import { ref } from 'vue'

export function useResizable() {
  const isResizing = ref(false)

  const startResize = (
    event: MouseEvent,
    initialValue: number,
    onResize: (newValue: number) => void,
    options: { min?: number; max?: number; direction?: 'horizontal' | 'vertical' } = {}
  ) => {
    const { min = 100, max = 1000, direction = 'vertical' } = options

    isResizing.value = true
    const startPos = direction === 'vertical' ? event.clientY : event.clientX
    const startValue = initialValue

    const handleMouseMove = (e: MouseEvent) => {
      const delta = (direction === 'vertical' ? e.clientY : e.clientX) - startPos
      const newValue = Math.max(min, Math.min(max, startValue + delta))
      onResize(newValue)
    }

    const handleMouseUp = () => {
      isResizing.value = false
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
  }

  return {
    isResizing,
    startResize
  }
}
