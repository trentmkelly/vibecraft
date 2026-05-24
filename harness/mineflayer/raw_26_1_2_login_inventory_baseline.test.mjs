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

test('raw 26.1.2 fresh login receives vanilla empty inventory baseline', { timeout: 45_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-login-inventory-')
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
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const joined = await runJoinProbe(port, 'InvBaseline', {
      RUSTCRAFT_EXPECT_HELD_SLOT: '0'
    })

    assert.equal(joined.ok, true)
    assert.ok(joined.play.some(packet => packet.id === 18), 'set_container_content should initialize player inventory')
    assert.ok(joined.play.some(packet => packet.id === 96), 'set_cursor_item should clear carried cursor item')
    assert.ok(joined.play.some(packet => packet.id === 105), 'set_held_slot should select the first hotbar slot')
    assert.ok(joined.joinState.initialChunkCount > 0, 'fresh login should receive an initial chunk batch')
    assert.equal(joined.joinState.lastReceivedChunk, joined.joinState.initialChunkCount - 1)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

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
