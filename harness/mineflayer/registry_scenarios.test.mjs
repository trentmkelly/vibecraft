import test from 'node:test'
import assert from 'node:assert/strict'
import {
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
