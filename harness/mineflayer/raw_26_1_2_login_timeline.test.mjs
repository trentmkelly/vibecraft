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

const expectedInitialPlayPrefixIds = [
  49, 70, 10, 64, 105, 103, 104, 76, 18, 96, 113, 94, 95, 72, 43, 97,
  38, 38, 38, 38, 12
]

test('raw 26.1.2 login timeline reaches play and first chunks in vanilla-shaped order', { timeout: 45_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-login-timeline-')
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

    const started = Date.now()
    const joined = await runJoinProbe(port, 'TimelineProbe')
    const durationMs = Date.now() - started

    assert.equal(joined.ok, true)
    assert.ok(durationMs < 10_000, `login timeline took ${durationMs}ms`)
    assert.equal(joined.login, 2)
    assert.ok(joined.config.some(packet => packet.id === 14), 'configuration should advertise known packs')
    assert.ok(joined.config.some(packet => packet.id === 7), 'configuration should sync registries')
    assert.equal(joined.config.at(-1).id, 3, 'configuration should finish before play packets')
    assert.deepEqual(
      joined.play.slice(0, expectedInitialPlayPrefixIds.length).map(packet => packet.id),
      expectedInitialPlayPrefixIds
    )
    assert.ok(joined.joinState.initialChunkCount > 0)
    const chunkBatchStart = expectedInitialPlayPrefixIds.length
    const chunkBatchEnd = chunkBatchStart + joined.joinState.initialChunkCount
    assert.deepEqual(
      joined.play.slice(chunkBatchStart, chunkBatchEnd).map(packet => packet.id),
      Array(joined.joinState.initialChunkCount).fill(45)
    )
    assert.equal(joined.play[chunkBatchEnd].id, 11)
    assert.equal(joined.joinState.lastReceivedChunk, joined.joinState.initialChunkCount - 1)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runJoinProbe (port, username) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_HOST: host,
        RUSTCRAFT_PORT: String(port),
        RUSTCRAFT_USERNAME: username
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
