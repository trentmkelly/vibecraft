import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
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
const rustCraftBin = path.resolve(new URL('../..', here).pathname, 'target', 'debug', 'rustcraft')

test('raw 26.1.2 status probe validates MOTD, version, player counts, and ping echo', async () => {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_status_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: process.env,
      timeout: 8000,
      maxBuffer: 1024 * 1024
    }
  )

  const result = JSON.parse(stdout)
  assert.equal(result.ok, true)
  assert.equal(result.status.version.name, '26.1.2')
  assert.equal(result.status.version.protocol, 775)
  assert.equal(result.status.description.text, 'A Minecraft Server')
  assert.equal(result.status.players.online, 0)
  assert.equal(result.status.players.max, 20)
  assert.equal(result.pong, result.expectedPong)
})

test('raw 26.1.2 status probe covers hidden player counts and disabled status replies', async () => {
  const hidden = await startFixtureServer({
    port: 25665,
    properties: {
      'hide-online-players': 'true',
      'max-players': '37'
    }
  })
  try {
    const hiddenResult = await runProbe({ port: 25665 })
    assert.equal(hiddenResult.status.players.max, 37)
    assert.equal(hiddenResult.status.players.online, 0)
    assert.deepEqual(hiddenResult.status.players.sample, [])
  } finally {
    await hidden.cleanup()
  }

  const disabled = await startFixtureServer({
    port: 25666,
    properties: {
      'enable-status': 'false'
    }
  })
  try {
    await assert.rejects(
      () => execFileAsync(
        process.execPath,
        ['raw_26_1_2_status_probe.mjs'],
        {
          cwd: here,
          env: {
            ...process.env,
            RUSTCRAFT_PORT: '25666',
            RUSTCRAFT_TIMEOUT_MS: '500'
          },
          timeout: 3000,
          maxBuffer: 1024 * 1024
        }
      ),
      error => {
        const stderr = error.stderr ?? ''
        return stderr.includes('timed out waiting for status packet') || stderr.includes('socket timed out')
      }
    )
  } finally {
    await disabled.cleanup()
  }
})

async function runProbe ({ port }) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_status_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_PORT: String(port)
      },
      timeout: 8000,
      maxBuffer: 1024 * 1024
    }
  )
  return JSON.parse(stdout)
}

async function startFixtureServer (options) {
  const root = await createTempWorld('rustcraft-status-probe-')
  await writeOfflineServerFiles(root, {
    port: options.port,
    properties: options.properties
  })
  const server = startRustCraft({
    root,
    binary: rustCraftBin,
    port: options.port
  })
  await waitForPort(options.port)

  return {
    root,
    server,
    cleanup: async () => {
      await stopServer(server.child)
      await rm(root, { recursive: true, force: true })
    }
  }
}
