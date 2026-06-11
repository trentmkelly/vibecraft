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

test('raw 26.1.2 login cancellation leaves same username able to reconnect', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-login-cancel-')
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

    for (const phase of ['login_success', 'registry_sync', 'first_chunk']) {
      const username = `Cancel${phase.replaceAll('_', '').slice(0, 9)}`
      const cancelled = await runJoinProbe(port, {
        VIBECRAFT_USERNAME: username,
        VIBECRAFT_RAW_PROBE_ABORT_AFTER: phase
      })
      assert.equal(cancelled.ok, true)
      assert.equal(cancelled.aborted, true)
      assert.equal(cancelled.phase, phase)

      const retry = await runJoinProbe(port, { VIBECRAFT_USERNAME: username })
      assert.equal(retry.ok, true)
      assert.ok(retry.config.some(packet => packet.id === 3), `expected ${phase} retry to finish configuration`)
      assert.ok(retry.play.some(packet => packet.id === 49), `expected ${phase} retry to reach play`)
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runJoinProbe (port, env) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        ...env
      },
      timeout: 30000,
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
