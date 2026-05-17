import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { access, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

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
