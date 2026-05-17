import assert from 'node:assert/strict'
import test from 'node:test'
import {
  createConfigurationStatePlan,
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
    playPackets: [49, 105, 72, 12, 48, 11].map(id => ({ id }))
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
