import assert from 'node:assert/strict'
import crypto from 'node:crypto'
import { execFile } from 'node:child_process'
import test from 'node:test'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

test('raw 26.1.2 first-tick passive play packets do not race-disconnect', { timeout: 45_000 }, async () => {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: new URL('.', import.meta.url),
      env: {
        ...process.env,
        RUSTCRAFT_USERNAME: `First${crypto.randomUUID().replaceAll('-', '').slice(0, 11)}`,
        RUSTCRAFT_RAW_PROBE_FIRST_TICK_ACTIONS: '1',
        RUSTCRAFT_RAW_PROBE_KEEPALIVE_MS: '15000',
        RUSTCRAFT_RAW_PROBE_OUTPUT: 'summary'
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  const result = JSON.parse(stdout)
  assert.equal(result.ok, true)
  assert.ok(result.keepAliveReplies > 0, 'server should keep the first-tick connection alive')
})
