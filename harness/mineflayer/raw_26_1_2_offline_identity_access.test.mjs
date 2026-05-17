import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { readFile, rm, writeFile } from 'node:fs/promises'
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
})

async function withServer (options, callback) {
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
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)
    await callback({ port, root, username: options.username })
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
