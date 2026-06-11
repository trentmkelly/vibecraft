import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createOfflineParitySnapshot,
  sessionSnapshot
} from './offline_parity_snapshot.mjs'

test('sessionSnapshot preserves raw events, packet names, log excerpts, and normalized artifact surfaces', () => {
  const session = fakeSession('/tmp/vibecraft-snapshot', 25565, 'SnapBot')
  const snapshot = sessionSnapshot(session, { maxLogLines: 1 })

  assert.equal(snapshot.profile.username, 'SnapBot')
  assert.deepEqual(snapshot.rawEvents.map(event => event.name), ['login', 'packet', 'spawn'])
  assert.deepEqual(snapshot.packetNames, ['success', 'finish_configuration'])
  assert.deepEqual(snapshot.serverLogExcerpt, [{
    stream: 'stdout',
    text: '2026-05-17T12:00:00Z /tmp/vibecraft-snapshot accepted SnapBot on 25565'
  }])
  assert.equal(snapshot.normalized.profile.username, '<username>')
  assert.equal(snapshot.normalized.serverLogs[0].text, '<timestamp> <run-dir> accepted <username> on <port>\n')
})

test('createOfflineParitySnapshot stores normalized official-vs-VibeCraft diffs', () => {
  const official = fakeSession('/tmp/official-snapshot', 25565, 'SnapBot')
  const vibecraft = fakeSession('/tmp/vibecraft-snapshot', 25566, 'SnapBot')
  const snapshot = createOfflineParitySnapshot({
    official,
    vibecraft,
    diff: [{
      path: 'events',
      official: [{ name: 'spawn', summary: ['SnapBot'] }],
      rebuilt: [{ name: 'kicked', summary: ['SnapBot'] }]
    }]
  })

  assert.equal(snapshot.scenario, 'offline-mode-happy-path')
  assert.deepEqual(snapshot.normalizedDiff, [{
    official: [{ name: 'spawn', summary: ['<username>'] }],
    path: 'events',
    rebuilt: [{ name: 'kicked', summary: ['<username>'] }]
  }])
  assert.match(snapshot.diffText, /path: events/)
})

function fakeSession(root, port, username) {
  return {
    root,
    endpoint: { port },
    profile: { username, expectedUuid: 'uuid', actualUuid: 'uuid' },
    uuid: 'uuid',
    timeline: [
      { name: 'login', at: 1, summary: [] },
      { name: 'packet', at: 2, summary: ['success'] },
      { name: 'spawn', at: 3, summary: [] }
    ],
    packetTrace: [
      { name: 'success', state: 'login' },
      { name: 'finish_configuration', state: 'configuration' }
    ],
    serverLogs: [{
      stream: 'stdout',
      text: `2026-05-17T12:00:00Z ${root} accepted ${username} on ${port}\n`
    }],
    parityDiff: []
  }
}
