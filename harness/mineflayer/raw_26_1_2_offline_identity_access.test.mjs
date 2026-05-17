import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { access, mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
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
const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')
const host = '127.0.0.1'

test('raw 26.1.2 offline identity and access files gate login like vanilla surfaces', { timeout: 120_000 }, async () => {
  await withServer({ username: 'IdentityOK' }, async ({ port, root, username }) => {
    const joined = await runJoinProbe(port, username)
    assert.equal(joined.ok, true)
    assert.equal(joined.joinState.profile.name, username)
    assert.equal(joined.joinState.profile.uuid, offlineUuid(username))

    const usercache = JSON.parse(await readFile(path.join(root, 'usercache.json'), 'utf8'))
    assert.equal(usercache.length, 1)
    assert.equal(usercache[0].uuid, offlineUuid(username))
    assert.equal(usercache[0].name, username)
    assert.match(usercache[0].expiresOn, /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} \+0000$/)
  })

  await withServer({ username: 'CaseCache' }, async ({ port, root, username }) => {
    const lower = username.toLowerCase()
    const first = await runJoinProbe(port, username)
    const second = await runJoinProbe(port, lower)
    assert.equal(first.ok, true)
    assert.equal(second.ok, true)
    assert.equal(first.joinState.profile.uuid, offlineUuid(username))
    assert.equal(second.joinState.profile.uuid, offlineUuid(lower))
    assert.notEqual(first.joinState.profile.uuid, second.joinState.profile.uuid)

    const usercache = JSON.parse(await readFile(path.join(root, 'usercache.json'), 'utf8'))
    assert.deepEqual(
      usercache.map(entry => [entry.name, entry.uuid]).sort(),
      [
        [lower, offlineUuid(lower)],
        [username, offlineUuid(username)]
      ].sort()
    )
    assert.ok(usercache.every(entry => /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} \+0000$/.test(entry.expiresOn)))
  })

  await withServer({
    username: 'NoList',
    properties: { 'enforce-whitelist': 'true' },
    files: {
      'whitelist.json': JSON.stringify([profile('Listed')]),
      'ops.json': JSON.stringify([])
    }
  }, async ({ port, username }) => {
    const rejected = await runJoinProbe(port, username, { RUSTCRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(rejected.reason, /multiplayer\.disconnect\.not_whitelisted/)
  })

  await withServer({
    username: 'Listed',
    properties: { 'enforce-whitelist': 'true' },
    files: {
      'whitelist.json': JSON.stringify([profile('Listed')])
    }
  }, async ({ port, username }) => {
    const joined = await runJoinProbe(port, username)
    assert.equal(joined.ok, true)
    assert.equal(joined.joinState.profile.uuid, offlineUuid(username))
  })

  await withServer({
    username: 'OpBypass',
    properties: { 'enforce-whitelist': 'true' },
    files: {
      'whitelist.json': JSON.stringify([]),
      'ops.json': JSON.stringify([{ ...profile('OpBypass'), level: 4, bypassesPlayerLimit: false }])
    }
  }, async ({ port, username }) => {
    const joined = await runJoinProbe(port, username)
    assert.equal(joined.ok, true)
    assert.equal(joined.joinState.profile.uuid, offlineUuid(username))
  })

  await withServer({
    username: 'Banned',
    files: {
      'banned-players.json': JSON.stringify([{ ...profile('Banned'), created: '2026-05-17 00:00:00 +0000', source: 'Server', expires: 'forever', reason: 'test' }])
    }
  }, async ({ port, username }) => {
    const rejected = await runJoinProbe(port, username, { RUSTCRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(rejected.reason, /multiplayer\.disconnect\.banned/)
  })

  await withServer({
    username: 'IpBanned',
    files: {
      'banned-ips.json': JSON.stringify([{ ip: host, created: '2026-05-17 00:00:00 +0000', source: 'Server', expires: 'forever', reason: 'test' }])
    }
  }, async ({ port, username }) => {
    const rejected = await runJoinProbe(port, username, { RUSTCRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(rejected.reason, /multiplayer\.disconnect\.ip_banned/)
  })

  await withRestartableServer({
    username: 'Pardoned',
    files: {
      'banned-players.json': JSON.stringify([{ ...profile('Pardoned'), created: '2026-05-17 00:00:00 +0000', source: 'Server', expires: 'forever', reason: 'test' }])
    }
  }, async ({ port, root, username, restart }) => {
    const rejected = await runJoinProbe(port, username, { RUSTCRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(rejected.reason, /multiplayer\.disconnect\.banned/)

    await writeFile(path.join(root, 'banned-players.json'), '[]\n')
    await restart()

    const joined = await runJoinProbe(port, username)
    assert.equal(joined.ok, true)
    assert.equal(joined.joinState.profile.uuid, offlineUuid(username))
  })

  await withRestartableServer({
    username: 'IpPardoned',
    files: {
      'banned-ips.json': JSON.stringify([{ ip: host, created: '2026-05-17 00:00:00 +0000', source: 'Server', expires: 'forever', reason: 'test' }])
    }
  }, async ({ port, root, username, restart }) => {
    const rejected = await runJoinProbe(port, username, { RUSTCRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(rejected.reason, /multiplayer\.disconnect\.ip_banned/)

    await writeFile(path.join(root, 'banned-ips.json'), '[]\n')
    await restart()

    const joined = await runJoinProbe(port, username)
    assert.equal(joined.ok, true)
    assert.equal(joined.joinState.profile.uuid, offlineUuid(username))
  })

  await withRestartableServer({ username: 'HotEditBan' }, async ({ port, root, username, server }) => {
    const joined = await runJoinProbe(port, username)
    assert.equal(joined.ok, true)

    await writeFile(
      path.join(root, 'banned-players.json'),
      `${JSON.stringify([{ ...profile(username), created: '2026-05-17 00:00:00 +0000', source: 'Server', expires: 'forever', reason: 'hot edit' }])}\n`
    )
    server.child.stdin.write('reload\n')
    await delay(250)

    const rejected = await runJoinProbe(port, username, { RUSTCRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(rejected.reason, /multiplayer\.disconnect\.banned/)

    await writeFile(path.join(root, 'banned-players.json'), '[]\n')
    server.child.stdin.write('whitelist reload\n')
    await delay(250)

    const rejoined = await runJoinProbe(port, username)
    assert.equal(rejoined.ok, true)
  })

  await withRestartableServer({ username: 'PersistPos' }, async ({ port, username, restart }) => {
    const movedPosition = { x: 4.5, y: 81.0, z: -3.5, yaw: 90, pitch: 12.5 }
    const moved = await runJoinProbe(port, username, {
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement',
      RUSTCRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify(movedPosition),
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'first_tick_actions'
    })
    assert.equal(moved.ok, true)
    assert.equal(moved.aborted, true)

    await delay(250)
    await restart()

    const rejoined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(movedPosition)
    })
    assert.deepEqual(rejoined.joinState.position, movedPosition)
  })

  await withRestartableServer({ username: 'PersistSlot' }, async ({ port, username, restart }) => {
    const selectedSlot = 6
    const changed = await runJoinProbe(port, username, {
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'held_slot',
      RUSTCRAFT_RAW_PROBE_HELD_SLOT: String(selectedSlot),
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'first_tick_actions'
    })
    assert.equal(changed.ok, true)
    assert.equal(changed.aborted, true)

    await delay(250)
    await restart()

    const rejoined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_HELD_SLOT: String(selectedSlot)
    })
    assert.equal(rejoined.ok, true)
  })

  await withRestartableServer({ username: 'BadSlot' }, async ({ port, username, restart }) => {
    const changed = await runJoinProbe(port, username, {
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'held_slot',
      RUSTCRAFT_RAW_PROBE_HELD_SLOT: '12',
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'first_tick_actions'
    })
    assert.equal(changed.ok, true)
    assert.equal(changed.aborted, true)

    await delay(250)
    await restart()

    const rejoined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_HELD_SLOT: '0'
    })
    assert.equal(rejoined.ok, true)
  })

  await withRestartableServer({ username: 'LoadedStats' }, async ({ port, root, username }) => {
    const uuid = offlineUuid(username)
    const saved = {
      health: 13.5,
      foodLevel: 7,
      foodSaturation: 2.5,
      xpProgress: 0.375,
      xpLevel: 9,
      xpTotal: 123,
      selectedSlot: 3,
      gameMode: 1,
      previousGameMode: 0,
      position: { x: 2.5, y: 83, z: 4.5, yaw: 35, pitch: -10 }
    }
    await writePlayerData(root, uuid, saved)

    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(saved.position),
      RUSTCRAFT_EXPECT_HELD_SLOT: String(saved.selectedSlot),
      RUSTCRAFT_EXPECT_HEALTH: String(saved.health),
      RUSTCRAFT_EXPECT_FOOD_LEVEL: String(saved.foodLevel),
      RUSTCRAFT_EXPECT_FOOD_SATURATION: String(saved.foodSaturation),
      RUSTCRAFT_EXPECT_XP_PROGRESS: String(saved.xpProgress),
      RUSTCRAFT_EXPECT_XP_LEVEL: String(saved.xpLevel),
      RUSTCRAFT_EXPECT_XP_TOTAL: String(saved.xpTotal),
      RUSTCRAFT_EXPECT_GAME_MODE: String(saved.gameMode),
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: String(saved.previousGameMode),
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '13'
    })
    assert.equal(joined.ok, true)
  })

  await withRestartableServer({ username: 'ClampedStats' }, async ({ port, root, username }) => {
    const uuid = offlineUuid(username)
    const position = { x: 1.5, y: 80, z: 1.5, yaw: 0, pitch: 0 }
    await writePlayerData(root, uuid, {
      health: 27.25,
      foodLevel: 31,
      foodSaturation: 42.5,
      xpProgress: 1.5,
      xpLevel: -4,
      xpTotal: -99,
      selectedSlot: 42,
      gameMode: 99,
      previousGameMode: 99,
      position
    })

    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(position),
      RUSTCRAFT_EXPECT_HEALTH: '20',
      RUSTCRAFT_EXPECT_FOOD_LEVEL: '20',
      RUSTCRAFT_EXPECT_FOOD_SATURATION: '20',
      RUSTCRAFT_EXPECT_XP_PROGRESS: '1',
      RUSTCRAFT_EXPECT_XP_LEVEL: '0',
      RUSTCRAFT_EXPECT_XP_TOTAL: '0',
      RUSTCRAFT_EXPECT_HELD_SLOT: '0',
      RUSTCRAFT_EXPECT_GAME_MODE: '0',
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '0',
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '0'
    })
    assert.equal(joined.ok, true)
  })

  await withServer({
    username: 'DefaultCreative',
    properties: { gamemode: 'creative' }
  }, async ({ port, username }) => {
    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_GAME_MODE: '1',
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '255',
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '13'
    })
    assert.equal(joined.ok, true)
  })

  await withServer({
    username: 'InvalidMode',
    properties: { gamemode: 'not_a_mode' }
  }, async ({ port, username }) => {
    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_GAME_MODE: '0',
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '255',
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '0'
    })
    assert.equal(joined.ok, true)
  })

  await withServer({
    username: 'NumericMode',
    properties: { gamemode: '1' }
  }, async ({ port, username }) => {
    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_GAME_MODE: '1',
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '255',
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '13'
    })
    assert.equal(joined.ok, true)
  })

  await withServer({
    username: 'DefaultSpectator',
    properties: { gamemode: 'spectator' }
  }, async ({ port, root, username }) => {
    const joined = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_GAME_MODE: '3',
      RUSTCRAFT_EXPECT_PREVIOUS_GAME_MODE: '255',
      RUSTCRAFT_EXPECT_ABILITY_FLAGS: '15'
    })
    assert.equal(joined.ok, true)
    const uuid = offlineUuid(username)
    assert.equal(await readPlayerDataInt(root, uuid, 'playerGameType'), 3)
    assert.equal(await readPlayerDataInt(root, uuid, 'previousPlayerGameType'), undefined)
  })

  await withServer({
    username: 'ForcedAdventure',
    properties: { gamemode: 'adventure', 'force-gamemode': 'true' }
  }, async ({ port, root, username }) => {
    const uuid = offlineUuid(username)
    const position = { x: 3.5, y: 80, z: 3.5, yaw: 15, pitch: 5 }
    await writePlayerData(root, uuid, {
      health: 20,
      foodLevel: 20,
      foodSaturation: 5,
      xpProgress: 0,
      xpLevel: 0,
      xpTotal: 0,
      selectedSlot: 0,
      gameMode: 1,
      previousGameMode: 0,
      position
    })

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

  await withRestartableServer({ username: 'FreshSave' }, async ({ port, root, username, restart }) => {
    const uuid = offlineUuid(username)
    const playerdata = path.join(root, 'world', 'playerdata', `${uuid}.dat`)
    const advancements = path.join(root, 'world', 'advancements', `${uuid}.json`)
    const stats = path.join(root, 'world', 'stats', `${uuid}.json`)

    const aborted = await runJoinProbe(port, username, {
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'login_success'
    })
    assert.equal(aborted.ok, true)
    assert.equal(aborted.aborted, true)
    await delay(250)
    assert.equal(await exists(playerdata), false, 'login success alone must not create playerdata')

    await restart()
    const joined = await runJoinProbe(port, username)
    assert.equal(joined.ok, true)
    await delay(250)

    const bytes = await readFile(playerdata)
    assert.deepEqual([...bytes.subarray(0, 2)], [0x1f, 0x8b], 'playerdata must be gzip-compressed NBT')
    assert.equal(await exists(advancements), false, 'fresh raw login should not create advancements before advancement progress exists')
    assert.equal(await exists(stats), false, 'fresh raw login should not create stats before stat changes exist')
  })

  await withRestartableServer({ username: 'CorruptPD' }, async ({ port, root, username, restart }) => {
    const uuid = offlineUuid(username)
    const playerdata = path.join(root, 'world', 'playerdata', `${uuid}.dat`)
    const playerdataOld = path.join(root, 'world', 'playerdata', `${uuid}.dat_old`)
    const firstPosition = { x: 6.5, y: 81.0, z: 6.5, yaw: 10, pitch: 0 }
    const secondPosition = { x: 8.5, y: 82.0, z: -8.5, yaw: 45, pitch: -5 }

    await runJoinProbe(port, username, {
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement',
      RUSTCRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify(firstPosition),
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'first_tick_actions'
    })
    await delay(250)

    await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(firstPosition),
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement',
      RUSTCRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify(secondPosition),
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'first_tick_actions'
    })
    await delay(250)
    assert.equal(await exists(playerdataOld), true, 'second save should rotate primary playerdata to .dat_old')

    await writeFile(playerdata, 'corrupt playerdata')
    await restart()

    const recovered = await runJoinProbe(port, username, {
      RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(firstPosition)
    })
    assert.deepEqual(recovered.joinState.position, firstPosition)
    const backups = await corruptBackups(root, uuid)
    assert.equal(backups.length, 1)
  })

  for (const [label, username, initialUsercache] of [
    ['missing', 'CrbMiss', null],
    ['empty', 'CrbEmpty', '[]\n'],
    ['malformed', 'CrbBad', '[{\"uuid\":\"broken\"}\n'],
    ['stale', 'CrbStale', JSON.stringify([{ uuid: '00000000-0000-0000-0000-000000000099', name: 'CrbStale', expiresOn: '2000-01-01 00:00:00 +0000' }])],
    ['duplicate', 'CrbDup', JSON.stringify([
      { uuid: offlineUuid('CrbDup'), name: 'OldDuplicate', expiresOn: '2999-01-01 00:00:00 +0000' },
      { uuid: offlineUuid('CrbDup'), name: 'CrbDup', expiresOn: '2999-01-01 00:00:00 +0000' }
    ])]
  ]) {
    await withServer({
      username,
      files: initialUsercache == null ? {} : { 'usercache.json': initialUsercache }
    }, async ({ port, root }) => {
      try {
        const joined = await runJoinProbe(port, username)
        assert.equal(joined.ok, true)
        assert.equal(joined.joinState.profile.uuid, offlineUuid(username))

        const usercache = JSON.parse(await readFile(path.join(root, 'usercache.json'), 'utf8'))
        const matching = usercache.filter(entry => entry.name === username)
        assert.equal(matching.length, 1, `${label} cache should be repaired with one current entry`)
        assert.equal(matching[0].uuid, offlineUuid(username))
        assert.match(matching[0].expiresOn, /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} \+0000$/)
        assert.ok(
          !usercache.some(entry => entry.uuid === '00000000-0000-0000-0000-000000000099'),
          `${label} cache should drop stale bogus UUIDs`
        )
      } catch (error) {
        throw new Error(`${label} usercache repair failed for ${username}: ${error.message}`, { cause: error })
      }
    })
  }
})

