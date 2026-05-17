import assert from 'node:assert/strict'
import test from 'node:test'

import {
  diffConfigurationTranscripts,
  normalizeConfigurationTranscript,
  recordServerConfigurationTranscript
} from './configuration_transcript_oracle.mjs'

test('normalizeConfigurationTranscript preserves registry order, tags, known packs, and play packet ids', () => {
  const transcript = normalizeConfigurationTranscript({
    config: [
      { id: 7, registry: 'minecraft:damage_type', elements: 1, elementIds: ['minecraft:generic'] },
      { id: 13, registries: [{ registry: 'minecraft:damage_type', tags: [{ tag: 'minecraft:is_fire', entries: [0] }] }] },
      { id: 14, packs: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }] },
      { id: 3 }
    ],
    play: [{ id: 49 }, { id: 72 }]
  })

  assert.deepEqual(transcript.registries.map(entry => entry.registry), ['minecraft:damage_type'])
  assert.deepEqual(transcript.tags[0].tags[0], { tag: 'minecraft:is_fire', entries: [0] })
  assert.deepEqual(transcript.knownPacks, [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }])
  assert.equal(transcript.finishConfigurationPacketId, 3)
  assert.deepEqual(transcript.playPacketIds, [49, 72])
})

test('diffConfigurationTranscripts reports first actionable registry and known-pack mismatches', () => {
  const actual = normalizeConfigurationTranscript({
    config: [
      { id: 7, registry: 'minecraft:damage_type', elements: 1, elementIds: ['minecraft:generic'] },
      { id: 14, packs: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }] },
      { id: 3 }
    ],
    play: []
  })
  const official = normalizeConfigurationTranscript({
    config: [
      { id: 7, registry: 'minecraft:damage_type', elements: 2, elementIds: ['minecraft:generic', 'minecraft:fall'] },
      { id: 14, packs: [{ namespace: 'minecraft', id: 'vanilla', version: '26.1.2' }] },
      { id: 3 }
    ],
    play: []
  })

  const diff = diffConfigurationTranscripts(actual, official)

  assert.equal(diff.ok, false)
  assert.deepEqual(diff.diffs.map(entry => entry.path), [
    'registry.minecraft:damage_type.elements',
    'registry.minecraft:damage_type.elementIds',
    'knownPacks'
  ])
})

test('recordServerConfigurationTranscript records the live RustCraft configuration transcript', async () => {
  const transcript = await recordServerConfigurationTranscript({
    host: process.env.RUSTCRAFT_HOST ?? '127.0.0.1',
    port: Number(process.env.RUSTCRAFT_PORT ?? 25565)
  })

  assert.equal(transcript.finishConfigurationPacketId, 3)
  assert.ok(transcript.registries.some(entry => entry.registry === 'minecraft:worldgen/biome'))
  assert.ok(transcript.knownPacks.some(pack => pack.namespace === 'minecraft' && pack.id === 'core'))
  assert.ok(transcript.playPacketIds.includes(49))
})
