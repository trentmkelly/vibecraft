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

test('raw 26.1.2 transport smoke covers accept, handshake, clean close, and reconnect', { timeout: 75_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-transport-smoke-')
  const username = 'SmokeReplay'
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

    const first = await runJoinProbe(port, username)
    assert.equal(first.ok, true)
    assert.equal(first.login, 2)
    assert.equal(first.joinState.profile.name, username)
    assert.equal(first.joinState.profile.uuid, offlineUuid(username))
    assert.ok(first.config.some(packet => packet.id === 3), 'expected first login to finish configuration')
    assert.ok(first.play.some(packet => packet.id === 49), 'expected first login to reach play')

    const second = await runJoinProbe(port, username)
    assert.equal(second.ok, true)
    assert.equal(second.login, 2)
    assert.equal(second.joinState.profile.name, username)
    assert.equal(second.joinState.profile.uuid, offlineUuid(username))
    assert.ok(second.config.some(packet => packet.id === 3), 'expected reconnect to finish configuration')
    assert.ok(second.play.some(packet => packet.id === 49), 'expected reconnect to reach play')
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
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username
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
