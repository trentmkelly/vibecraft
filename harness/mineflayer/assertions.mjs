import { offlineUuid } from './runner.mjs'

export function assertOfflineUuid(profile) {
  const expected = offlineUuid(profile.username)
  if (profile.expectedUuid && profile.expectedUuid !== expected) {
    return failure('uuid.expected_mismatch', `fixture expected UUID does not match vanilla offline UUID for ${profile.username}`, {
      username: profile.username,
      expected,
      fixtureExpected: profile.expectedUuid
    })
  }
  if (profile.actualUuid && normalizeUuid(profile.actualUuid) !== expected) {
    return failure('uuid.actual_mismatch', `bot UUID does not match vanilla offline UUID for ${profile.username}`, {
      username: profile.username,
      expected,
      actual: profile.actualUuid
    })
  }
  return ok({ username: profile.username, expected })
}

export function assertLoginPhases(events) {
  const spawnIndex = events.findIndex(event => event.name === 'spawn')
  const earlyFailure = events.find((event, index) =>
    (spawnIndex === -1 || index < spawnIndex) && ['kicked', 'error', 'end'].includes(event.name)
  )
  if (earlyFailure) {
    return failure('login.interrupted', `login was interrupted before spawn by ${earlyFailure.name}`, {
      event: earlyFailure,
      observed: events.map(event => event.name)
    })
  }
  return assertEventOrder(events, ['login', 'spawn'], { allowPacketsBetween: true, name: 'login phases' })
}

export function assertEventOrder(events, expected, options = {}) {
  const names = events.map(event => event.name)
  let cursor = 0
  const matched = []
  for (const expectedName of expected) {
    const index = names.indexOf(expectedName, cursor)
    if (index === -1) {
      return failure('event.missing', `missing ${expectedName} while checking ${options.name ?? 'event order'}`, {
        expected,
        observed: names,
        matched
      })
    }
    matched.push(expectedName)
    cursor = index + 1
  }
  if (options.exact === true) {
    const filtered = options.allowPacketsBetween
      ? names.filter(name => name !== 'packet')
      : names
    if (JSON.stringify(filtered) !== JSON.stringify(expected)) {
      return failure('event.extra_or_reordered', 'event order did not exactly match', {
        expected,
        observed: filtered
      })
    }
  }
  return ok({ expected, observed: names, matched })
}

export function assertPacketOrder(events, expectedPackets) {
  const packets = events
    .filter(event => event.name === 'packet')
    .map(event => event.summary?.[0])
  let cursor = 0
  for (const packet of expectedPackets) {
    const index = packets.indexOf(packet, cursor)
    if (index === -1) {
      return failure('packet.missing', `missing packet ${packet}`, {
        expected: expectedPackets,
        observed: packets
      })
    }
    cursor = index + 1
  }
  return ok({ expected: expectedPackets, observed: packets })
}

export function normalizeKickedMessage(value) {
  if (value == null) return null
  const parsed = typeof value === 'string' ? parseComponent(value) : value
  const rendered = renderComponent(parsed)
  const message = rendered == null ? String(value) : rendered
  return message
    .replace(/\\[nr]/g, ' ')
    .replace(/\s+/g, ' ')
    .replace(/§[0-9A-FK-OR]/gi, '')
    .trim()
}

export function assertNoUnexpectedKick(events) {
  const kicked = events.find(event => event.name === 'kicked')
  if (kicked) {
    return failure('kick.unexpected', 'unexpected kicked event', {
      message: normalizeKickedMessage(kicked.summary?.[0]),
      event: kicked
    })
  }
  return ok({ kicked: false })
}

export function assertKickedMessage(events, expected) {
  const kicked = events.find(event => event.name === 'kicked')
  if (!kicked) {
    return failure('kick.missing', 'missing kicked event', { expected })
  }
  const actual = normalizeKickedMessage(kicked.summary?.[0])
  const normalizedExpected = normalizeKickedMessage(expected)
  if (actual !== normalizedExpected) {
    return failure('kick.message_mismatch', 'kicked message mismatch', {
      expected: normalizedExpected,
      actual
    })
  }
  return ok({ expected: normalizedExpected, actual })
}

export function assertParityDiff(diff, options = {}) {
  const allowed = new Set(options.allowedPaths ?? [])
  const unexpected = diff.filter(entry => !allowed.has(entry.path))
  if (unexpected.length > 0) {
    return failure('parity.diff', 'official and VibeCraft artifacts differ', {
      unexpected,
      allowed: [...allowed]
    })
  }
  return ok({ diffCount: diff.length, allowed: [...allowed] })
}

export function formatParityDiff(diff) {
  if (diff.length === 0) return 'official and VibeCraft artifacts match'
  return diff.map(entry => [
    `path: ${entry.path}`,
    `official: ${stableStringify(entry.left ?? entry.official)}`,
    `VibeCraft: ${stableStringify(entry.right ?? entry.rebuilt)}`
  ].join('\n')).join('\n\n')
}

export function formatAssertionFailure(result) {
  if (result.ok) return null
  return `${result.code}: ${result.message}\n${JSON.stringify(result.details, null, 2)}`
}

function parseComponent(value) {
  try {
    return JSON.parse(value)
  } catch {
    return value
  }
}

function renderComponent(component) {
  if (component == null) return null
  if (typeof component === 'string') return component
  if (Array.isArray(component)) {
    return component.map(renderComponent).filter(part => part != null).join('')
  }
  if (typeof component !== 'object') return String(component)
  const ownText = component.translate ?? component.text ?? component.keybind ?? component.selector ?? component.score?.name
  const extras = component.extra ? renderComponent(component.extra) : ''
  return ownText == null && extras === '' ? null : `${ownText ?? ''}${extras}`
}

function stableStringify(value) {
  if (value === undefined) return '<missing>'
  return JSON.stringify(value, null, 2)
}

function normalizeUuid(uuid) {
  return uuid.toLowerCase()
}

function ok(details = {}) {
  return { ok: true, details }
}

function failure(code, message, details = {}) {
  return { ok: false, code, message, details }
}
