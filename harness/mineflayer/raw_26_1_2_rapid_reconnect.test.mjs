import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
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

test('raw 26.1.2 rapid reconnects do not leak stale session state', { timeout: 75_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-rapid-reconnect-')
  const username = 'RapidReturn'
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

    for (let attempt = 0; attempt < 3; attempt++) {
      const joined = await runJoinProbe(port, username, {
        RUSTCRAFT_RAW_PROBE_KEEPALIVE_MS: '15000'
      })

      assert.equal(joined.ok, true, `attempt ${attempt} should join`)
      assert.equal(joined.joinState.profile.name, username)
      assert.equal(joined.joinState.profile.uuid, offlineUuid(username))
      assert.equal(joined.joinState.dimension, 'minecraft:overworld')
      assert.equal(joined.joinState.lastReceivedChunk, 8)
      assert.ok(joined.config.some(packet => packet.id === 3), `attempt ${attempt} should finish configuration`)
      assert.ok(joined.play.some(packet => packet.id === 49), `attempt ${attempt} should receive play login`)
      assert.ok(joined.play.some(packet => packet.id === 70), `attempt ${attempt} should receive tab-list profile`)
      assert.ok(joined.keepAliveReplies > 0, `attempt ${attempt} should remain alive through keepalive`)
    }
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
