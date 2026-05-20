import assert from 'node:assert/strict'
import { mkdtemp, readFile, rm } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import {
  diffConfigurationTranscripts,
  normalizeConfigurationTranscript,
  recordServerConfigurationTranscript,
  writeOfficialConfigurationTranscriptFixture
} from './configuration_transcript_oracle.mjs'

test('normalizeConfigurationTranscript preserves registry order, tags, known packs, and play packet ids', () => {
  const transcript = normalizeConfigurationTranscript({
    config: [
      { id: 12, length: 3, features: ['minecraft:vanilla'], connectionId: 'volatile', timestamp: 123 },
      { id: 7, registry: 'minecraft:damage_type', elements: 1, elementIds: ['minecraft:generic'] },
      { id: 13, registries: [{ registry: 'minecraft:damage_type', tags: [{ tag: 'minecraft:is_fire', entries: [0] }] }] },
      { id: 14, packs: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }] },
      { id: 3 }
    ],
    play: [{ id: 49 }, { id: 72 }]
  })

  assert.equal(transcript.enabledFeaturesPacketId, 12)
  assert.deepEqual(transcript.enabledFeatures, ['minecraft:vanilla'])
  assert.deepEqual(transcript.registries.map(entry => entry.registry), ['minecraft:damage_type'])
  assert.deepEqual(transcript.tags[0].tags[0], { tag: 'minecraft:is_fire', entries: [0] })
  assert.deepEqual(transcript.knownPacks, [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }])
  assert.equal(transcript.finishConfigurationPacketId, 3)
  assert.deepEqual(transcript.playPacketIds, [49, 72])
  assert.equal('connectionId' in transcript, false)
  assert.equal('timestamp' in transcript, false)
})

test('diffConfigurationTranscripts reports first actionable registry and known-pack mismatches', () => {
  const actual = normalizeConfigurationTranscript({
    config: [
      { id: 7, registry: 'minecraft:damage_type', elements: 1, elementIds: ['minecraft:generic'], elementDataFields: { 'minecraft:generic': ['message_id'] } },
      { id: 13, registries: [{ registry: 'minecraft:damage_type', tags: [{ tag: 'minecraft:is_fire', entries: [0] }] }] },
      { id: 14, packs: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }] },
      { id: 3 }
    ],
    play: []
  })
  const official = normalizeConfigurationTranscript({
    config: [
      { id: 7, registry: 'minecraft:damage_type', elements: 2, elementIds: ['minecraft:generic', 'minecraft:fall'], elementDataFields: { 'minecraft:generic': ['message_id', 'scaling'] } },
      { id: 13, registries: [{ registry: 'minecraft:damage_type', tags: [{ tag: 'minecraft:is_fire', entries: [0, 1] }] }] },
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
    'registry.minecraft:damage_type.element.minecraft:generic.fieldPaths',
    'tags.minecraft:damage_type.minecraft:is_fire.entries',
    'knownPacks'
  ])
})

test('writeOfficialConfigurationTranscriptFixture stores only the normalized fixture shape', async () => {
  const tmp = await mkdtemp(path.join(os.tmpdir(), 'rustcraft-transcript-fixture-'))
  try {
    const fixturePath = path.join(tmp, 'official.json')
    const transcript = normalizeConfigurationTranscript({
      config: [
        { id: 12, length: 4, features: ['minecraft:vanilla'], tempPath: tmp },
        { id: 7, registry: 'minecraft:worldgen/biome', elements: 1, elementIds: ['minecraft:plains'], elementDataFields: { 'minecraft:plains': ['effects.water_color'] } },
        { id: 13, registries: [{ registry: 'minecraft:damage_type', tags: [{ tag: 'minecraft:is_fire', entries: [] }] }] },
        { id: 14, packs: [{ namespace: 'minecraft', id: 'core', version: '26.1.2' }] },
        { id: 3, randomUsername: 'OfficialProbe123' }
      ],
      play: [{ id: 49 }]
    })

    const result = await writeOfficialConfigurationTranscriptFixture({ fixturePath, transcript })
    const saved = JSON.parse(await readFile(fixturePath, 'utf8'))

    assert.equal(result.fixturePath, fixturePath)
    assert.deepEqual(saved, transcript)
    assert.equal(saved.enabledFeaturesPacketId, 12)
    assert.deepEqual(saved.enabledFeatures, ['minecraft:vanilla'])
    assert.equal(saved.registries[0].elementFieldPaths['minecraft:plains'][0], 'effects.water_color')
    assert.equal(JSON.stringify(saved).includes(tmp), false)
    assert.equal(JSON.stringify(saved).includes('OfficialProbe123'), false)
  } finally {
    await rm(tmp, { recursive: true, force: true })
  }
})

test('recordServerConfigurationTranscript records the live RustCraft configuration transcript', {
  skip: process.env.RUSTCRAFT_RUN_LIVE_TRANSCRIPT_TEST !== '1'
}, async () => {
  const transcript = await recordServerConfigurationTranscript({
    host: process.env.RUSTCRAFT_HOST ?? '127.0.0.1',
    port: Number(process.env.RUSTCRAFT_PORT ?? 25565)
  })

  assert.equal(transcript.finishConfigurationPacketId, 3)
  assert.ok(transcript.registries.some(entry => entry.registry === 'minecraft:worldgen/biome'))
  assert.ok(transcript.knownPacks.some(pack => pack.namespace === 'minecraft' && pack.id === 'core'))
  assert.ok(transcript.playPacketIds.includes(49))
})
