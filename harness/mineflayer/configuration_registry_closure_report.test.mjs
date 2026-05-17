import assert from 'node:assert/strict'
import test from 'node:test'

import { configurationCompletionManifest } from './configuration_completion_manifest.mjs'
import {
  documentedRegistryOmissions,
  evaluateConfigurationRegistryClosureGate,
  loadConfigurationRegistryClosureReport
} from './configuration_registry_closure_report.mjs'

test('configuration registry closure report covers every synchronized registry', async () => {
  const report = await loadConfigurationRegistryClosureReport()
  const registries = report.map(entry => entry.registry)

  assert.equal(new Set(registries).size, registries.length, 'registry closure report must not contain duplicates')
  assert.equal(registries[0], 'minecraft:biome', 'report must preserve decompiled synchronized registry order')
  assert.equal(registries.at(-1), 'minecraft:timeline', 'report must include the full synchronized registry list')

  const uncovered = report.filter(entry => {
    return !entry.emitted && !documentedRegistryOmissions.has(entry.registry)
  })
  assert.deepEqual(uncovered, [], 'each synchronized registry must be emitted or explicitly documented')

  const staleOmissions = [...documentedRegistryOmissions.keys()].filter(registry => {
    return !registries.includes(registry) || report.find(entry => entry.registry === registry)?.emitted
  })
  assert.deepEqual(staleOmissions, [], 'documented omissions must still exist in vanilla and must not already be emitted')
})

test('synced registry report entries are tied to manifest and raw-probe validation', async () => {
  const report = await loadConfigurationRegistryClosureReport()
  const synced = report.filter(entry => entry.emitted)

  assert.ok(synced.length > 0, 'expected at least one synced registry')

  for (const entry of synced) {
    assert.equal(entry.status, 'synced')
    assert.equal(entry.validator, 'raw_26_1_2_join_probe')
    assert.equal(entry.omission, null)
    assert.notEqual(entry.codec, 'unknown', `${entry.registry} needs a codec source`)
    assert.equal(
      entry.expectedElements,
      configurationCompletionManifest.elementCounts[entry.registry],
      `${entry.registry} element count must come from the completion manifest`
    )
  }
})

test('omitted registry report entries carry actionable milestone evidence', async () => {
  const report = await loadConfigurationRegistryClosureReport()
  const omitted = report.filter(entry => !entry.emitted)

  assert.deepEqual(
    omitted.map(entry => entry.registry),
    [
      'minecraft:biome',
      'minecraft:enchantment',
      'minecraft:test_environment',
      'minecraft:test_instance',
      'minecraft:dialog',
      'minecraft:world_clock',
      'minecraft:timeline'
    ]
  )

  for (const entry of omitted) {
    assert.equal(entry.status, 'documented_omission')
    assert.equal(entry.validator, 'documented_omission')
    assert.notEqual(entry.codec, 'unknown', `${entry.registry} needs a codec source`)
    assert.ok(entry.omission.milestone.length > 0, `${entry.registry} needs a milestone`)
    assert.ok(entry.omission.evidence.length > 0, `${entry.registry} needs evidence`)
    assert.ok(entry.omission.next.length > 30, `${entry.registry} needs an actionable next step`)
  }
})

test('configuration registry closure gate fails uncovered registries and missing play-entry evidence', () => {
  const report = [
    {
      registry: 'minecraft:damage_type',
      status: 'synced',
      emitted: true,
      codec: 'DamageType.DIRECT_CODEC',
      expectedElements: 50,
      validator: 'raw_26_1_2_join_probe',
      omission: null
    },
    {
      registry: 'minecraft:unknown_future_registry',
      status: 'documented_omission',
      emitted: false,
      codec: 'Unknown.CODEC',
      expectedElements: null,
      validator: 'documented_omission',
      omission: null
    }
  ]

  const gate = evaluateConfigurationRegistryClosureGate(report, '')
  assert.equal(gate.ok, false)
  assert.equal(gate.checks.find(check => check.name === 'minecraft:damage_type').ok, true)
  assert.equal(gate.checks.find(check => check.name === 'minecraft:unknown_future_registry').ok, false)
  assert.ok(gate.checks.some(check => check.name.startsWith('play-entry:') && !check.ok))
})

test('configuration registry closure gate passes current report with raw-probe play-entry evidence', async () => {
  const report = await loadConfigurationRegistryClosureReport()
  const rawProbe = [
    'expected 70-byte play login packet after holder-id encoding',
    'expected 62-byte player_position packet with fixed-int relatives',
    'missing play packet'
  ].join('\n')

  const gate = evaluateConfigurationRegistryClosureGate(report, rawProbe)
  assert.equal(gate.ok, true)
  assert.equal(gate.checks.filter(check => check.name.startsWith('play-entry:')).length, 3)
})
