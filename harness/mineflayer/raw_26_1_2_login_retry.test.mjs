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
let port = 0

test('raw 26.1.2 offline login retry recovers immediately after a forced first-attempt disconnect', { timeout: 90_000 }, async () => {
  port = await reservePort()
  const root = await createTempWorld('vibecraft-login-retry-')
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

    const phases = ['login_success', 'registry_sync', 'first_chunk']

    for (const phase of phases) {
      const username = `Rt${phase.replaceAll('_', '').slice(0, 8)}${crypto.randomUUID().replaceAll('-', '').slice(0, 5)}`
      const failed = await runJoinProbe(username, { VIBECRAFT_RAW_PROBE_ABORT_AFTER: phase })
      assert.equal(failed.ok, true)
      assert.equal(failed.aborted, true)
      assert.equal(failed.phase, phase)

      const retry = await runJoinProbe(username)
      assert.equal(retry.ok, true)
      assert.equal(retry.joinState.profile.name, username)
      assert.ok(retry.configPacketCount > 0, `${phase} retry should receive config packets`)
      assert.ok(retry.playPacketCount > 0, `${phase} retry should receive play packets`)
      assert.ok(retry.joinState.initialChunkCount > 0, `${phase} retry should receive initial chunks`)
      assert.equal(retry.joinState.lastReceivedChunk, retry.joinState.initialChunkCount - 1, `${phase} retry should receive complete initial chunk batch`)
      assert.ok(
        serverLogs(server).some(line => line.includes(`${username}[`) && line.includes('logged in with entity id')),
        `${phase} retry should emit a successful login log line`
      )
    }
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runJoinProbe (username, env = {}) {
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

function serverLogs (server) {
  return server.logs.flatMap(entry => entry.text.split(/\r?\n/).filter(Boolean))
}
