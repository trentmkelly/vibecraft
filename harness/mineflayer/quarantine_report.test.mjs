import test from 'node:test'
import assert from 'node:assert/strict'
import {
  classifyFailure,
  createQuarantineReport,
  formatQuarantineReport,
  prismarineDependencySnapshot
} from './quarantine_report.mjs'

test('classifyFailure separates Mineflayer version incompatibility from server milestones', () => {
  assert.deepEqual(classifyFailure({
    error: new Error('No data available for version 1.99')
  }), {
    kind: 'mineflayer_client_version',
    reason: 'Mineflayer or minecraft-data does not support the requested version'
  })
  assert.deepEqual(classifyFailure({
    milestones: [{ name: 'configurationOrdering', ok: false }]
  }), {
    kind: 'server_bug',
    reason: 'Failed server milestone: configurationOrdering'
  })
})

test('classifyFailure uses raw disconnect packets for outdated client/server reports', () => {
  assert.equal(classifyFailure({
    rawDisconnectPackets: [{ data: { reason: 'multiplayer.disconnect.outdated_client' } }]
  }).kind, 'mineflayer_client_version')
})

test('prismarineDependencySnapshot extracts mineflayer and prismarine package versions', () => {
  assert.deepEqual(prismarineDependencySnapshot({
    packages: {
      '': { version: '0.0.0' },
      'node_modules/mineflayer': { version: '4.37.1' },
      'node_modules/prismarine-chat': { version: '1.11.0' },
      'node_modules/minecraft-data': { version: '3.90.0' },
      'node_modules/left-pad': { version: '1.0.0' }
    }
  }), {
    'minecraft-data': '3.90.0',
    mineflayer: '4.37.1',
    'prismarine-chat': '1.11.0'
  })
})

test('createQuarantineReport captures protocol, dependencies, raw disconnect packets, and evidence', () => {
  const report = createQuarantineReport({
    session: {
      endpoint: { version: '1.21.6' },
      protocolVersion: 771,
      error: new Error('spawn timeout'),
      timeline: [{ name: 'login', summary: [] }],
      packetTrace: [
        { name: 'disconnect', state: 'login', keys: ['reason'], data: { reason: 'Server closed' } }
      ]
    },
    milestones: [{ name: 'firstSpawn', ok: false }],
    packageLock: {
      packages: { 'node_modules/mineflayer': { version: '4.37.1' } }
    }
  })
  assert.equal(report.classification.kind, 'server_bug')
  assert.equal(report.protocol.requestedVersion, '1.21.6')
  assert.equal(report.protocol.protocolVersion, 771)
  assert.equal(report.rawDisconnectPackets.length, 1)
  assert.deepEqual(report.dependencies, { mineflayer: '4.37.1' })
  assert.equal(report.evidence.timeline[0].name, 'login')
})

test('formatQuarantineReport emits compact triage text', () => {
  const report = createQuarantineReport({
    session: { endpoint: { version: '1.21.6' }, packetTrace: [] },
    packageLock: { packages: { 'node_modules/mineflayer': { version: '4.37.1' } } }
  })
  assert.match(formatQuarantineReport(report), /classification: unknown/)
  assert.match(formatQuarantineReport(report), /mineflayer@4\.37\.1/)
})
