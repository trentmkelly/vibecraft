export function summarizeLoginOrder (rawProbe) {
  const configIds = rawProbe.config?.map(packet => packet.id) ?? []
  const playIds = rawProbe.play?.map(packet => packet.id) ?? []
  return [
    'handshake',
    'login_start',
    rawProbe.compressionThreshold === null || rawProbe.compressionThreshold === undefined
      ? 'login/set_compression_absent'
      : 'login/set_compression',
    rawProbe.login === 2 ? 'login_success' : `login_packet_${rawProbe.login}`,
    'login_acknowledgement',
    ...configIds.map(id => configurationName(id)),
    'finish_configuration_acknowledgement',
    ...playIds.map(id => playName(id))
  ]
}

export function evaluateLoginOrder (rawProbe) {
  const timeline = summarizeLoginOrder(rawProbe)
  const expectedPrefix = [
    'handshake',
    'login_start',
    rawProbe.compressionThreshold === null || rawProbe.compressionThreshold === undefined
      ? 'login/set_compression_absent'
      : 'login/set_compression',
    'login_success',
    'login_acknowledgement',
    'configuration/update_enabled_features'
  ]
  const expectedConfigurationTail = [
    'configuration/select_known_packs',
    'configuration/finish_configuration',
    'finish_configuration_acknowledgement',
    'play/login'
  ]
  const failures = []
  if (!startsWith(timeline, expectedPrefix)) failures.push('login-prefix')
  if (!containsOrdered(timeline, expectedConfigurationTail)) failures.push('configuration-to-play-tail')
  if (timeline.indexOf('play/login') < timeline.indexOf('configuration/finish_configuration')) {
    failures.push('play-before-finish-configuration')
  }
  if (timeline.includes('configuration/registry_data') && timeline.indexOf('configuration/registry_data') > timeline.indexOf('configuration/select_known_packs')) {
    failures.push('registry-after-known-packs')
  }

  return {
    ok: failures.length === 0,
    failures,
    timeline
  }
}

export function compareLoginOrderAgainstOfficial (actualRawProbe, officialRawProbe) {
  const actual = summarizeLoginOrder(actualRawProbe)
  const official = summarizeLoginOrder(officialRawProbe)
  const firstMismatch = firstSequenceMismatch(actual, official)
  return {
    ok: firstMismatch === null,
    actual,
    official,
    firstMismatch
  }
}

function startsWith (values, prefix) {
  return prefix.every((value, index) => values[index] === value)
}

function containsOrdered (values, expected) {
  let cursor = 0
  for (const value of values) {
    if (value === expected[cursor]) cursor++
    if (cursor === expected.length) return true
  }
  return false
}

function firstSequenceMismatch (actual, official) {
  const length = Math.max(actual.length, official.length)
  for (let index = 0; index < length; index++) {
    if (actual[index] !== official[index]) {
      return {
        index,
        actual: actual[index] ?? null,
        official: official[index] ?? null,
        actualContext: actual.slice(Math.max(0, index - 3), index + 4),
        officialContext: official.slice(Math.max(0, index - 3), index + 4)
      }
    }
  }
  return null
}

function configurationName (id) {
  return {
    3: 'configuration/finish_configuration',
    7: 'configuration/registry_data',
    12: 'configuration/update_enabled_features',
    13: 'configuration/update_tags',
    14: 'configuration/select_known_packs'
  }[id] ?? `configuration/packet_${id}`
}

function playName (id) {
  return {
    49: 'play/login',
    70: 'play/player_info_update',
    45: 'play/level_chunk_with_light'
  }[id] ?? `play/packet_${id}`
}
