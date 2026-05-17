import { PROTOCOL_STATES, packetFlowSummary } from './packet_recorder.mjs'

export const GOLDEN_PACKET_FLOWS = {
  status: [
    packet('outbound', 'status', 'ping_start', ['payload']),
    packet('inbound', 'status', 'server_info', ['description', 'players', 'version']),
    packet('outbound', 'status', 'ping', ['time']),
    packet('inbound', 'status', 'pong', ['time'])
  ],
  login: [
    packet('outbound', 'login', 'login_start', ['username']),
    packet('inbound', 'login', 'success', ['uuid', 'username'])
  ],
  configuration: [
    packet('inbound', 'configuration', 'registry_data', ['registryCodec']),
    packet('inbound', 'configuration', 'update_tags', ['tags']),
    packet('outbound', 'configuration', 'finish_configuration', []),
    packet('inbound', 'configuration', 'finish_configuration', [])
  ],
  play: [
    packet('inbound', 'play', 'login', ['entityId']),
    packet('inbound', 'play', 'position', ['x', 'y', 'z']),
    packet('outbound', 'play', 'teleport_confirm', ['teleportId']),
    packet('inbound', 'play', 'keep_alive', ['keepAliveId']),
    packet('outbound', 'play', 'keep_alive', ['keepAliveId'])
  ]
}

export function goldenPacketTests() {
  return PROTOCOL_STATES.map(state => ({
    state,
    expected: GOLDEN_PACKET_FLOWS[state]
  }))
}

export function compareGoldenPackets(state, observedPackets, options = {}) {
  const expected = options.expected ?? GOLDEN_PACKET_FLOWS[state]
  if (!expected) throw new Error(`No golden packet flow for ${state}`)
  const observed = packetFlowSummary(observedPackets).filter(packet => packet.state === state)
  const diffs = []
  const exact = options.exact ?? false
  let cursor = 0
  for (const expectedPacket of expected) {
    const index = observed.findIndex((packet, observedIndex) =>
      observedIndex >= cursor && packetMatches(expectedPacket, packet)
    )
    if (index === -1) {
      diffs.push({
        code: 'golden_packet.missing',
        expected: expectedPacket,
        observedFrom: observed.slice(cursor)
      })
      continue
    }
    cursor = index + 1
  }
  if (exact && cursor < observed.length) {
    diffs.push({
      code: 'golden_packet.extra',
      extra: observed.slice(cursor)
    })
  }
  return {
    ok: diffs.length === 0,
    state,
    expected,
    observed,
    diffs
  }
}

export function formatGoldenPacketDiff(result) {
  if (result.ok) return `${result.state}: golden packet flow matched`
  return `${result.state}: golden packet flow mismatch\n${JSON.stringify(result.diffs, null, 2)}`
}

function packet(direction, state, name, keys) {
  return { direction, state, name, keys: [...keys].sort() }
}

function packetMatches(expected, observed) {
  return expected.direction === observed.direction &&
    expected.state === observed.state &&
    expected.name === observed.name &&
    expected.keys.every(key => observed.keys.includes(key))
}
