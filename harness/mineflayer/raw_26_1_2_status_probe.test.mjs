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
const vibeCraftBin = path.resolve(new URL('../..', here).pathname, 'target', 'debug', 'vibecraft')
const host = '127.0.0.1'

test('raw 26.1.2 status probe validates MOTD, version, player counts, and ping echo', async () => {
  // Java oracle:
  // - ServerStatusPacketListenerImpl.handleStatusRequest sends one
  //   ClientboundStatusResponsePacket per status connection.
  // - ServerStatusPacketListenerImpl.handlePingRequest echoes
  //   ServerboundPingRequestPacket#getTime in ClientboundPongResponsePacket,
  //   then disconnects.
  const port = await reservePort()
  const fixture = await startFixtureServer({ port })
  try {
    const result = await runProbe({ port })
    assert.equal(result.ok, true)
    assert.equal(result.status.version.name, '26.1.2')
    assert.equal(result.status.version.protocol, 775)
    assert.equal(result.status.description.text, 'A Minecraft Server')
    assert.equal(result.status.players.online, 0)
    assert.equal(result.status.players.max, 20)
    assert.equal(result.pong, result.expectedPong)
  } finally {
    await fixture.cleanup()
  }
})

test('raw 26.1.2 status probe covers hidden player counts and disabled status replies', async () => {
  // Java oracle:
  // - MinecraftServer.buildPlayerStatus keeps the online count but returns an
  //   empty sample when hidesOnlinePlayers() is true.
  // - ServerHandshakePacketListenerImpl.handleIntention disconnects status
  //   requests when repliesToStatus() is false.
  const hiddenPort = await reservePort()
  const hidden = await startFixtureServer({
    port: hiddenPort,
    properties: {
      'hide-online-players': 'true',
      'max-players': '37'
    }
  })
  try {
    const hiddenResult = await runProbe({ port: hiddenPort })
    assert.equal(hiddenResult.status.players.max, 37)
    assert.equal(hiddenResult.status.players.online, 0)
    assert.deepEqual(hiddenResult.status.players.sample, [])
  } finally {
    await hidden.cleanup()
  }

  const disabledPort = await reservePort()
  const disabled = await startFixtureServer({
    port: disabledPort,
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
            VIBECRAFT_HOST: host,
            VIBECRAFT_PORT: String(disabledPort),
            VIBECRAFT_TIMEOUT_MS: '500'
          },
          timeout: 3000,
          maxBuffer: 1024 * 1024
        }
      ),
      error => {
        const stderr = error.stderr ?? ''
        return stderr.includes('timed out waiting for status packet') ||
          stderr.includes('socket timed out') ||
          stderr.includes('read ECONNRESET')
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
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port)
      },
      timeout: 8000,
      maxBuffer: 1024 * 1024
    }
  )
  return JSON.parse(stdout)
}

async function startFixtureServer (options) {
  const root = await createTempWorld('vibecraft-status-probe-')
  await writeOfflineServerFiles(root, {
    port: options.port,
    properties: {
      motd: 'A Minecraft Server',
      ...(options.properties ?? {})
    }
  })
  const server = startVibeCraft({
    root,
    binary: vibeCraftBin,
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

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.listen(0, host, resolve).once('error', reject)
  })
  const port = server.address().port
  await new Promise(resolve => server.close(resolve))
  return port
}