async function withServer (options, callback) {
  await withRestartableServer(options, async context => {
    await callback(context)
  })
}

async function withRestartableServer (options, callback) {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-identity-')
  let server
  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: options.properties ?? {}
    })
    for (const [name, contents] of Object.entries(options.files ?? {})) {
      await writeFile(path.join(root, name), `${contents}\n`)
    }
    const restart = async () => {
      if (server) await stopServer(server.child)
      server = startRustCraft({ binary, root, port, levelName: 'world' })
      await waitForPort(port, host, 10_000)
    }
    await restart()
    await callback({ port, root, username: options.username, restart, server })
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
      cwd: new URL('.', import.meta.url),
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

function profile (name) {
  return { uuid: offlineUuid(name), name }
}

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function exists (file) {
  try {
    await access(file)
    return true
  } catch {
    return false
  }
}

async function corruptBackups (root, uuid) {
  const dir = path.join(root, 'world', 'playerdata')
  const entries = await readdir(dir)
  return entries.filter(name => name.startsWith(`${uuid}_corrupted_`) && name.endsWith('.dat'))
}

async function writePlayerData (root, uuid, saved) {
  const dir = path.join(root, 'world', 'playerdata')
  await mkdir(dir, { recursive: true })
  await writeFile(path.join(dir, `${uuid}.dat`), playerDataNbt(saved))
}

