import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { access, rm, unlink } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
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
const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')
const host = '127.0.0.1'

test('raw 26.1.2 playerdata ownership stays bound to offline UUID files', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-playerdata-owner-')
  const alpha = { username: 'OwnAlpha', position: { x: 11.5, y: 82, z: 1.5, yaw: 45, pitch: 5 } }
  const beta = { username: 'OwnBeta', position: { x: -12.5, y: 83, z: -2.5, yaw: 180, pitch: -15 } }
  let server

  try {
    await writeOfflineServerFiles(root, { port, levelName: 'world' })
    server = await start(root, port)

    await saveMovedProfile(port, alpha)
    await saveMovedProfile(port, beta)
    await waitForFile(playerDataFile(root, alpha.username))
    await waitForFile(playerDataFile(root, beta.username))

    await stopServer(server.child)
    server = await start(root, port)

    await expectPosition(port, alpha.username, alpha.position)
    await expectPosition(port, beta.username, beta.position)

    await unlink(playerDataFile(root, alpha.username))
    await stopServer(server.child)
    server = await start(root, port)

    await expectPosition(port, alpha.username, alpha.position)
    await expectPosition(port, beta.username, beta.position)

    await unlink(playerDataFile(root, alpha.username))
    await unlink(playerDataOldFile(root, alpha.username))
    await stopServer(server.child)
    server = await start(root, port)

    await expectPosition(port, alpha.username, { x: 0.5, y: 80, z: 0.5, yaw: 0, pitch: 0 })
    await expectPosition(port, beta.username, beta.position)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function saveMovedProfile (port, profile) {
  const changed = await runJoinProbe(port, profile.username, {
    RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement',
    RUSTCRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify(profile.position)
  })
  assert.equal(changed.ok, true)
}

async function expectPosition (port, username, position) {
  const joined = await runJoinProbe(port, username, {
    RUSTCRAFT_EXPECT_JOIN_POSITION: JSON.stringify(position)
  })
  assert.equal(joined.ok, true)
  assert.deepEqual(joined.joinState.position, position)
}

function playerDataFile (root, username) {
  return path.join(root, 'world', 'playerdata', `${offlineUuid(username)}.dat`)
}

function playerDataOldFile (root, username) {
  return path.join(root, 'world', 'playerdata', `${offlineUuid(username)}.dat_old`)
}

async function start (root, port) {
  const server = startRustCraft({ binary, root, port, levelName: 'world' })
  await waitForPort(port, host, 10_000)
  return server
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

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.listen(0, host, resolve)
    server.once('error', reject)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}

async function waitForFile (file) {
  const deadline = Date.now() + 5_000
  while (Date.now() < deadline) {
    try {
      await access(file)
      return
    } catch {
      await delay(50)
    }
  }
  throw new Error(`timed out waiting for ${file}`)
}

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}
