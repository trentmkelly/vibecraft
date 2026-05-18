import { createHash } from 'node:crypto'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { gunzipSync, inflateSync } from 'node:zlib'

const SECTOR_BYTES = 4096
const HEADER_BYTES = 8192

export async function readRegionFile (filePath) {
  return readRegionBuffer(await readFile(filePath), filePath)
}

export function readRegionBuffer (buffer, filePath = 'r.0.0.mca') {
  if (buffer.length < HEADER_BYTES) {
    throw new Error(`region file too small: ${buffer.length} bytes`)
  }

  const regionPos = regionPosFromFileName(filePath)
  const chunks = []
  for (let index = 0; index < 1024; index++) {
    const locationOffset = index * 4
    const sectorOffset = buffer.readUIntBE(locationOffset, 3)
    const sectorCount = buffer.readUInt8(locationOffset + 3)
    if (sectorOffset === 0 || sectorCount === 0) continue

    const chunkOffset = sectorOffset * SECTOR_BYTES
    const allocatedBytes = sectorCount * SECTOR_BYTES
    if (chunkOffset + 5 > buffer.length) {
      throw new Error(`chunk location ${index} points past end of region file`)
    }

    const length = buffer.readUInt32BE(chunkOffset)
    const compression = buffer.readUInt8(chunkOffset + 4)
    const compressed = buffer.subarray(chunkOffset + 5, chunkOffset + 4 + length)
    const payload = decompressRegionPayload(compression, compressed)
    const nbt = new NbtReader(payload).readNamedRoot()
    const localX = index % 32
    const localZ = Math.floor(index / 32)

    chunks.push({
      index,
      localX,
      localZ,
      chunkX: regionPos.x * 32 + localX,
      chunkZ: regionPos.z * 32 + localZ,
      sectorOffset,
      sectorCount,
      allocatedBytes,
      compressedBytes: compressed.length,
      payloadBytes: payload.length,
      compression,
      payloadSha256: createHash('sha256').update(payload).digest('hex'),
      nbt,
      summary: summarizeChunkNbt(nbt.value)
    })
  }
  return { path: filePath, regionPos, chunks }
}

export function summarizeRegion (region) {
  return {
    path: region.path,
    regionPos: region.regionPos,
    chunkCount: region.chunks.length,
    chunks: region.chunks.map(chunk => ({
      index: chunk.index,
      localX: chunk.localX,
      localZ: chunk.localZ,
      chunkX: chunk.summary.xPos ?? chunk.chunkX,
      chunkZ: chunk.summary.zPos ?? chunk.chunkZ,
      status: chunk.summary.status,
      lastUpdate: chunk.summary.lastUpdate,
      inhabitedTime: chunk.summary.inhabitedTime,
      sectionCount: chunk.summary.sectionCount,
      nonEmptySectionCount: chunk.summary.nonEmptySectionCount,
      heightmaps: chunk.summary.heightmaps,
      structures: chunk.summary.structures,
      sections: chunk.summary.sections,
      blockPalette: chunk.summary.blockPalette,
      biomePalette: chunk.summary.biomePalette,
      payloadBytes: chunk.payloadBytes,
      payloadSha256: chunk.payloadSha256
    }))
  }
}

export function summarizeChunkNbt (root) {
  const sections = arrayValue(compoundField(root, 'sections')) ?? []
  const heightmaps = Object.fromEntries(
    compoundEntries(compoundField(root, 'Heightmaps'))
      .map(([name, tag]) => [name, { type: tag.type, entries: arrayValue(tag)?.length ?? 0 }])
      .sort(([left], [right]) => left.localeCompare(right))
  )
  return {
    dataVersion: numericValue(compoundField(root, 'DataVersion')),
    xPos: numericValue(compoundField(root, 'xPos')),
    zPos: numericValue(compoundField(root, 'zPos')),
    yPos: numericValue(compoundField(root, 'yPos')),
    status: stringValue(compoundField(root, 'Status')),
    lastUpdate: numericValue(compoundField(root, 'LastUpdate')),
    inhabitedTime: numericValue(compoundField(root, 'InhabitedTime')),
    sectionCount: sections.length,
    nonEmptySectionCount: sections.filter(section => blockPaletteNames(section).length > 0).length,
    heightmaps,
    structures: summarizeStructures(compoundField(root, 'structures')),
    sections: sections.map(summarizeSection),
    blockPalette: uniqueSorted(sections.flatMap(blockPaletteNames)),
    biomePalette: uniqueSorted(sections.flatMap(biomePaletteNames))
  }
}

export function regionPosFromFileName (filePath) {
  const match = path.basename(filePath).match(/^r\.(-?\d+)\.(-?\d+)\.mca$/)
  if (!match) return { x: 0, z: 0 }
  return { x: Number(match[1]), z: Number(match[2]) }
}

function decompressRegionPayload (compression, payload) {
  switch (compression) {
    case 1:
      return gunzipSync(payload)
    case 2:
      return inflateSync(payload)
    case 3:
      return payload
    default:
      throw new Error(`unsupported region chunk compression id ${compression}`)
  }
}

function summarizeStructures (tag) {
  const starts = compoundField(tag, 'starts')
  const references = compoundField(tag, 'References') ?? compoundField(tag, 'references')
  return {
    startKeys: compoundEntries(starts).map(([name]) => name).sort(),
    referenceKeys: compoundEntries(references).map(([name]) => name).sort()
  }
}

