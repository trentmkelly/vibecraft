import test from 'node:test'
import assert from 'node:assert/strict'
import {
  normalizeLoginArtifacts,
  normalizeLoginDiff,
  normalizeLoginSessionArtifact
} from './artifact_normalizer.mjs'

test('normalizeLoginArtifacts redacts paths, ports, timestamps, and randomized usernames', () => {
  assert.deepEqual(normalizeLoginArtifacts({
    log: '2026-05-17T12:00:00Z /tmp/run-a listening on 25565 for GateBot-ab12',
    epoch: '1779043200000'
  }, {
    roots: ['/tmp/run-a'],
    ports: [25565]
  }), {
    epoch: '<timestamp-ms>',
    log: '<timestamp> <run-dir> listening on <port> for <username>'
  })
})

test('normalizeLoginArtifacts preserves UUIDs, packet order, and kicked messages', () => {
  const artifacts = normalizeLoginArtifacts({
    profile: {
      username: 'RustCraftBot-random',
      uuid: '5627dd98-e6be-3c21-b8a8-e92344183641'
    },
    timeline: [
      { name: 'packet', summary: ['login'] },
      { name: 'kicked', summary: ['{"text":"Server closed"}'] }
    ],
    packetTrace: [
      { name: 'success', state: 'login', keys: ['uuid'] },
      { name: 'keep_alive', state: 'play', keys: ['keepAliveId'] }
    ]
  })
  assert.equal(artifacts.profile.uuid, '5627dd98-e6be-3c21-b8a8-e92344183641')
  assert.deepEqual(artifacts.packetTrace.map(packet => packet.name), ['success', 'keep_alive'])
  assert.equal(artifacts.timeline[1].summary[0], '{"text":"Server closed"}')
  assert.equal(artifacts.profile.username, '<username>')
})

test('normalizeLoginDiff normalizes official-vs-RustCraft diffs without changing diff shape', () => {
  const diff = normalizeLoginDiff([
    {
      path: 'logs',
      official: ['started /tmp/official on 25565'],
      rebuilt: ['started /tmp/rebuilt on 25566']
    }
  ], {
    roots: ['/tmp/official', '/tmp/rebuilt'],
    ports: [25565, 25566]
  })
  assert.deepEqual(diff, [{
    official: ['started <run-dir> on <port>'],
    path: 'logs',
    rebuilt: ['started <run-dir> on <port>']
  }])
})

test('normalizeLoginSessionArtifact extracts stable session surfaces', () => {
  const normalized = normalizeLoginSessionArtifact({
    profile: { username: 'Steve' },
    uuid: '5627dd98-e6be-3c21-b8a8-e92344183641',
    timeline: [{ name: 'spawn', at: 1779043200000, summary: ['Steve joined'] }],
    packetTrace: [{ name: 'login', state: 'play', keys: ['entityId'] }],
    serverLogs: [{ stream: 'stdout', text: 'Steve joined from /tmp/world:25565' }],
    parityDiff: []
  }, {
    roots: ['/tmp/world'],
    ports: [25565],
    usernames: ['Steve']
  })
  assert.equal(normalized.profile.username, '<username>')
  assert.equal(normalized.uuid, '5627dd98-e6be-3c21-b8a8-e92344183641')
  assert.equal(normalized.timeline[0].at, 1779043200000)
  assert.equal(normalized.timeline[0].summary[0], '<username> joined')
  assert.equal(normalized.serverLogs[0].text, '<username> joined from <run-dir>:<port>')
})
