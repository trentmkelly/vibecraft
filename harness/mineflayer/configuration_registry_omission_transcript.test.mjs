import assert from 'node:assert/strict'
import { spawn } from 'node:child_process'
import { mkdtemp, rm, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import { documentedRegistryOmissions } from './configuration_registry_closure_report.mjs'
import {
  normalizeConfigurationTranscript,
  omittedRegistriesFromTranscript,
  recordOfficialServerConfigurationTranscript,
  recordServerConfigurationTranscript
} from './configuration_transcript_oracle.mjs'

const documentedOmissions = [...documentedRegistryOmissions.keys()]
const here = new URL('.', import.meta.url)
const rustCraftBinary = new URL('../../target/debug/rustcraft', here)

test('registry transcript omission helper identifies official-only synchronized registries', () => {
  const actual = normalizeConfigurationTranscript({
    config: [
      { id: 7, registry: 'minecraft:worldgen/biome', elements: 65, elementIds: ['minecraft:plains'] },
      { id: 7, registry: 'minecraft:banner_pattern', elements: 43, elementIds: ['minecraft:base'] },
      { id: 7, registry: 'minecraft:jukebox_song', elements: 21, elementIds: ['minecraft:13'] },
      { id: 3 }
    ],
    play: []
  })
  const official = normalizeConfigurationTranscript({
    config: [
      { id: 7, registry: 'minecraft:worldgen/biome', elements: 65, elementIds: ['minecraft:plains'] },
      { id: 7, registry: 'minecraft:banner_pattern', elements: 43, elementIds: ['minecraft:base'] },
      { id: 7, registry: 'minecraft:enchantment', elements: 42, elementIds: ['minecraft:sharpness'] },
      { id: 7, registry: 'minecraft:jukebox_song', elements: 21, elementIds: ['minecraft:13'] },
      { id: 7, registry: 'minecraft:timeline', elements: 1, elementIds: ['minecraft:empty'] },
      { id: 3 }
    ],
    play: []
  })

  assert.deepEqual(omittedRegistriesFromTranscript(actual, official), [
    'minecraft:enchantment',
    'minecraft:timeline'
  ])
})

test('documented registry omissions match the live official transcript gap', {
  skip: process.env.RUSTCRAFT_RUN_OFFICIAL_TRANSCRIPT_TEST !== '1'
}, async () => {
  const rustPort = Number(process.env.RUSTCRAFT_TRANSCRIPT_PORT ?? 25567)
  const vanillaPort = Number(process.env.VANILLA_TRANSCRIPT_PORT ?? 25566)
  const rustServer = await startRustCraftServer({ port: rustPort })
  try {
    await new Promise(resolve => setTimeout(resolve, 250))
    const actual = await recordServerConfigurationTranscript({
      host: '127.0.0.1',
      port: rustPort,
      username: 'RustOmitProbe'
    })
    const official = await recordOfficialServerConfigurationTranscript({
      port: vanillaPort,
      username: 'VanillaOmitProbe'
    })

    assert.deepEqual(
      omittedRegistriesFromTranscript(actual, official),
      documentedOmissions
    )
  } finally {
    await rustServer.stop()
  }
})

async function startRustCraftServer ({ port }) {
  const cwd = await mkdtemp(path.join(os.tmpdir(), 'rustcraft-transcript-'))
  await writeFile(path.join(cwd, 'eula.txt'), 'eula=true\n')
  await writeFile(path.join(cwd, 'server.properties'), [
    'online-mode=false',
    'enforce-secure-profile=false',
    'enable-status=true',
    'network-compression-threshold=-1',
    `server-port=${port}`,
    ''
  ].join('\n'))
  const child = spawn(rustCraftBinary.pathname, ['--nogui', '--port', String(port)], {
    cwd,
    stdio: ['pipe', 'pipe', 'pipe']
  })
  let output = ''
  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      child.kill('SIGTERM')
      reject(new Error(`RustCraft did not bind within 15000ms:\n${output}`))
    }, 15000)
    const onData = chunk => {
      output += chunk.toString()
      if (output.includes(`Status listener bound to 0.0.0.0:${port}`)) {
        clearTimeout(timeout)
        resolve()
      }
    }
    child.stdout.on('data', onData)
    child.stderr.on('data', onData)
    child.once('exit', code => {
      clearTimeout(timeout)
      reject(new Error(`RustCraft exited before readiness with code ${code}:\n${output}`))
    })
  })

  return {
    stop: async () => {
      if (child.exitCode === null) child.kill('SIGTERM')
      await new Promise(resolve => {
        child.once('exit', resolve)
        setTimeout(resolve, 5000)
      })
      await rm(cwd, { recursive: true, force: true })
    }
  }
}
