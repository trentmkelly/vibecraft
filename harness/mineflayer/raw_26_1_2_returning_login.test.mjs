import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
import { existsSync } from 'node:fs'
import { rm } from 'node:fs/promises'
import { createServer } from 'node:net'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
  startVibeCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'vibecraft')
const host = '127.0.0.1'
const savedPosition = { x: 4.5, y: 96, z: -3.5, yaw: 45, pitch: 10 }

test('raw 26.1.2 returning offline login loads saved playerdata instead of first-join defaults', { timeout: 75_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-returning-login-')
  const username = process.env.VIBECRAFT_RETURNING_USERNAME ?? `Ret${crypto.randomUUID().replaceAll('-', '').slice(0, 10)}`
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const first = await runJoinProbe(port, username, {
      VIBECRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement',
      VIBECRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify(savedPosition),
      VIBECRAFT_RAW_PROBE_POST_ACTION_MS: '250'
    })
    assert.equal(first.ok, true)
    assert.equal(first.joinState.profile.name, username)
    assert.equal(first.joinState.profile.uuid, offlineUuid(username))
    assert.equal(first.joinState.entityId, 1)
    assert.ok(first.joinState.position.y > -64)
    assert.equal(first.joinState.position.yaw, 0)
    assert.equal(first.joinState.position.pitch, 0)
    assert.notDeepEqual(first.joinState.position, savedPosition)

    await delay(250)
    const playerdata = path.join(root, 'world', 'playerdata', `${first.joinState.profile.uuid}.dat`)
    assert.equal(existsSync(playerdata), true, `expected saved playerdata at ${playerdata}`)

    const second = await runJoinProbe(port, username)
    assert.equal(second.ok, true)
    assert.equal(second.joinState.profile.name, username)
    assert.equal(second.joinState.profile.uuid, first.joinState.profile.uuid)
    assert.equal(second.compressionThreshold, first.compressionThreshold)
    assert.equal(second.joinState.entityId, first.joinState.entityId)
    assertSavedPosition(second.joinState.position, savedPosition)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runJoinProbe (port, name, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: name,
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
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}

function assertSavedPosition (actual, expected) {
  assert.equal(actual.x, expected.x)
  assert.equal(actual.y, expected.y)
  assert.equal(actual.z, expected.z)
  assert.equal(actual.yaw, expected.yaw)
  assert.equal(actual.pitch, expected.pitch)
}

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function offlineUuid (name) {
  const hash = crypto.createHash('md5').update(`OfflinePlayer:${name}`, 'utf8').digest()
  hash[6] = (hash[6] & 0x0f) | 0x30
  hash[8] = (hash[8] & 0x3f) | 0x80
  const hex = hash.toString('hex')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}
