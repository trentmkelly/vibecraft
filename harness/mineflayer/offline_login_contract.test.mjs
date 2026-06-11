import test from 'node:test'
import assert from 'node:assert/strict'
import { mkdir, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { createTempWorld, offlineUuid } from './runner.mjs'
import {
  assertOfflineLoginContract,
  formatOfflineLoginContract,
  runOfflineLoginContract
} from './offline_login_contract.mjs'

test('assertOfflineLoginContract verifies readiness, EULA, offline UUID, and play entry', async () => {
  const root = await createTempWorld('vibecraft-login-contract-')
  const session = await fakeSession(root, {
    username: 'ContractBot',
    eula: 'eula=true\n',
    properties: 'online-mode=false\n',
    actualUuid: offlineUuid('ContractBot'),
    timeline: [{ name: 'login' }, { name: 'spawn' }]
  })

  try {
    const contract = await assertOfflineLoginContract(session)
    assert.equal(contract.ok, true)
    assert.deepEqual(contract.checks.map(check => [check.name, check.ok]), [
      ['serverReadiness', true],
      ['eulaAccepted', true],
      ['offlineServerProperties', true],
      ['deterministicProfile', true],
      ['playStateEntry', true]
    ])
  } finally {
    await rm(root, { recursive: true, force: true })
  }
})

test('assertOfflineLoginContract reports actionable failures before scenario assertions', async () => {
  const root = await createTempWorld('vibecraft-login-contract-fail-')
  const session = await fakeSession(root, {
    username: 'BadBot',
    eula: 'eula=false\n',
    properties: 'online-mode=true\n',
    actualUuid: 'bad-uuid',
    timeline: [{ name: 'login' }],
    error: new Error('port never opened')
  })

  try {
    const contract = await assertOfflineLoginContract(session)
    assert.equal(contract.ok, false)
    assert.deepEqual(contract.checks.map(check => check.ok), [false, false, false, false, false])
    assert.match(formatOfflineLoginContract(contract), /FAIL serverReadiness: port never opened/)
    assert.match(formatOfflineLoginContract(contract), /FAIL playStateEntry: login/)
  } finally {
    await rm(root, { recursive: true, force: true })
  }
})

test('runOfflineLoginContract wraps the shared observed login fixture', async () => {
  const root = await createTempWorld('vibecraft-login-contract-run-')
  const session = await fakeSession(root, {
    username: 'RunnerBot',
    eula: 'eula=true\n',
    properties: 'online-mode=false\n',
    actualUuid: offlineUuid('RunnerBot'),
    timeline: [{ name: 'login' }, { name: 'spawn' }]
  })

  try {
    const result = await runOfflineLoginContract({
      runLogin: async () => session
    })
    assert.equal(result.ok, true)
    assert.match(result.report, /PASS deterministicProfile/)
  } finally {
    await rm(root, { recursive: true, force: true })
  }
})

async function fakeSession(root, options) {
  await mkdir(root, { recursive: true })
  const eula = path.join(root, 'eula.txt')
  const serverProperties = path.join(root, 'server.properties')
  await writeFile(eula, options.eula)
  await writeFile(serverProperties, options.properties)
  const expectedUuid = offlineUuid(options.username)
  return {
    root,
    paths: {
      eula,
      serverProperties,
      world: path.join(root, 'world')
    },
    endpoint: { port: 25565 },
    profile: {
      username: options.username,
      expectedUuid,
      actualUuid: options.actualUuid
    },
    uuid: options.actualUuid,
    timeline: options.timeline,
    packetTrace: [],
    error: options.error
  }
}