async function readPlayerDataInt (root, uuid, name) {
  const bytes = await readFile(path.join(root, 'world', 'playerdata', `${uuid}.dat`))
  return topLevelNbtInt(gunzipSync(bytes), name)
}

function topLevelNbtInt (buffer, wantedName) {
  let offset = 0
  assert.equal(buffer[offset++], 10)
  const rootNameLength = buffer.readUInt16BE(offset); offset += 2 + rootNameLength
  while (offset < buffer.length) {
    const type = buffer[offset++]
    if (type === 0) return undefined
    const nameLength = buffer.readUInt16BE(offset); offset += 2
    const name = buffer.toString('utf8', offset, offset + nameLength); offset += nameLength
    if (type === 3 && name === wantedName) return buffer.readInt32BE(offset)
    offset = skipNbtPayload(buffer, offset, type)
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
  if (saved.previousGameMode != null) {
    entries.push(nbtInt('previousPlayerGameType', saved.previousGameMode))
  }
  return gzipSync(nbtRoot(entries))
}

function nbtRoot (entries) {
  return Buffer.concat([Buffer.from([10, 0, 0]), ...entries, Buffer.from([0])])
}

function nbtNamed (type, name, payload) {
  return Buffer.concat([Buffer.from([type]), utf16Name(name), payload])
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
  return nbtNamed(8, name, Buffer.concat([utf16Name(value)]))
}

function nbtList (name, type, payloads) {
  const length = Buffer.alloc(4)
  length.writeInt32BE(payloads.length)
  return nbtNamed(9, name, Buffer.concat([Buffer.from([type]), length, ...payloads]))
}

function skipNbtPayload (buffer, offset, type) {
  switch (type) {
    case 1: return offset + 1
    case 3: return offset + 4
    case 5: return offset + 4
    case 6: return offset + 8
    case 8: {
      const length = buffer.readUInt16BE(offset)
      return offset + 2 + length
    }
    case 9: {
      const childType = buffer[offset++]
      const length = buffer.readInt32BE(offset); offset += 4
      for (let i = 0; i < length; i++) offset = skipNbtPayload(buffer, offset, childType)
      return offset
    }
    default:
      throw new Error(`unsupported test NBT type ${type}`)
  }
}

function utf16Name (value) {
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
