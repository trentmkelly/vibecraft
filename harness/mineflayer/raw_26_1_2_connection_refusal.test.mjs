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

test('raw 26.1.2 connection refusal covers before readiness, shutdown, and closed port', { timeout: 60_000 }, async () => {
  const port = await reservePort()

  const beforeReadiness = await runJoinProbeExpectingRefusal(port, 'BeforeReady')
  assertRefused(beforeReadiness, 'before readiness')

  const root = await createTempWorld('rustcraft-refusal-')
  let server
  try {
    await writeOfflineServerFiles(root, { port, levelName: 'world' })
    server = startRustCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, '127.0.0.1', 10_000)

    server.child.stdin.write('stop\n')
    const duringShutdown = await runJoinProbeExpectingRefusal(port, 'DuringStop')
    assertRefused(duringShutdown, 'during shutdown')
    await stopServer(server.child)

    const afterPortClose = await runJoinProbeExpectingRefusal(port, 'AfterClose')
    assertRefused(afterPortClose, 'after port close')
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

async function runJoinProbeExpectingRefusal(port, username) {
  try {
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
  } catch (error) {
    if (error.stdout) return JSON.parse(error.stdout)
    if (error.stderr) return JSON.parse(error.stderr)
    throw error
  }
}

function assertRefused(result, phase) {
  assert.equal(result.ok, false, `${phase} should not complete login`)
  assert.match(
    result.error,
    /ECONNREFUSED|ECONNRESET|socket hang up|read ECONNRESET|Connection closed/i,
    `${phase} should fail with a socket close/refusal, got ${result.error}`
  )
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
