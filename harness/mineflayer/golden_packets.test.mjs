import test from 'node:test'
import assert from 'node:assert/strict'
import {
  GOLDEN_PACKET_FLOWS,
  compareGoldenPackets,
  formatGoldenPacketDiff,
  goldenPacketTests
} from './golden_packets.mjs'

test('goldenPacketTests defines coverage for every protocol state', () => {
  assert.deepEqual(goldenPacketTests().map(testCase => testCase.state), [
    'status',
    'login',
    'configuration',
    'play'
  ])
  for (const testCase of goldenPacketTests()) {
    assert.ok(testCase.expected.length > 0)
  }
})

test('compareGoldenPackets accepts ordered subsequences with required keys', () => {
  const packets = [
    observed('outbound', 'login', 'login_start', { username: 'Bot', signature: null }),
    observed('inbound', 'login', 'success', { uuid: 'u', username: 'Bot', properties: [] })
  ]
  assert.equal(compareGoldenPackets('login', packets).ok, true)
})

test('compareGoldenPackets reports missing packets with stable diff output', () => {
  const result = compareGoldenPackets('status', [
    observed('outbound', 'status', 'ping_start', { payload: 1 })
  ])
  assert.equal(result.ok, false)
  assert.equal(result.diffs[0].code, 'golden_packet.missing')
  assert.match(formatGoldenPacketDiff(result), /status: golden packet flow mismatch/)
})

test('compareGoldenPackets exact mode reports trailing extras', () => {
  const result = compareGoldenPackets('login', [
    observed('outbound', 'login', 'login_start', { username: 'Bot' }),
    observed('inbound', 'login', 'success', { uuid: 'u', username: 'Bot' }),
    observed('inbound', 'login', 'set_compression', { threshold: 256 })
  ], { exact: true })
  assert.equal(result.ok, false)
  assert.equal(result.diffs[0].code, 'golden_packet.extra')
})

test('golden packet flows include status, login, configuration, and play milestones', () => {
  assert.deepEqual(GOLDEN_PACKET_FLOWS.status.map(packet => packet.name), [
    'ping_start',
    'server_info',
    'ping',
    'pong'
  ])
  assert.ok(GOLDEN_PACKET_FLOWS.configuration.some(packet => packet.name === 'registry_data'))
  assert.ok(GOLDEN_PACKET_FLOWS.play.some(packet => packet.name === 'keep_alive'))
})

test('formatGoldenPacketDiff returns compact success messages', () => {
  const result = compareGoldenPackets('play', GOLDEN_PACKET_FLOWS.play.map(packet => ({
    direction: packet.direction,
    state: packet.state,
    name: packet.name,
    data: Object.fromEntries(packet.keys.map(key => [key, 1]))
  })))
  assert.equal(formatGoldenPacketDiff(result), 'play: golden packet flow matched')
})

function observed(direction, state, name, data) {
  return { direction, state, name, data }
}
