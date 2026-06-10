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

test('raw 26.1.2 slow initial chunk path keeps play connection alive until terrain is ready', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-slow-initial-chunk-')
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      seed: 424242,
      properties: {
        'view-distance': '4',
        'simulation-distance': '4'
      }
    })
    server = startVibeCraft({
      binary,
      root,
      port,
      levelName: 'world',
      env: {
        VIBECRAFT_INITIAL_CHUNK_DELAY_MS: '11000'
      }
    })
    await waitForPort(port, host, 10_000)

    const joined = await runJoinProbe(port, 'SlowChunk', {
      VIBECRAFT_EXPECT_WORLD_SEED: '424242'
    })

    assert.equal(joined.ok, true)
    assert.ok(joined.keepAliveReplies >= 1, 'expected a keepalive while initial chunks were delayed')
    assert.equal(joined.joinState.initialChunkCount, 9)
    assert.equal(joined.play.at(-1).id, 11)
    assert.ok(joined.play.findIndex(packet => packet.id === 44) < joined.play.findIndex(packet => packet.id === 45))
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
