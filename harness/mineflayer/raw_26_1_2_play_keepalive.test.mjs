import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 play probe survives the first keepalive interval', async () => {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_RAW_PROBE_KEEPALIVE_MS: '12000'
      },
      timeout: 18000,
      maxBuffer: 1024 * 1024
    }
  )

  const result = JSON.parse(stdout)
  assert.equal(result.ok, true)
  assert.ok(result.keepAliveReplies >= 1, 'expected at least one keepalive round trip')
  assert.ok(result.play.some(packet => packet.id === 113), 'expected clientbound keep_alive packet')
})
