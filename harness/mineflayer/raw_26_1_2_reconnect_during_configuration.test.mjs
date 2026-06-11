import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import crypto from 'node:crypto'
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

test('raw 26.1.2 reconnect during configuration cleans stale profile/session state', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-reconnect-config-')
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

    for (const phase of ['registry_sync', 'known_packs']) {
      const username = `Cfg${phase.replaceAll('_', '').slice(0, 6)}${crypto.randomUUID().replaceAll('-', '').slice(0, 6)}`
      const dropped = await runJoinProbe(port, username, { VIBECRAFT_RAW_PROBE_ABORT_AFTER: phase })
      assert.equal(dropped.ok, true)
      assert.equal(dropped.aborted, true)
      assert.equal(dropped.phase, phase)
      assert.ok(dropped.configPacketCount > 0, `${phase} should abort after configuration starts`)

      const retry = await runJoinProbe(port, username)
      assert.equal(retry.ok, true, `${phase} retry should reach play`)
      assert.equal(retry.joinState.profile.name, username)
      assert.ok(retry.configPacketCount > 0, `${phase} retry should receive configuration packets`)
      assert.ok(retry.playPacketCount > 0, `${phase} retry should receive play packets`)
      assert.ok(retry.joinState.initialChunkCount > 0, `${phase} retry should receive initial chunks`)
      assert.equal(
        retry.joinState.lastReceivedChunk,
        retry.joinState.initialChunkCount - 1,
        `${phase} retry should receive a complete initial chunk batch`
      )
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
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username,
        VIBECRAFT_RAW_PROBE_OUTPUT: 'summary',
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
