import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { createServer } from 'node:net'
import { access, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { promisify } from 'node:util'

import {
  createTempWorld,
  offlineUuid,
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

test('raw 26.1.2 profile files use vanilla offline UUID and name semantics', { timeout: 90_000 }, async () => {
  await withServer('vibecraft-profile-file-open-', {}, async ({ port, root }) => {
    const fresh = await runJoinProbe(port, 'ProfileFresh')
    const returning = await runJoinProbe(port, 'ProfileFresh')
    const mixedCase = await runJoinProbe(port, 'ProfileFresh'.toLowerCase())

    assert.equal(fresh.ok, true)
    assert.equal(returning.ok, true)
    assert.equal(mixedCase.ok, true)
    assert.equal(returning.joinState.profile.uuid, fresh.joinState.profile.uuid)
    assert.notEqual(mixedCase.joinState.profile.uuid, fresh.joinState.profile.uuid)

    await assertProfileFiles(root, ['ProfileFresh', 'profilefresh'])
  })

  await withServer('vibecraft-profile-file-ban-', {
    files: {
      'banned-players.json': [banEntry('ProfileBanned')]
    }
  }, async ({ port }) => {
    const banned = await runJoinProbe(port, 'ProfileBanned', { VIBECRAFT_EXPECT_LOGIN_DISCONNECT: '1' })
    assert.match(banned.reason, /multiplayer\.disconnect\.banned/)
  })

  await withServer('vibecraft-profile-file-access-', {
    properties: {
      'enforce-whitelist': 'true'
    },
    files: {
      'whitelist.json': [profile('ProfileListed')],
      'ops.json': [{ ...profile('ProfileOp'), level: 4, bypassesPlayerLimit: false }]
    }
  }, async ({ port, root }) => {
    const listed = await runJoinProbe(port, 'ProfileListed')
    const op = await runJoinProbe(port, 'ProfileOp')
    const rejected = await runJoinProbe(port, 'ProfileNoList', { VIBECRAFT_EXPECT_LOGIN_DISCONNECT: '1' })

    assert.equal(listed.ok, true)
    assert.equal(op.ok, true)
    assert.match(rejected.reason, /multiplayer\.disconnect\.not_whitelisted/)
    await assertProfileFiles(root, ['ProfileListed', 'ProfileOp'])

    const ops = JSON.parse(await readFile(path.join(root, 'ops.json'), 'utf8'))
    assert.equal(ops[0].uuid, offlineUuid('ProfileOp'))
    assert.equal(ops[0].name, 'ProfileOp')
    const whitelist = JSON.parse(await readFile(path.join(root, 'whitelist.json'), 'utf8'))
    assert.equal(whitelist[0].uuid, offlineUuid('ProfileListed'))
    assert.equal(whitelist[0].name, 'ProfileListed')
  })
})

async function withServer (prefix, options, callback) {
  const port = await reservePort()
  const root = await createTempWorld(prefix)
  let server

  try {
    await writeOfflineServerFiles(root, {
      port,
      levelName: 'world',
      properties: {
        'view-distance': '4',
        'simulation-distance': '4',
        ...(options.properties ?? {})
      }
    })
    await writeJsonFiles(root, options.files ?? {})
    server = startVibeCraft({ binary, root, port, levelName: 'world' })
    await waitForPort(port, host, 10_000)
    await callback({ port, root, server })
  } finally {
    if (server) await stopServer(server.child)
    await rm(root, { recursive: true, force: true })
  }
}

async function assertProfileFiles (root, names) {
  const usercache = JSON.parse(await readFile(path.join(root, 'usercache.json'), 'utf8'))
  for (const name of names) {
    const uuid = offlineUuid(name)
    const matches = usercache.filter(entry => entry.name === name)
    assert.equal(matches.length, 1, `${name} should appear once in usercache`)
    assert.equal(matches[0].uuid, uuid)
    assert.match(matches[0].expiresOn, /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} \+0000$/)
    await access(path.join(root, 'world', 'playerdata', `${uuid}.dat`))
  }
}

async function writeJsonFiles (root, files) {
  await Promise.all(Object.entries(files).map(([name, content]) =>
    writeFile(path.join(root, name), `${JSON.stringify(content)}\n`)
  ))
}

function profile (name) {
  return { uuid: offlineUuid(name), name }
}

function banEntry (name) {
  return {
    ...profile(name),
    created: '2026-05-17 00:00:00 +0000',
    source: 'Server',
    expires: 'forever',
    reason: 'profile file matrix'
  }
}

async function runJoinProbe (port, username, env = {}) {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_HOST: host,
        VIBECRAFT_PORT: String(port),
        VIBECRAFT_USERNAME: username,
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
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  const { port } = server.address()
  await new Promise(resolve => server.close(resolve))
  return port
}
