import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

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
