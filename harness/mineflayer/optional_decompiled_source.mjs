export const decompiledSourceRoot = process.env.VIBECRAFT_DECOMPILED_SOURCE_ROOT
export const missingJavaSourceReason = 'optional Java source root unavailable'

const warned = new Set()

export function warnMissingJavaSource (consumer) {
  if (warned.has(consumer)) return
  warned.add(consumer)
  console.warn(`${consumer}: ${missingJavaSourceReason}; skipping source-backed parity check`)
}

