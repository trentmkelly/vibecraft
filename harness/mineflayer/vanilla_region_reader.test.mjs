import assert from 'node:assert/strict'
import test from 'node:test'
import { deflateSync } from 'node:zlib'

import {
  readRegionBuffer,
  regionPosFromFileName,
  summarizeRegion
} from './vanilla_region_reader.mjs'

test('region reader decodes Anvil locations, zlib NBT payloads, and chunk summaries', () => {
  const chunkNbt = namedCompound('', [
    intTag('DataVersion', 4438),
    intTag('xPos', -1),
    intTag('zPos', -1),
    stringTag('Status', 'minecraft:full'),
    longTag('LastUpdate', 42n),
    longTag('InhabitedTime', 7n),
    compoundTag('Heightmaps', [
      longArrayTag('WORLD_SURFACE_WG', [1n, 2n]),
      longArrayTag('OCEAN_FLOOR_WG', [3n])
    ]),
    listTag('sections', 10, [
      compoundPayload([
        byteTag('Y', 0),
        compoundTag('block_states', [
          listTag('palette', 10, [
            compoundPayload([stringTag('Name', 'minecraft:stone')]),
            compoundPayload([stringTag('Name', 'minecraft:water')])
          ]),
          longArrayTag('data', [0n])
        ]),
        compoundTag('biomes', [
          listTag('palette', 8, [
            stringPayload('minecraft:plains'),
            stringPayload('minecraft:river')
          ])
        ])
      ])
    ]),
    compoundTag('structures', [
      compoundTag('starts', [compoundTag('minecraft:village', [])]),
      compoundTag('References', [longArrayTag('minecraft:village', [12n])])
    ])
  ])
  const compressed = deflateSync(chunkNbt)
  const chunkLength = compressed.length + 1
  const sectorCount = Math.ceil((chunkLength + 4) / 4096)
  const region = Buffer.alloc(8192 + sectorCount * 4096)
  region.writeUIntBE(2, 1023 * 4, 3)
  region.writeUInt8(sectorCount, 1023 * 4 + 3)
  region.writeUInt32BE(123456, 4096 + 1023 * 4)
  region.writeUInt32BE(chunkLength, 8192)
  region.writeUInt8(2, 8196)
  compressed.copy(region, 8197)

  const decoded = readRegionBuffer(region, '/tmp/world/dimensions/minecraft/overworld/region/r.-1.-1.mca')
  const summary = summarizeRegion(decoded)

  assert.deepEqual(decoded.regionPos, { x: -1, z: -1 })
  assert.equal(decoded.chunks.length, 1)
  assert.equal(decoded.chunks[0].localX, 31)
  assert.equal(decoded.chunks[0].localZ, 31)
  assert.equal(decoded.chunks[0].chunkX, -1)
  assert.equal(decoded.chunks[0].chunkZ, -1)
  assert.equal(decoded.chunks[0].compression, 2)
  assert.equal(summary.chunkCount, 1)
  assert.deepEqual(summary.chunks[0].heightmaps, {
    OCEAN_FLOOR_WG: { type: 'long_array', entries: 1 },
    WORLD_SURFACE_WG: { type: 'long_array', entries: 2 }
  })
  assert.deepEqual(summary.chunks[0].blockPalette, ['minecraft:stone', 'minecraft:water'])
  assert.deepEqual(summary.chunks[0].biomePalette, ['minecraft:plains', 'minecraft:river'])
  assert.equal(summary.chunks[0].sections.length, 1)
  assert.deepEqual(summary.chunks[0].sections[0].blockPalette, ['minecraft:stone', 'minecraft:water'])
  assert.equal(summary.chunks[0].sections[0].blockStatesData.entries, 1)
  assert.match(summary.chunks[0].sections[0].blockStatesData.sha256, /^[0-9a-f]{64}$/)
  assert.equal(summary.chunks[0].sections[0].biomeData.entries, 0)
  assert.equal(summary.chunks[0].sections[0].biomeData.sha256, null)
  assert.deepEqual(summary.chunks[0].structures.startKeys, ['minecraft:village'])
  assert.deepEqual(summary.chunks[0].structures.referenceKeys, ['minecraft:village'])
})

test('region file names use vanilla floor-divided coordinates', () => {
  assert.deepEqual(regionPosFromFileName('r.2.-3.mca'), { x: 2, z: -3 })
  assert.deepEqual(regionPosFromFileName('/tmp/not-a-region.dat'), { x: 0, z: 0 })
})

function namedCompound (name, fields) {
  return Buffer.concat([Buffer.from([10]), utf(name), ...fields, Buffer.from([0])])
}

function compoundPayload (fields) {
  return { payload: Buffer.concat([...fields, Buffer.from([0])]) }
}

function byteTag (name, value) {
  return Buffer.concat([Buffer.from([1]), utf(name), Buffer.from([value & 0xff])])
}

function intTag (name, value) {
  const payload = Buffer.alloc(4)
  payload.writeInt32BE(value)
  return Buffer.concat([Buffer.from([3]), utf(name), payload])
}

function longTag (name, value) {
  const payload = Buffer.alloc(8)
  payload.writeBigInt64BE(value)
  return Buffer.concat([Buffer.from([4]), utf(name), payload])
}

function stringTag (name, value) {
  return Buffer.concat([Buffer.from([8]), utf(name), stringPayload(value).payload])
}

function compoundTag (name, fields) {
  return Buffer.concat([Buffer.from([10]), utf(name), compoundPayload(fields).payload])
}

function listTag (name, elementType, values) {
  const length = Buffer.alloc(4)
  length.writeInt32BE(values.length)
  return Buffer.concat([Buffer.from([9]), utf(name), Buffer.from([elementType]), length, ...values.map(value => value.payload)])
}

function longArrayTag (name, values) {
  const length = Buffer.alloc(4)
  length.writeInt32BE(values.length)
  const payload = Buffer.alloc(values.length * 8)
  values.forEach((value, index) => payload.writeBigInt64BE(value, index * 8))
  return Buffer.concat([Buffer.from([12]), utf(name), length, payload])
}

function stringPayload (value) {
  return { payload: utf(value) }
}

function utf (value) {
  const bytes = Buffer.from(value, 'utf8')
  const length = Buffer.alloc(2)
  length.writeUInt16BE(bytes.length)
  return Buffer.concat([length, bytes])
}
