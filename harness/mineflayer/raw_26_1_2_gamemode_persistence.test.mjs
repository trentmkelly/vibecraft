import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'
import { gunzipSync, gzipSync } from 'node:zlib'

import {
  createTempWorld,
  offlineUuid,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')
const host = '127.0.0.1'

test('raw 26.1.2 loads saved game mode and preserves previous mode', { timeout: 90_000 }, async () => {
  await withServer('rustcraft-gamemode-saved-', {
    properties: { gamemode: 'survival', 'force-gamemode': 'false' }
  }, async ({ port, root }) => {
    const username = 'SavedCreative'
    const uuid = offlineUuid(username)
    const position = { x: 4.5, y: 80, z: 4.5, yaw: 90, pitch: 10 }
    await writePlayerData(root, uuid, { gameMode: 1, previousGameMode: 0, position })

    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(position),
      RUSTCRAFT_EXPECT_GAME_MODE: '1',
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '0',
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '13'
    })

    assert.equal(joined.ok, true)
    assert.equal(await readPlayerDataInt(root, uuid, 'playerGameType'), 1)
    assert.equal(await readPlayerDataInt(root, uuid, 'previousPlayerGameType'), 0)
  })
})

test('raw 26.1.2 applies default spectator game mode to fresh profiles', { timeout: 90_000 }, async () => {
  await withServer('rustcraft-gamemode-spectator-', {
    properties: { gamemode: 'spectator' }
  }, async ({ port, root }) => {
    const username = 'FreshSpectator'
    const uuid = offlineUuid(username)
    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_GAME_MODE: '3',
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '255',
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '15'
    })

    assert.equal(joined.ok, true)
    assert.equal(await readPlayerDataInt(root, uuid, 'playerGameType'), 3)
    assert.equal(await readPlayerDataInt(root, uuid, 'previousPlayerGameType'), undefined)
  })
})

test('raw 26.1.2 force-gamemode overrides saved mode and persists effective mode', { timeout: 90_000 }, async () => {
  await withServer('rustcraft-gamemode-forced-', {
    properties: { gamemode: 'adventure', 'force-gamemode': 'true' }
  }, async ({ port, root }) => {
    const username = 'ForcedAdventure'
    const uuid = offlineUuid(username)
    const position = { x: 3.5, y: 80, z: 3.5, yaw: 15, pitch: 5 }
    await writePlayerData(root, uuid, { gameMode: 1, previousGameMode: 0, position })

    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(position),
      RUSTCRAFT_EXPECT_GAME_MODE: '2',
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '0',
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '0'
    })

    assert.equal(joined.ok, true)
    assert.equal(await readPlayerDataInt(root, uuid, 'playerGameType'), 2)
    assert.equal(await readPlayerDataInt(root, uuid, 'previousPlayerGameType'), 0)
  })
})

async function withServer (prefix, options, callback) {
  const port = await reservePort()
  const root = await createTempWorld(prefix)
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        'view-distance': '4',
        'simulation-distance': '4',
        ...(options.properties ?? {})
      }
    })
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)
    return await callback({ port, root })
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
}

