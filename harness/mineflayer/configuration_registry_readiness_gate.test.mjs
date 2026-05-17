import assert from 'node:assert/strict'
import test from 'node:test'

import { configurationCompletionManifest } from './configuration_completion_manifest.mjs'
import {
  evaluateManifestReadiness,
  formatConfigurationRegistryReadinessGateReport,
  runConfigurationRegistryReadinessGate
} from './configuration_registry_readiness_gate.mjs'

function passingRawProbe () {
  return {
    ok: true,
    config: [
      ...configurationCompletionManifest.registryOrder.map(registry => ({
        id: 7,
        registry,
        elements: configurationCompletionManifest.elementCounts[registry],
        elementIds: configurationCompletionManifest.requiredElements[registry] ?? ['minecraft:placeholder']
      })),
      {
        id: 13,
        registries: Object.entries(configurationCompletionManifest.requiredTags).map(([registry, tags]) => ({
          registry,
          tags: Object.entries(tags).map(([tag, entries]) => ({ tag, entries }))
        }))
      },
      {
        id: 14,
        packs: configurationCompletionManifest.knownPacks
      },
      {
        id: configurationCompletionManifest.finishConfigurationPacketId
      }
    ],
    play: [
      { id: 49, length: 70 },
      { id: 105, length: 2 },
      { id: 72, length: 62 },
      { id: 12, length: 1 },
      { id: 48, length: 259 },
      { id: 11, length: 2 }
    ]
  }
}

test('configuration registry readiness gate validates raw probe registry and play-entry coverage', () => {
  const checks = evaluateManifestReadiness(configurationCompletionManifest, passingRawProbe())

  assert.deepEqual(checks.map(check => check.ok), checks.map(() => true))
  assert.deepEqual(
    checks.map(check => check.name),
    [
      'raw-26-registry-order',
      'raw-26-registry-element-counts',
      'raw-26-required-registry-elements',
      'raw-26-required-tags',
      'raw-26-known-pack-order',
      'raw-26-finish-configuration',
      'raw-26-play-entry-packets'
    ]
  )
})

test('configuration registry readiness gate fails on missing elements, tag drift, known-pack drift, and play decode drift', () => {
  const rawProbe = passingRawProbe()
  rawProbe.config.find(packet => packet.registry === 'minecraft:biome').elements = 64
  rawProbe.config.find(packet => packet.registry === 'minecraft:trim_material').elementIds = ['minecraft:iron']
  rawProbe.config.find(packet => packet.id === 13).registries
    .find(registry => registry.registry === 'minecraft:damage_type').tags
    .find(tag => tag.tag === 'minecraft:is_fire').entries = []
  rawProbe.config.find(packet => packet.id === 14).packs = [{ namespace: 'minecraft', id: 'wrong', version: '26.1.2' }]
  rawProbe.play.find(packet => packet.id === 72).length = 59

  const checks = evaluateManifestReadiness(configurationCompletionManifest, rawProbe)
  const failures = checks.filter(check => !check.ok).map(check => check.name)

  assert.deepEqual(failures, [
    'raw-26-registry-element-counts',
    'raw-26-required-registry-elements',
    'raw-26-required-tags',
    'raw-26-known-pack-order',
    'raw-26-play-entry-packets'
  ])
})

test('configuration registry readiness gate can run against a live server', async () => {
  const gate = await runConfigurationRegistryReadinessGate({
    host: process.env.RUSTCRAFT_HOST ?? '127.0.0.1',
    port: Number(process.env.RUSTCRAFT_PORT ?? 25565)
  })

  assert.equal(gate.ok, true, formatConfigurationRegistryReadinessGateReport(gate))
})
