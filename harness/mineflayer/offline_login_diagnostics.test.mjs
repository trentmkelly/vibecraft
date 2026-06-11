import test from 'node:test'
import assert from 'node:assert/strict'
import {
  assertOfflineLoginDiagnosticsContract,
  buildOfflineLoginDiagnostics
} from './offline_login_diagnostics.mjs'

test('buildOfflineLoginDiagnostics captures every required failure surface', () => {
  const diagnostics = buildOfflineLoginDiagnostics({
    profile: {
      username: 'DiagBot',
      expectedUuid: 'diag-uuid'
    },
    endpoint: { port: 25565 },
    server: { child: { pid: 12345 } },
    packetTrace: [
      { name: 'success', state: 'login', keys: ['uuid'] },
      { name: 'disconnect', state: 'configuration', keys: ['reason'] }
    ],
    timeline: [
      { name: 'login', summary: [] },
      { name: 'kicked', summary: ['{"text":"Unexpected packet"}'] }
    ],
    error: new Error('configuration failed')
  })

  assert.deepEqual(diagnostics, {
    username: 'DiagBot',
    expectedUuid: 'diag-uuid',
    serverPid: 12345,
    port: 25565,
    lastPacket: { name: 'disconnect', state: 'configuration', keys: ['reason'] },
    lastBotEvent: { name: 'kicked', summary: ['{"text":"Unexpected packet"}'] },
    normalizedDisconnectComponent: 'Unexpected packet',
    error: 'configuration failed'
  })
  assert.deepEqual(assertOfflineLoginDiagnosticsContract(diagnostics), {
    ok: true,
    missing: []
  })
})

test('assertOfflineLoginDiagnosticsContract reports missing diagnostic fields', () => {
  const diagnostics = buildOfflineLoginDiagnostics({
    profile: { username: 'MissingBot' },
    endpoint: {},
    packetTrace: [],
    timeline: []
  })
  assert.deepEqual(assertOfflineLoginDiagnosticsContract(diagnostics), {
    ok: false,
    missing: ['serverPid', 'port', 'lastPacket', 'lastBotEvent']
  })
})
