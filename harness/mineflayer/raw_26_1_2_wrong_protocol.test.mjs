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

test('raw 26.1.2 wrong-protocol probe keeps status usable and rejects login', { timeout: 60_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('vibecraft-wrong-protocol-')
  let server

  try {
    await writeOfflineServerFiles(root, { port, levelName: 'world' })
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)

    const status = await runProbe(port, 'raw_26_1_2_status_probe.mjs', {
      VIBECRAFT_PROTOCOL_VERSION: '1'
    })
    assert.equal(status.ok, true)
    assert.equal(status.status.version.protocol, 775)

    const login = await runProbe(port, 'raw_26_1_2_join_probe.mjs', {
      VIBECRAFT_PROTOCOL_VERSION: '1',
      VIBECRAFT_EXPECT_LOGIN_DISCONNECT: '1',
      VIBECRAFT_USERNAME: 'WrongProto'
    })
    assert.equal(login.ok, true)
    assert.equal(login.disconnected, true)
    assert.equal(login.login, 0)
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runProbe (port, script, env) {
  const { stdout } = await execFileAsync(
    process.execPath,
    [script],
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
