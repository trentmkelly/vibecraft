import assert from 'node:assert/strict'
import test from 'node:test'

import {
  buildVanillaWorldgenOraclePlan,
  chunkToBlockCoord,
  chunkToRegionCoord,
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
})