function summarizeSection (section) {
  const blockStates = compoundField(section, 'block_states')
  const biomes = compoundField(section, 'biomes')
  return {
    y: numericValue(compoundField(section, 'Y')),
    blockPalette: blockPaletteNames(section),
    blockStatesData: summarizePackedArray(compoundField(blockStates, 'data')),
    biomePalette: biomePaletteNames(section),
    biomeData: summarizePackedArray(compoundField(biomes, 'data'))
  }
}

function blockPaletteNames (section) {
  const palette = arrayValue(compoundField(compoundField(section, 'block_states'), 'palette')) ?? []
  return palette.map(entry => stringValue(compoundField(entry, 'Name'))).filter(Boolean)
}

function biomePaletteNames (section) {
  const palette = arrayValue(compoundField(compoundField(section, 'biomes'), 'palette')) ?? []
  return palette.map(stringValue).filter(Boolean)
}

function uniqueSorted (values) {
  return [...new Set(values)].sort()
}

function summarizePackedArray (tag) {
  const values = arrayValue(tag)
  if (!values) return { entries: 0, sha256: null }
  const body = JSON.stringify(values)
  return {
    entries: values.length,
    sha256: createHash('sha256').update(body).digest('hex')
  }
}

function compoundField (tag, name) {
  if (!tag || tag.type !== 'compound') return undefined
  return tag.value[name]
}

function compoundEntries (tag) {
  if (!tag || tag.type !== 'compound') return []
  return Object.entries(tag.value)
}

function arrayValue (tag) {
  if (!tag) return undefined
  if (tag.type === 'list' || tag.type === 'byte_array' || tag.type === 'int_array' || tag.type === 'long_array') {
    return tag.value
  }
  return undefined
}

function stringValue (tag) {
  return tag?.type === 'string' ? tag.value : undefined
}

function numericValue (tag) {
  if (!tag) return undefined
  if (tag.type === 'long') return Number(tag.value)
  return ['byte', 'short', 'int', 'float', 'double'].includes(tag.type) ? tag.value : undefined
}

class NbtReader {
  constructor (buffer) {
    this.buffer = buffer
    this.offset = 0
  }

  readNamedRoot () {
    const type = this.readU8()
    if (type === 0) throw new Error('NBT root cannot be TAG_End')
    const name = this.readString()
    return { name, value: this.readPayload(type) }
  }

  readPayload (type) {
    switch (type) {
      case 1:
        return { type: 'byte', value: this.readI8() }
      case 2:
        return { type: 'short', value: this.readI16() }
      case 3:
        return { type: 'int', value: this.readI32() }
      case 4:
        return { type: 'long', value: this.readI64() }
      case 5:
        return { type: 'float', value: this.readF32() }
      case 6:
        return { type: 'double', value: this.readF64() }
      case 7:
        return { type: 'byte_array', value: this.readByteArray() }
      case 8:
        return { type: 'string', value: this.readString() }
      case 9:
        return this.readList()
      case 10:
        return this.readCompound()
      case 11:
        return { type: 'int_array', value: this.readIntArray() }
      case 12:
        return { type: 'long_array', value: this.readLongArray() }
      default:
        throw new Error(`unsupported NBT tag id ${type}`)
    }
  }

  readList () {
    const elementType = this.readU8()
    const length = this.readI32()
    if (length < 0) throw new Error(`negative NBT list length ${length}`)
    const value = []
    for (let index = 0; index < length; index++) {
      value.push(this.readPayload(elementType))
    }
    return { type: 'list', elementType, value }
  }

  readCompound () {
    const value = {}
    while (true) {
      const type = this.readU8()
      if (type === 0) break
      const name = this.readString()
      value[name] = this.readPayload(type)
    }
    return { type: 'compound', value }
  }

  readByteArray () {
    const length = this.readI32()
    if (length < 0) throw new Error(`negative NBT byte array length ${length}`)
    const bytes = [...this.buffer.subarray(this.offset, this.offset + length)]
    this.offset += length
    return bytes
  }

  readIntArray () {
    const length = this.readI32()
    if (length < 0) throw new Error(`negative NBT int array length ${length}`)
    return Array.from({ length }, () => this.readI32())
  }

  readLongArray () {
    const length = this.readI32()
    if (length < 0) throw new Error(`negative NBT long array length ${length}`)
    return Array.from({ length }, () => this.readI64().toString())
  }

  readString () {
    const length = this.readU16()
    const value = this.buffer.toString('utf8', this.offset, this.offset + length)
    this.offset += length
    return value
  }

  readU8 () {
    return this.buffer.readUInt8(this.offset++)
  }

  readI8 () {
    return this.buffer.readInt8(this.offset++)
  }

  readU16 () {
    const value = this.buffer.readUInt16BE(this.offset)
    this.offset += 2
    return value
  }

  readI16 () {
    const value = this.buffer.readInt16BE(this.offset)
    this.offset += 2
    return value
  }

  readI32 () {
    const value = this.buffer.readInt32BE(this.offset)
    this.offset += 4
    return value
  }

  readI64 () {
    const value = this.buffer.readBigInt64BE(this.offset)
    this.offset += 8
    return value
  }

  readF32 () {
    const value = this.buffer.readFloatBE(this.offset)
    this.offset += 4
    return value
  }

  readF64 () {
    const value = this.buffer.readDoubleBE(this.offset)
    this.offset += 8
    return value
  }
}
