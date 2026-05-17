import assert from 'node:assert/strict'
import test from 'node:test'
import {
  createConfigurationCustomPayloadPlan,
  createConfigurationReplayPlan,
  createConfigurationStatePlan,
  summarizeConfigurationCustomPayloadEvidence,
  summarizeConfigurationReplayEvidence,
  summarizeConfigurationStateEvidence
} from './configuration_state_scenarios.mjs'

test('configuration state plan covers registries, tags, features, known packs, and finish', () => {
  const plan = createConfigurationStatePlan()

  assert.equal(plan.name, 'mineflayer-offline-configuration-state')
  assert.ok(plan.required.includes('enabled-features-before-finish'))
  assert.ok(plan.required.includes('registry-order-before-finish'))
  assert.ok(plan.required.includes('tag-packets-before-finish'))
  assert.ok(plan.required.includes('known-packs-before-finish'))
  assert.ok(plan.required.includes('finish-configuration-after-known-packs'))
  assert.ok(plan.required.includes('play-entry-after-client-finish'))
  assert.deepEqual(plan.manifest.knownPacks, [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }])
})

test('configuration state evidence passes only when manifest order reaches play after finish', () => {
  const plan = createConfigurationStatePlan()
  const complete = {
    configPackets: [
      { id: 12 },
      ...plan.manifest.registryOrder.map(registry => ({ id: 7, registry })),
      { id: 13 },
      { id: 14, packs: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }] },
      { id: 3 }
    ],
    playPackets: [49, 105, 72, 12, 45, 11].map(id => ({ id }))
  }

  assert.equal(summarizeConfigurationStateEvidence(complete, plan).ok, true)

  const missingKnownPacks = {
    ...complete,
    configPackets: complete.configPackets.filter(packet => packet.id !== 14)
  }
  const summary = summarizeConfigurationStateEvidence(missingKnownPacks, plan)
  assert.equal(summary.ok, false)
  assert.ok(summary.missing.includes('known-packs-before-finish'))
})

test('configuration regression evidence rejects play without complete configuration sync', () => {
  const plan = createConfigurationStatePlan()
  const incomplete = {
    configPackets: [
      { id: 12 },
      { id: 13 },
      { id: 14, packs: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }] },
      { id: 3 }
    ],
    playPackets: [49, 105, 72, 12, 45, 11].map(id => ({ id }))
  }

  const summary = summarizeConfigurationStateEvidence(incomplete, plan)
  assert.equal(summary.ok, false)
  assert.ok(summary.missing.includes('registry-order-before-finish'))

  const noFinish = {
    configPackets: [
      { id: 12 },
      ...plan.manifest.registryOrder.map(registry => ({ id: 7, registry })),
      { id: 13 },
      { id: 14, packs: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }] }
    ],
    playPackets: [49, 105, 72, 12, 45, 11].map(id => ({ id }))
  }

  const noFinishSummary = summarizeConfigurationStateEvidence(noFinish, plan)
  assert.equal(noFinishSummary.ok, false)
  assert.ok(noFinishSummary.missing.includes('finish-configuration-after-known-packs'))
})

test('configuration custom payload plan covers unknown payloads, brand, client info, cookies, and disconnects', () => {
  const plan = createConfigurationCustomPayloadPlan()

  assert.equal(plan.name, 'mineflayer-offline-configuration-custom-payload')
  assert.deepEqual(plan.required, [
    'records-unknown-custom-payload',
    'records-brand-exchange',
    'records-client-information',
    'records-cookie-request-response',
    'records-configuration-disconnect'
  ])
})

test('configuration custom payload evidence requires every diagnostic surface', () => {
  const complete = {
    packetTrace: [
      { name: 'custom_payload', channel: 'minecraft:brand' },
      { name: 'custom_payload', channel: 'rustcraft:unknown_probe' },
      { name: 'client_information' },
      { name: 'cookie_request' },
      { name: 'cookie_response' },
      { name: 'disconnect' }
    ],
    timeline: []
  }

  assert.equal(summarizeConfigurationCustomPayloadEvidence(complete).ok, true)

  const missingCookieResponse = {
    ...complete,
    packetTrace: complete.packetTrace.filter(packet => packet.name !== 'cookie_response')
  }
  const summary = summarizeConfigurationCustomPayloadEvidence(missingCookieResponse)
  assert.equal(summary.ok, false)
  assert.ok(summary.missing.includes('records-cookie-request-response'))
})

test('configuration replay plan requires official and RustCraft transcripts without hidden sleeps', () => {
  const plan = createConfigurationReplayPlan()

  assert.equal(plan.name, 'mineflayer-offline-configuration-replay')
  assert.deepEqual(plan.required, [
    'records-official-configuration-transcript',
    'records-rustcraft-configuration-transcript',
    'matches-login-milestones',
    'matches-configuration-milestones',
    'matches-play-entry-milestones',
    'no-hidden-sleeps',
    'no-retry-only-success'
  ])
})

test('configuration replay evidence compares milestones and rejects sleep or retry-only success', () => {
  const milestones = ['tcp-connect', 'login-success', 'configuration-start', 'known-packs', 'finish-configuration', 'play-login']
  const complete = {
    official: { configPackets: [{ id: 12 }], milestones },
    rustcraft: { configPackets: [{ id: 12 }], milestones },
    hiddenSleeps: [],
    retryOnlySuccess: false
  }

  assert.equal(summarizeConfigurationReplayEvidence(complete).ok, true)

  const hiddenSleepSummary = summarizeConfigurationReplayEvidence({
    ...complete,
    hiddenSleeps: ['waited 1000ms before reading play packet']
  })
  assert.equal(hiddenSleepSummary.ok, false)
  assert.ok(hiddenSleepSummary.missing.includes('no-hidden-sleeps'))

  const retryOnlySummary = summarizeConfigurationReplayEvidence({
    ...complete,
    retryOnlySuccess: true
  })
  assert.equal(retryOnlySummary.ok, false)
  assert.ok(retryOnlySummary.missing.includes('no-retry-only-success'))
})
