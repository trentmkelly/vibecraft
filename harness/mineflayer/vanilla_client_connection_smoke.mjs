import { mkdir, copyFile, readdir, stat, writeFile } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  createVanillaClientXephyrPlan,
  runVanillaClientXephyr,
  vanillaClientArtifactInventory
} from './vanilla_client_xephyr.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')

export function createVanillaClientConnectionSmokePlan (options = {}) {
  const client = createVanillaClientXephyrPlan(options)
  const host = options.host ?? process.env.VIBECRAFT_VANILLA_CLIENT_HOST ?? '127.0.0.1'
  const port = Number(options.port ?? process.env.VIBECRAFT_VANILLA_CLIENT_PORT ?? 25565)
  const artifactsDir = options.artifactsDir ?? client.artifactsDir
  return {
    name: 'vanilla-client-xephyr-connection-smoke',
    client,
    server: `${host}:${port}`,
    host,
    port,
    artifactsDir,
    joinedScreenshotPath: path.join(artifactsDir, 'joined-local-server.png'),
    smokeReportPath: path.join(artifactsDir, 'connection-smoke.json'),
    collectedLatestLogPath: path.join(artifactsDir, 'latest.log'),
    collectedCrashReportsDir: path.join(artifactsDir, 'crash-reports'),
    phases: [
      'launch vanilla client in Xephyr',
      'focus nested Minecraft window',
      'open Multiplayer from main menu',
      'select the saved local server row or configured coordinate',
      'join server',
      'wait for terrain to render',
      'capture joined-world screenshot',
      'collect latest.log and crash reports'
    ],
    defaults: {
      launchDelayMs: 5000,
      menuDelayMs: 750,
      terrainWaitMs: 12000,
      multiplayerButton: { x: 640, y: 360 },
      serverRow: { x: 640, y: 230 },
      joinButton: { x: 640, y: 675 }
    }
  }
}

export function xdotoolConnectionSteps (plan, options = {}) {
  const menuDelayMs = Number(options.menuDelayMs ?? plan.defaults.menuDelayMs)
  const multiplayerButton = options.multiplayerButton ?? plan.defaults.multiplayerButton
  const serverRow = options.serverRow ?? plan.defaults.serverRow
  const joinButton = options.joinButton ?? plan.defaults.joinButton
  return [
    { tool: 'xdotool', args: ['search', '--onlyvisible', '--name', 'Minecraft', 'windowactivate'], description: 'focus Minecraft window' },
    { tool: 'sleep', ms: menuDelayMs, description: 'wait for focus' },
    { tool: 'xdotool', args: ['mousemove', String(multiplayerButton.x), String(multiplayerButton.y), 'click', '1'], description: 'open Multiplayer' },
    { tool: 'sleep', ms: menuDelayMs, description: 'wait for Multiplayer list' },
    { tool: 'xdotool', args: ['mousemove', String(serverRow.x), String(serverRow.y), 'click', '1'], description: 'select saved local server' },
    { tool: 'sleep', ms: menuDelayMs, description: 'wait for server row selection' },
    { tool: 'xdotool', args: ['mousemove', String(joinButton.x), String(joinButton.y), 'click', '1'], description: 'join selected server' }
  ]
}

export async function runVanillaClientConnectionSmoke (options = {}) {
  const plan = createVanillaClientConnectionSmokePlan(options)
  await mkdir(plan.artifactsDir, { recursive: true })
  await runVanillaClientXephyr({
    ...options,
    launchDelayMs: options.launchDelayMs ?? plan.defaults.launchDelayMs
  })

  const steps = xdotoolConnectionSteps(plan, options)
  for (const step of steps) {
    if (step.tool === 'sleep') await delay(step.ms)
    else await runDisplayCommand(step.tool, step.args, plan.client.display)
  }

  await delay(Number(options.terrainWaitMs ?? plan.defaults.terrainWaitMs))
  await runDisplayCommand('import', ['-window', 'root', plan.joinedScreenshotPath], plan.client.display)
  const artifacts = await collectConnectionSmokeArtifacts(plan)
  const report = {
    plan: reportableSmokePlan(plan),
    artifacts,
    steps: steps.map(step => ({ description: step.description, tool: step.tool, args: step.args, ms: step.ms }))
  }
  await writeFile(plan.smokeReportPath, JSON.stringify(report, null, 2))
  return report
}

export async function collectConnectionSmokeArtifacts (planOrOptions = {}) {
  const plan = planOrOptions.name === 'vanilla-client-xephyr-connection-smoke'
    ? planOrOptions
    : createVanillaClientConnectionSmokePlan(planOrOptions)
  await mkdir(plan.artifactsDir, { recursive: true })
  await mkdir(plan.collectedCrashReportsDir, { recursive: true })

  const base = await vanillaClientArtifactInventory(plan.client)
  const copiedCrashReports = []
  if (base.latestLog) {
    await copyFile(base.latestLog, plan.collectedLatestLogPath)
  }
  for (const crashReport of base.crashReports) {
    const target = path.join(plan.collectedCrashReportsDir, path.basename(crashReport))
    await copyFile(crashReport, target)
    copiedCrashReports.push(target)
  }

  return {
    launchScreenshot: base.screenshot,
    joinedScreenshot: await pathIfPresent(plan.joinedScreenshotPath),
    latestLog: await pathIfPresent(plan.collectedLatestLogPath),
    crashReports: copiedCrashReports.sort(),
    smokeReport: await pathIfPresent(plan.smokeReportPath)
  }
}

function reportableSmokePlan (plan) {
  return {
    name: plan.name,
    display: plan.client.display,
    gameDir: plan.client.gameDir,
    server: plan.server,
    artifactsDir: plan.artifactsDir,
    phases: plan.phases
  }
}

async function pathIfPresent (filePath) {
  try {
    await stat(filePath)
    return filePath
  } catch {
    return null
  }
}

async function runDisplayCommand (command, args, display) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      env: {
        ...process.env,
        DISPLAY: display
      }
    })
    let stderr = ''
    child.stderr?.on('data', chunk => {
      stderr += chunk.toString('utf8')
    })
    child.on('error', reject)
    child.on('close', code => {
      if (code === 0) resolve()
      else reject(new Error(`${command} ${args.join(' ')} failed with exit ${code}: ${stderr}`))
    })
  })
}

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runVanillaClientConnectionSmoke().then(report => {
    console.log(JSON.stringify(report, null, 2))
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
