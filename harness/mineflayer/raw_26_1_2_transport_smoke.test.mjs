import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 transport smoke covers accept, handshake, clean close, and reconnect', async () => {
  const first = await runJoinProbe('SmokeReplay')
  assert.equal(first.ok, true)
  assert.equal(first.login, 2)
  assert.ok(first.config.some(packet => packet.id === 3), 'expected first login to finish configuration')
  assert.ok(first.play.some(packet => packet.id === 49), 'expected first login to reach play')

  const second = await runJoinProbe('SmokeReplay')
  assert.equal(second.ok, true)
  assert.equal(second.login, 2)
  assert.ok(second.config.some(packet => packet.id === 3), 'expected reconnect to finish configuration')
  assert.ok(second.play.some(packet => packet.id === 49), 'expected reconnect to reach play')
})

async function runJoinProbe (username) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        VIBECRAFT_USERNAME: username
      },
      timeout: 30000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}
