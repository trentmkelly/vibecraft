import test from 'node:test'
import assert from 'node:assert/strict'
import {
  buildRegistryLoginDiffEvidence,
  buildRegistrySizeGuardEvidence,
  buildRegistrySyncEvidence,
  compactRegistryDiff,
  createRegistryScenarioPlan,
  summarizeRegistryEvidence
} from './registry_scenarios.mjs'

test('createRegistryScenarioPlan covers sync, diff, and size guard scenarios', () => {
  const plan = createRegistryScenarioPlan()

  assert.equal(plan.name, 'mineflayer-offline-registry-scenarios')
  assert.deepEqual(plan.scenarios.map(scenario => scenario.name), [
    'registry-sync',
    'registry-login-diff',
    'registry-size-guard'
  ])
})

test('registry scenarios include required packet, oracle, diff, and payload evidence', () => {
  const scenarios = Object.fromEntries(createRegistryScenarioPlan().scenarios.map(scenario => [scenario.name, scenario.required]))

  assert.ok(scenarios['registry-sync'].includes('compares-registry-ids'))
  assert.ok(scenarios['registry-sync'].includes('compares-tag-contents'))
  assert.ok(scenarios['registry-sync'].includes('compares-known-packs'))
  assert.ok(scenarios['registry-sync'].includes('compares-enabled-feature-order'))
  assert.ok(scenarios['registry-login-diff'].includes('compact-registry-diff-on-play-state-failure'))
  assert.ok(scenarios['registry-size-guard'].includes('no-mineflayer-parser-errors'))
  assert.ok(scenarios['registry-size-guard'].includes('no-compression-regression'))
})

test('summarizeRegistryEvidence passes complete evidence and fails missing surfaces', () => {
  const plan = createRegistryScenarioPlan()
  const complete = Object.fromEntries(plan.scenarios.map(scenario => [
    scenario.name,
    Object.fromEntries(scenario.required.map(key => [key, true]))
  ]))

  assert.equal(summarizeRegistryEvidence(complete, plan).ok, true)
  const incomplete = summarizeRegistryEvidence({ 'registry-sync': { 'compares-registry-ids': true } }, plan)
  assert.equal(incomplete.ok, false)
  assert.ok(incomplete.scenarios.find(result => result.name === 'registry-sync').missing.includes('compares-tag-contents'))
})

test('compactRegistryDiff reports registry, tag, known pack, and feature differences', () => {
  const official = [{
    name: 'registry_data',
    registryId: 'minecraft:dimension_type',
    keys: ['registryCodec'],
    tagCount: 2,
    knownPackCount: 1,
    enabledFeatures: ['minecraft:vanilla']
  }]
  const rustCraft = [{
    name: 'registry_data',
    registryId: 'minecraft:dimension_type',
    keys: ['registryCodec'],
    tagCount: 1,
    knownPackCount: 1,
    enabledFeatures: []
  }]

  assert.deepEqual(compactRegistryDiff(official, rustCraft), [{
    registry: 'minecraft:dimension_type',
    official: {
      keys: ['registryCodec'],
      tags: 2,
      knownPacks: 1,
      enabledFeatures: ['minecraft:vanilla']
    },
    rustCraft: {
      keys: ['registryCodec'],
      tags: 1,
      knownPacks: 1,
      enabledFeatures: []
    }
  }])
})

test('buildRegistrySyncEvidence compares vanilla transcript surfaces required by registry-sync', () => {
  const officialTranscript = {
    source: 'official-server.jar',
    registries: [{ registry: 'minecraft:dimension_type', elementIds: ['minecraft:overworld'] }],
    tags: [{ registry: 'minecraft:dimension_type', tags: [{ tag: 'minecraft:overworld_like', entries: [0] }] }],
    knownPacks: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }],
    enabledFeatures: ['minecraft:vanilla']
  }
  const rustCraftTranscript = structuredClone(officialTranscript)
  rustCraftTranscript.source = 'rustcraft'

  assert.equal(summarizeRegistryEvidence({
    'registry-sync': buildRegistrySyncEvidence({ officialTranscript, rustCraftTranscript }),
    'registry-login-diff': Object.fromEntries(createRegistryScenarioPlan().scenarios[1].required.map(key => [key, true])),
    'registry-size-guard': Object.fromEntries(createRegistryScenarioPlan().scenarios[2].required.map(key => [key, true]))
  }).ok, true)

  rustCraftTranscript.tags[0].tags[0].entries = []
  assert.equal(buildRegistrySyncEvidence({ officialTranscript, rustCraftTranscript })['compares-tag-contents'], false)
})

test('buildRegistryLoginDiffEvidence requires same bot and emits compact diff on play failure', () => {
  const officialTranscript = {
    username: 'RegistryProbe',
    registries: [{ registry: 'minecraft:damage_type', elementIds: ['minecraft:generic', 'minecraft:fall'] }],
    tags: [],
    knownPacks: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }]
  }
  const rustCraftTranscript = {
    username: 'RegistryProbe',
    registries: [{ registry: 'minecraft:damage_type', elementIds: ['minecraft:generic'] }],
    tags: [],
    knownPacks: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }],
    disconnectReason: 'Failed to decode registry data'
  }

  const evidence = buildRegistryLoginDiffEvidence({
    officialTranscript,
    rustCraftTranscript,
    rustCraftPlayStateEntered: false
  })

  assert.equal(evidence['same-bot-against-rustcraft-and-official'], true)
  assert.equal(evidence['compact-registry-diff-on-play-state-failure'], true)
  assert.equal(evidence['configuration-diff-on-play-state-failure'], true)
  assert.deepEqual(evidence.diffs.map(diff => diff.registry), ['minecraft:damage_type'])
})

test('buildRegistrySizeGuardEvidence fails closed on parser, truncation, and compression regressions', () => {
  const passing = buildRegistrySizeGuardEvidence({
    transcript: {
      registryBytes: 120_000,
      tagBytes: 80_000,
      finishConfigurationPacketId: 3,
      truncatedPackets: false
    },
    parserErrors: [],
    compressionThreshold: -1,
    expectedCompressionThreshold: -1
  })

  assert.equal(passing['large-registry-payload'], true)
  assert.equal(passing['large-tag-payload'], true)
  assert.equal(passing['configuration-completes'], true)
  assert.equal(passing['no-mineflayer-parser-errors'], true)

  const failing = buildRegistrySizeGuardEvidence({
    transcript: { registryBytes: 1, tagBytes: 1, finishConfigurationPacketId: 7, truncatedPackets: true },
    parserErrors: ['VarInt too big'],
    compressionThreshold: 256,
    expectedCompressionThreshold: -1
  })

  assert.equal(failing['configuration-completes'], false)
  assert.equal(failing['no-mineflayer-parser-errors'], false)
  assert.equal(failing['no-truncated-packets'], false)
  assert.equal(failing['no-compression-regression'], false)
})
