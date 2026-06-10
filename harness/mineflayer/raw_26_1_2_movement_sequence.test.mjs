import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
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

test('raw 26.1.2 movement sequence stays in play and streams final terrain window', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-movement-sequence-')
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      seed: 24680,
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const movements = [
      { x: 4.5, y: 80, z: 1.5, yaw: 5, pitch: 0 },
      { x: 15.5, y: 80, z: 1.5, yaw: 20, pitch: 0 },
      { x: 16.5, y: 80, z: 1.5, yaw: 25, pitch: 0 },
      { x: 32.5, y: 80, z: -16.5, yaw: 45, pitch: 5 }
    ]
    const moved = await runJoinProbe(port, 'MoveTrace', {
      VIBECRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: 'client_information,movement,player_input',
      VIBECRAFT_RAW_PROBE_MOVEMENT_POSITION: JSON.stringify({ x: 1.5, y: 80, z: 1.5, yaw: 0, pitch: 0 }),
      VIBECRAFT_RAW_PROBE_EXTRA_MOVEMENTS: JSON.stringify(movements),
      VIBECRAFT_RAW_PROBE_POST_ACTION_MS: '1000'
    })

    assert.equal(moved.ok, true)
    assert.deepEqual(moved.joinState.dynamicChunkStreaming.cacheCenter, { x: 2, z: -2 })
    assert.equal(moved.joinState.dynamicChunkStreaming.batchSize, 9)
    assert.equal(moved.joinState.dynamicChunkStreaming.chunks.length, 9)
    assert.equal(moved.joinState.dynamicChunkStreaming.forgottenChunks.length > 0, true)
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
