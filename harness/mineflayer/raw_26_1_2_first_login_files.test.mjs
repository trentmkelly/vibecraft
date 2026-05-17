import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { access, readFile, readdir, rm } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'
import { gunzipSync } from 'node:zlib'

import {
  createTempWorld,
  offlineUuid,
  startRustCraft,
  stopServer,
  waitForPort,
  writeOfflineServerFiles
} from './runner.mjs'

const execFileAsync = promisify(execFile)
const here = new URL('.', import.meta.url)
const repoRoot = path.resolve(here.pathname, '..', '..')
const binary = path.join(repoRoot, 'target', 'debug', 'rustcraft')
const host = '127.0.0.1'

test('raw 26.1.2 first login creates only vanilla-compatible profile files at save points', { timeout: 90_000 }, async () => {
  const port = await reservePort()
  const root = await createTempWorld('rustcraft-first-login-files-')
  const username = 'FirstFiles'
  const uuid = offlineUuid(username)
  let server

  try {
    await writeOfflineServerFiles(root, { port, levelName: 'world' })
    server = await start(root, port)

    const aborted = await runJoinProbe(port, username, {
      RUSTCRAFT_RAW_PROBE_ABORT_AFTER: 'login_success'
    })
    assert.equal(aborted.ok, true)
    assert.equal(await exists(playerDataFile(root, uuid)), false, 'login success alone should not create playerdata')

    await stopServer(server.child)
    server = await start(root, port)

    const joined = await runJoinProbe(port, username)
    assert.equal(joined.ok, true)
    await waitForFile(playerDataFile(root, uuid))

    const playerdata = gunzipSync(await readFile(playerDataFile(root, uuid)))
    assert.equal(playerdata.includes(Buffer.from('recipeBook')), true, 'recipeBook should be stored inside playerdata')
    assert.equal(await exists(path.join(root, 'world', 'advancements', `${uuid}.json`)), false)
    assert.equal(await exists(path.join(root, 'world', 'stats', `${uuid}.json`)), false)
    assert.deepEqual(await profileSidecars(root, uuid), [], 'fresh login should not create recipe or last-known-position sidecar files')
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
})

function playerDataFile (root, uuid) {
  return path.join(root, 'world', 'playerdata', `${uuid}.dat`)
}

async function profileSidecars (root, uuid) {
  const world = path.join(root, 'world')
  const entries = await collectFiles(world)
  return entries
    .filter(file => file.includes(uuid))
    .filter(file => !file.endsWith(path.join('playerdata', `${uuid}.dat`)))
    .sort()
}

async function collectFiles (dir, prefix = '') {
  let entries
  try {
    entries = await readdir(dir, { withFileTypes: true })
  } catch {
    return []
  }
  const files = []
  for (const entry of entries) {
    const relative = path.join(prefix, entry.name)
    const absolute = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...await collectFiles(absolute, relative))
    } else {
      files.push(relative)
    }
  }
  return files
}

async function start (root, port) {
  const server = startRustCraft({ binary, root, port, levelName: 'world' })
  await waitForPort(port, host, 10_000)
  return server
}

async function runJoinProbe (port, username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        RUSTCRAFT_HOST: host,
        RUSTCRAFT_PORT: String(port),
        RUSTCRAFT_USERNAME: username,
        ...env
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  return JSON.parse(stdout)
}

async function reservePort () {
  const server = createServer()
  await new Promise((resolve, reject) => {
    server.listen(0, host, resolve)
    server.once('error', reject)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}

async function waitForFile (file) {
  const deadline = Date.now() + 5_000
  while (Date.now() < deadline) {
    if (await exists(file)) return
    await delay(50)
  }
  throw new Error(`timed out waiting for ${file}`)
}

async function exists (file) {
  try {
    await access(file)
    return true
  } catch {
    return false
  }
}

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}