async function runJoinProbe (port, username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_HOST: host,
        RUSTCRAFT_PORT: String(port),
        RUSTCRAFT_USERNAME: username,
        ...env
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function writePlayerData (root, uuid, saved) {
  const dir = path.join(root, 'world', 'playerdata')
  await mkdir(dir, { recursive: true })
  await writeFile(path.join(dir, `${uuid}.dat`), playerDataNbt({
    health: 20,
    foodLevel: 20,
    foodSaturation: 5,
    xpLevel: 0,
    xpProgress: 0,
    xpTotal: 0,
    selectedSlot: 0,
    ...saved
  }))
}

async function readPlayerDataInt (root, uuid, wantedName) {
  const data = gunzipSync(await readFile(path.join(root, 'world', 'playerdata', `${uuid}.dat`)))
  let offset = 0
  assert.equal(data[offset++], 10)
  const rootNameLength = data.readUInt16BE(offset); offset += 2 + rootNameLength
  while (offset < data.length) {
    const type = data[offset++]
    if (type === 0) break
    const nameLength = data.readUInt16BE(offset); offset += 2
    const name = data.toString('utf8', offset, offset + nameLength); offset += nameLength
    if (type === 3 && name === wantedName) return data.readInt32BE(offset)
    offset = skipNbtPayload(data, offset, type)
  }
  return undefined
}

function playerDataNbt (saved) {
  const entries = [
    nbtInt('DataVersion', 4791),
    nbtList('Pos', 6, [
      doublePayload(saved.position.x),
      doublePayload(saved.position.y),
      doublePayload(saved.position.z)
    ]),
    nbtList('Rotation', 5, [
      floatPayload(saved.position.yaw),
      floatPayload(saved.position.pitch)
    ]),
    nbtList('Motion', 6, [doublePayload(0), doublePayload(0), doublePayload(0)]),
    nbtByte('OnGround', 1),
    nbtFloat('Health', saved.health),
    nbtInt('foodLevel', saved.foodLevel),
    nbtFloat('foodSaturationLevel', saved.foodSaturation),
    nbtInt('XpLevel', saved.xpLevel),
    nbtFloat('XpP', saved.xpProgress),
    nbtInt('XpTotal', saved.xpTotal),
    nbtInt('SelectedItemSlot', saved.selectedSlot),
    nbtInt('playerGameType', saved.gameMode ?? 0),
    nbtString('Dimension', 'minecraft:overworld')
  ]
  if (saved.previousGameMode != null) entries.push(nbtInt('previousPlayerGameType', saved.previousGameMode))
  return gzipSync(Buffer.concat([Buffer.from([10, 0, 0]), ...entries, Buffer.from([0])]))
}

function nbtNamed (type, name, payload) {
  return Buffer.concat([Buffer.from([type]), stringPayload(name), payload])
}

function nbtByte (name, value) {
  return nbtNamed(1, name, Buffer.from([value & 0xff]))
}

function nbtInt (name, value) {
  const payload = Buffer.alloc(4)
  payload.writeInt32BE(value)
  return nbtNamed(3, name, payload)
}

function nbtFloat (name, value) {
  return nbtNamed(5, name, floatPayload(value))
}

function nbtString (name, value) {
  return nbtNamed(8, name, stringPayload(value))
}

function nbtList (name, type, payloads) {
  const length = Buffer.alloc(4)
  length.writeInt32BE(payloads.length)
  return nbtNamed(9, name, Buffer.concat([Buffer.from([type]), length, ...payloads]))
}

function skipNbtPayload (buffer, offset, type) {
  switch (type) {
    case 1: return offset + 1 // Byte
    case 2: return offset + 2 // Short (e.g. vanilla "Air")
    case 3: return offset + 4 // Int
    case 4: return offset + 8 // Long
    case 5: return offset + 4 // Float
    case 6: return offset + 8 // Double
    case 7: { // ByteArray
      const length = buffer.readInt32BE(offset)
      return offset + 4 + length
    }
    case 8: { // String
      const length = buffer.readUInt16BE(offset)
      return offset + 2 + length
    }
    case 9: { // List
      const childType = buffer[offset++]
      const length = buffer.readInt32BE(offset); offset += 4
      for (let i = 0; i < length; i++) offset = skipNbtPayload(buffer, offset, childType)
      return offset
    }
    case 10: // Compound
      return skipNbtCompound(buffer, offset)
    case 11: { // IntArray
      const length = buffer.readInt32BE(offset)
      return offset + 4 + length * 4
    }
    case 12: { // LongArray
      const length = buffer.readInt32BE(offset)
      return offset + 4 + length * 8
    }
    default:
      throw new Error(`unsupported test NBT type ${type}`)
  }
}

function skipNbtCompound (buffer, offset) {
  while (offset < buffer.length) {
    const type = buffer[offset++]
    if (type === 0) return offset
    const nameLength = buffer.readUInt16BE(offset); offset += 2 + nameLength
    offset = skipNbtPayload(buffer, offset, type)
  }
  return offset
}

function stringPayload (value) {
  const data = Buffer.from(value, 'utf8')
  const length = Buffer.alloc(2)
  length.writeUInt16BE(data.length)
  return Buffer.concat([length, data])
}

function floatPayload (value) {
  const payload = Buffer.alloc(4)
  payload.writeFloatBE(value)
  return payload
}

function doublePayload (value) {
  const payload = Buffer.alloc(8)
  payload.writeDoubleBE(value)
  return payload
}

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}
