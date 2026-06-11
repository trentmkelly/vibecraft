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

test('raw 26.1.2 status socket cleanup does not corrupt immediate offline login', { timeout: 45_000 }, async () => {
  // Java oracle: ServerHandshakePacketListenerImpl installs an independent
  // ServerStatusPacketListenerImpl for STATUS intent and ServerLoginPacketListenerImpl
  // for LOGIN intent; the status listener sends pong then disconnects only that
  // status Connection.
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-status-login-')
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

    const status = await runJsonProbe('raw_26_1_2_status_probe.mjs', { port })
    assert.equal(status.ok, true)
    assert.equal(status.status.version.protocol, 775)
    assert.equal(status.pong, status.expectedPong)

    const login = await runJsonProbe('raw_26_1_2_join_probe.mjs', {
      port,
      username: 'StatusLogin'
    })
    assert.equal(login.ok, true)
    assert.ok(login.config.some(packet => packet.id === 3), 'expected finish configuration packet')
    assert.ok(login.play.some(packet => packet.id === 49), 'expected play login packet')
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runJsonProbe (script, options = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    [script],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(options.port),
        ...(options.username ? { VIBECRAFT_USERNAME: options.username } : {})
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
    server.listen(0, host, resolve).once('error', reject)
  })
  const port = server.address().port
  await new Promise(resolve => server.close(resolve))
  return port
}
