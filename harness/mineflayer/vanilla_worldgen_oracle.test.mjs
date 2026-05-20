import assert from 'node:assert/strict'
import test from 'node:test'

import {
  buildVanillaWorldgenTraceReport,
  buildVanillaWorldgenOraclePlan,
  chunkToBlockCoord,
  chunkToRegionCoord,
  dimensionForceLoadCommandForChunk,
  forceLoadCommandForChunk,
  regionFileForChunk
} from './vanilla_worldgen_oracle.mjs'

test('chunk coordinates map to vanilla region file coordinates with floor division', () => {
  assert.equal(chunkToRegionCoord(0), 0)
  assert.equal(chunkToRegionCoord(31), 0)
  assert.equal(chunkToRegionCoord(32), 1)
  assert.equal(chunkToRegionCoord(-1), -1)
  assert.equal(chunkToRegionCoord(-32), -1)
  assert.equal(chunkToRegionCoord(-33), -2)

  assert.equal(regionFileForChunk({ x: 0, z: 0 }), 'world/dimensions/minecraft/overworld/region/r.0.0.mca')
  assert.equal(regionFileForChunk({ x: -1, z: 32 }), 'world/dimensions/minecraft/overworld/region/r.-1.1.mca')
  assert.equal(
    regionFileForChunk({ x: 64, z: -65, dimension: 'the_nether' }),
    'world/dimensions/minecraft/the_nether/region/r.2.-3.mca'
  )
  assert.equal(
    regionFileForChunk({ x: -64, z: 63, dimension: 'end' }),
    'world/dimensions/minecraft/the_end/region/r.-2.1.mca'
  )
})

test('vanilla worldgen oracle plan force-loads requested chunks and saves region artifacts', () => {
  const plan = buildVanillaWorldgenOraclePlan({
    seed: 12345n,
    levelName: 'parity',
    port: 25599,
    chunks: [
      { x: 0, z: 0 },
      { x: 1, z: 0 },
      { x: 32, z: -1 },
      { x: -1, z: 32 }
    ]
  })

  assert.equal(plan.seed, '12345')
  assert.equal(plan.levelName, 'parity')
  assert.equal(plan.port, 25599)
  assert.equal(chunkToBlockCoord(-2), -32)
  assert.equal(forceLoadCommandForChunk({ x: -2, z: 3 }), 'forceload add -32 48')
  assert.equal(
    dimensionForceLoadCommandForChunk({ x: -2, z: 3, dimension: 'the_nether' }),
    'execute in minecraft:the_nether run forceload add -32 48'
  )
  assert.equal(
    dimensionForceLoadCommandForChunk({ x: -2, z: 3, dimension: 'end' }),
    'execute in minecraft:the_end run forceload add -32 48'
  )
  assert.deepEqual(plan.commands, [
    'forceload add 0 0',
    'forceload add 16 0',
    'forceload add 512 -16',
    'forceload add -16 512',
    'save-all flush',
    'stop'
  ])
  assert.deepEqual(plan.regionFiles, [
    'parity/dimensions/minecraft/overworld/region/r.-1.1.mca',
    'parity/dimensions/minecraft/overworld/region/r.0.0.mca',
    'parity/dimensions/minecraft/overworld/region/r.1.-1.mca'
  ])
})

test('vanilla worldgen oracle rejects unsupported dimension artifact paths', () => {
  assert.throws(
    () => regionFileForChunk({ x: 0, z: 0, dimension: 'moon' }),
    /unsupported dimension/
  )
  assert.throws(
    () => dimensionForceLoadCommandForChunk({ x: 0, z: 0, dimension: 'moon' }),
    /unsupported dimension/
  )
})

test('vanilla worldgen trace report normalizes requested chunk evidence', () => {
  const report = buildVanillaWorldgenTraceReport({
    plan: {
      seed: '0',
      levelName: 'world',
      commands: ['forceload add 0 0', 'save-all flush', 'stop']
    },
    artifacts: [{
      path: 'world/dimensions/minecraft/overworld/region/r.0.0.mca',
      bytes: 8192,
      sha256: 'a'.repeat(64),
      chunkCount: 1,
      statusCounts: { 'minecraft:full': 1 },
      requestedChunks: [{
        chunkX: 0,
        chunkZ: 0,
        status: 'minecraft:full',
        sectionCount: 24,
        nonEmptySectionCount: 18,
        heightmaps: {
          WORLD_SURFACE: { type: 'long_array', entries: 37 }
        },
        structures: {
          startKeys: ['minecraft:village'],
          referenceKeys: ['minecraft:village']
        },
        blockPalette: ['minecraft:stone', 'minecraft:oak_log', 'minecraft:grass_block'],
        biomePalette: ['minecraft:plains'],
        payloadBytes: 1234,
        payloadSha256: 'b'.repeat(64),
        sections: [{
          y: 4,
          blockPalette: ['minecraft:stone', 'minecraft:oak_log'],
          blockStatesData: { entries: 256, sha256: 'c'.repeat(64) },
          biomePalette: ['minecraft:plains'],
          biomeData: { entries: 64, sha256: 'd'.repeat(64) }
        }]
      }]
    }]
  })

  assert.equal(report.format, 'rustcraft-vanilla-worldgen-trace-v1')
  assert.deepEqual(report.commandTrace, ['forceload add 0 0', 'save-all flush', 'stop'])
  assert.deepEqual(report.regionArtifacts[0].statusCounts, { 'minecraft:full': 1 })
  assert.deepEqual(report.requestedChunks[0], {
    dimension: 'overworld',
    chunkX: 0,
    chunkZ: 0,
    finalStatus: 'minecraft:full',
    heightmaps: {
      WORLD_SURFACE: { type: 'long_array', entries: 37 }
    },
    biomePalette: ['minecraft:plains'],
    sectionCount: 24,
    nonEmptySectionCount: 18,
    sectionPalettes: [{
      y: 4,
      blockPalette: ['minecraft:stone', 'minecraft:oak_log'],
      blockStatesData: { entries: 256, sha256: 'c'.repeat(64) },
      biomePalette: ['minecraft:plains'],
      biomeData: { entries: 64, sha256: 'd'.repeat(64) }
    }],
    structures: {
      startKeys: ['minecraft:village'],
      referenceKeys: ['minecraft:village']
    },
    featureBlockSamples: ['minecraft:grass_block', 'minecraft:oak_log'],
    serializedChunkNbt: {
      payloadBytes: 1234,
      payloadSha256: 'b'.repeat(64)
    }
  })
})
