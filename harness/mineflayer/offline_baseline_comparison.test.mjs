import test from 'node:test'
import assert from 'node:assert/strict'
import {
  acceptedTimelineEnvelope,
  compareEnvelope,
  runOfflineBaselineComparison
} from './offline_baseline_comparison.mjs'

test('acceptedTimelineEnvelope stores vanilla event and packet order', () => {
  const envelope = acceptedTimelineEnvelope(session(['login', 'packet', 'spawn'], ['success', 'finish_configuration']))
  assert.deepEqual(envelope.eventNames, ['login', 'packet', 'spawn'])
  assert.deepEqual(envelope.packetNames, ['success', 'finish_configuration'])
})

test('compareEnvelope reports event and packet differences against VibeCraft run', () => {
  const envelope = acceptedTimelineEnvelope(session(['login', 'spawn'], ['success']))
  const diff = compareEnvelope(envelope, session(['login', 'kicked'], ['disconnect']))
  assert.deepEqual(diff.map(entry => entry.path), ['events', 'packets'])
})

test('runOfflineBaselineComparison runs vanilla first and reuses its envelope', async () => {
  const result = await runOfflineBaselineComparison({
    runOfficial: async () => session(['login', 'packet', 'spawn'], ['success', 'finish_configuration']),
    runVibeCraft: async () => session(['login', 'packet', 'spawn'], ['success', 'finish_configuration'])
  })
  assert.equal(result.ok, true)
  assert.deepEqual(result.diff, [])
  assert.equal(result.assertion.ok, true)
  assert.equal(result.snapshot.scenario, 'offline-mode-happy-path')
})

function session(events, packets) {
  return {
    root: '/tmp/session',
    endpoint: { port: 25565 },
    profile: { username: 'BaselineBot', expectedUuid: 'uuid', actualUuid: 'uuid' },
    uuid: 'uuid',
    timeline: events.map(name => ({ name, summary: [name] })),
    packetTrace: packets.map(name => ({ name, state: 'play' })),
    serverLogs: []
  }
}
