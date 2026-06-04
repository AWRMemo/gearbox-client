import '@testing-library/jest-dom/vitest'

// Suppress React act() warnings from async hook state updates in tests
const originalError = console.error
console.error = (...args: unknown[]) => {
  const msg = args[0]?.toString() ?? ''
  if (msg.includes('was not wrapped in act')) return
  if (msg.includes('inside a test was not wrapped in act')) return
  originalError.apply(console, args)
}