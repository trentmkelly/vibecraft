import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

import {
  createTempWorld,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')

test('raw 26.1.2 port reuse starts and stops repeatedly on the same offline-mode port', { timeout: 60_000 }, async () => {
  const port = await reservePort()
  const cycles = []

  for (let cycle = 0; cycle < 3; cycle++) {
    const root = await createTempWorld(`rustcraft-port-reuse-${cycle}-`)
    const username = `PortReuse${cycle}`
    let server
    try {
      await writeOfflineServerFiles(root, { port, levelName: 'world' })
      server = startRustCraft({ binary, root, port, levelName: 'world' })
      await waitForPort(port, '127.0.0.1', 10_000)

      const login = await runJoinProbe(port, username)
      assert.equal(login.ok, true)
      assert.equal(login.login, 2)
      assert.ok(login.config.some(packet => packet.id === 3), `cycle ${cycle} should finish configuration`)
      assert.ok(login.play.some(packet => packet.id === 49), `cycle ${cycle} should reach play`)
      cycles.push({ cycle, username, ok: true })
    } finally {
      if (server) await stopServer(server.child)
      await rm(root, { recursive: true, force: true })
    }
  }

  assert.deepEqual(cycles.map(cycle => cycle.ok), [true, true, true])
})

async function runJoinProbe(port, username) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_PORT: String(port),
        RUSTCRAFT_USERNAME: username
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function reservePort() {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, '127.0.0.1', resolve)
  })
  const { port } = server.address()
  await new Promise((resolve, reject) => {
    server.close(error => error ? reject(error) : resolve())
  })
  return port
}
