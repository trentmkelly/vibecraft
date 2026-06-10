import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { access, rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
  offlineUuid,
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

test('raw 26.1.2 reconnect after clean save loads UUID-bound playerdata before spawn', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-reconnect-save-')
  const username = 'SavedReturn'
  const uuid = offlineUuid(username)
  const savedPosition = { x: -8.5, y: 84, z: 9.5, yaw: 270, pitch: -22.5 }
  const savedSlot = 2
  let server

  try {
    await writeOfflineServerFiles(root, { port, levelName: 'world' })
    server = await start(root, port)

    const changed = await runJoinProbe(port, username, {
      VIBECRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'movement,held_slot',
      VIBECRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify(savedPosition),
      VIBECRAFT_RAW_PROBE_HELD_SLOT: String(savedSlot)
    })
    assert.equal(changed.ok, true)

    await waitForFile(path.join(root, 'world', 'playerdata', `${uuid}.dat`))
    await stopServer(server.child)
    server = await start(root, port)

    const rejoined = await runJoinProbe(port, username, {
      VIBECRAFT_EXPECT_JOIN_POSITION: JSON.stringify(savedPosition),
      VIBECRAFT_EXPECT_HELD_SLOT: String(savedSlot)
    })
    assert.equal(rejoined.ok, true)
    assert.deepEqual(rejoined.joinState.position, savedPosition)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function start (root, port) {
  const server = startVibeCraft({ binary, root, port, levelName: 'world' })
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
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username,
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
