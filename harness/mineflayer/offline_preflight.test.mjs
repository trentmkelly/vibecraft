import test from 'node:test'
import assert from 'node:assert/strict'
import {
  preflightFailsFast,
  runOfflinePreflight
} from './offline_preflight.mjs'

test('runOfflinePreflight passes when login shard reaches all milestones', async () => {
  const result = await runOfflinePreflight({
    runShard: async () => ({
      ok: true,
      milestones: [{ name: 'tcpReadiness', ok: true }],
      report: 'ok',
      session: completeSession()
    })
  })
  assert.equal(result.ok, true)
  assert.equal(result.reason, null)
})

test('runOfflinePreflight fails fast with actionable artifacts when readiness or login stalls', async () => {
  const result = await runOfflinePreflight({
    username: 'PreflightBot',
    port: 25565,
    serverPid: 444,
    runShard: async () => ({
      ok: false,
      milestones: [
        { name: 'tcpReadiness', ok: false, detail: 'Timed out waiting for 127.0.0.1:25565' },
        { name: 'login', ok: false, detail: '<none>' }
      ],
      report: 'failed',
      session: {
        ...completeSession(),
        timeline: [{ name: 'end', summary: ['timeout'] }],
        packetTrace: [{ name: 'handshake', state: 'handshake', keys: [] }],
        error: new Error('Timed out waiting for 127.0.0.1:25565')
      }
    })
  })
  assert.equal(result.ok, false)
  assert.equal(result.reason, 'tcpReadiness,login')
  assert.equal(result.artifacts.diagnostics.username, 'PreflightBot')
  assert.equal(result.artifacts.diagnostics.port, 25565)
  assert.equal(preflightFailsFast(result).actionable, true)
})

function completeSession() {
  return {
    profile: {
      username: 'PreflightBot',
      expectedUuid: 'uuid',
      actualUuid: 'uuid'
    },
    endpoint: { port: 25565 },
    server: { child: { pid: 444 } },
    timeline: [{ name: 'login', summary: [] }, { name: 'spawn', summary: [] }],
    packetTrace: [{ name: 'finish_configuration', state: 'configuration', keys: [] }]
  }
}
