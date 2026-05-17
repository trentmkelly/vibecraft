import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
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

test('raw 26.1.2 selected inventory slot persists and rejects invalid hotbar slots', { timeout: 75_000 }, async () => {
  await withServer('rustcraft-slot-persist-', async ({ port, restart }) => {
    const selectedSlot = 6
    const changed = await runJoinProbe(port, 'SlotPersist', {
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'held_slot',
      RUSTCRAFT_RAW_PROBE_HELD_SLOT: String(selectedSlot),
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'first_tick_actions'
    })
    assert.equal(changed.ok, true)
    assert.equal(changed.aborted, true)

    await delay(250)
    await restart()

    const rejoined = await runJoinProbe(port, 'SlotPersist', {
      RUSTCRAFT_EXPECT_HELD_SLOT: String(selectedSlot)
    })
    assert.equal(rejoined.ok, true)
  })

  await withServer('rustcraft-slot-correct-', async ({ port, restart }) => {
    const invalid = await runJoinProbe(port, 'SlotCorrect', {
      RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'held_slot',
      RUSTCRAFT_RAW_PROBE_HELD_SLOT: '12',
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'first_tick_actions'
    })
    assert.equal(invalid.ok, true)
    assert.equal(invalid.aborted, true)

    await delay(250)
    await restart()

    const rejoined = await runJoinProbe(port, 'SlotCorrect', {
      RUSTCRAFT_EXPECT_HELD_SLOT: '0'
    })
    assert.equal(rejoined.ok, true)
  })
})

async function withServer (prefix, callback) {
  const port = await reservePort()
  const root = await createTempWorld(prefix)
  let server

  async function restart () {
    if (server) await stopServer(server.child)
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)
  }

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    await restart()
    await callback({ port, root, restart })
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

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}
