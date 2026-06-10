import { mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '..', '..')

export function createVanillaClientXephyrPlan (options = {}) {
  const display = options.display ?? process.env.VIBECRAFT_VANILLA_CLIENT_DISPLAY ?? ':2'
  const screen = options.screen ?? process.env.VIBECRAFT_VANILLA_CLIENT_SCREEN ?? '1280x720'
  const gameDir = options.gameDir ?? process.env.VIBECRAFT_VANILLA_CLIENT_GAME_DIR ?? '/tmp/codex-mc-xephyr'
  const artifactsDir = options.artifactsDir ?? process.env.VIBECRAFT_VANILLA_CLIENT_ARTIFACTS ?? path.join(repoRoot, 'artifacts', 'vanilla-client')
  return {
    name: 'vanilla-client-xephyr-oracle',
    display,
    screen,
    gameDir,
    artifactsDir,
    screenshotPath: path.join(artifactsDir, 'xephyr-root.png'),
    launchLogPath: path.join(artifactsDir, 'launch.json'),
    clientLogPath: path.join(gameDir, 'logs', 'latest.log'),
    crashReportsDir: path.join(gameDir, 'crash-reports'),
    environment: {
      DISPLAY: display,
      GLFW_PLATFORM: 'x11',
      XDG_SESSION_TYPE: 'x11'
    },
    requirements: [
      'Xephyr nested X11 display',
      'ImageMagick import for root screenshots',
      'a currently running vanilla java client process to clone argv from',
      'isolated --gameDir for artifacts and logs',
      'no full argv logging because vanilla launch args contain auth material'
    ]
  }
}

export function sanitizeMinecraftClientArgv (argv, options = {}) {
  const gameDir = options.gameDir ?? '/tmp/codex-mc-xephyr'
  const omitted = new Set(['--quickPlayPath'])
  const sanitized = []
  let replacedGameDir = false

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]
    if (arg === '--gameDir') {
      sanitized.push(arg, gameDir)
      replacedGameDir = true
      index += 1
      continue
    }
    if (omitted.has(arg)) {
      index += 1
      continue
    }
    sanitized.push(arg)
  }

  if (!replacedGameDir) sanitized.push('--gameDir', gameDir)
  return sanitized
}

export function redactMinecraftClientArgv (argv) {
  const secretArgs = new Set(['--accessToken', '--clientId', '--xuid', '--userProperties'])
  const redacted = []
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]
    redacted.push(arg)
    if (secretArgs.has(arg) && index + 1 < argv.length) {
      redacted.push('<redacted>')
      index += 1
    }
  }
  return redacted
}

export async function findRunningVanillaClientArgv (procRoot = '/proc') {
  const entries = await readdir(procRoot, { withFileTypes: true })
  for (const entry of entries) {
    if (!entry.isDirectory() || !/^\d+$/.test(entry.name)) continue
    const cmdlinePath = path.join(procRoot, entry.name, 'cmdline')
    try {
      const cmdline = await readFile(cmdlinePath)
      const argv = cmdline.toString('utf8').split('\0').filter(Boolean)
      if (argv.some(arg => arg === 'net.minecraft.client.main.Main')) {
        return { pid: Number(entry.name), argv }
      }
    } catch {
      // Processes can disappear while scanning /proc.
    }
  }
  throw new Error('no running vanilla Minecraft client java process found')
}

export async function runVanillaClientXephyr (options = {}) {
  const plan = createVanillaClientXephyrPlan(options)
  await mkdir(plan.artifactsDir, { recursive: true })
  await mkdir(plan.gameDir, { recursive: true })
  await ensureXephyr(plan, options)

  const source = options.argv
    ? { pid: null, argv: options.argv }
    : await findRunningVanillaClientArgv(options.procRoot)
  const argv = sanitizeMinecraftClientArgv(source.argv, { gameDir: plan.gameDir })
  const child = spawn(argv[0], argv.slice(1), {
    detached: true,
    stdio: 'ignore',
    env: {
      ...process.env,
      ...plan.environment
    }
  })
  child.unref()

  await delay(Number(options.launchDelayMs ?? process.env.VIBECRAFT_VANILLA_CLIENT_LAUNCH_DELAY_MS ?? 5000))
  await captureXephyrScreenshot(plan, options)
  await writeFile(plan.launchLogPath, JSON.stringify({
    plan: {
      ...plan,
      environment: plan.environment
    },
    sourcePid: source.pid,
    launchedPid: child.pid,
    argv: redactMinecraftClientArgv(argv)
  }, null, 2))
  return {
    plan,
    sourcePid: source.pid,
    launchedPid: child.pid
  }
}

async function ensureXephyr (plan, options) {
  if (options.skipXephyr === true || process.env.VIBECRAFT_VANILLA_CLIENT_SKIP_XEPHYR === '1') return
  const probe = await runCommand('xdpyinfo', ['-display', plan.display], { allowFailure: true })
  if (probe.code === 0) return
  const child = spawn('Xephyr', [plan.display, '-screen', plan.screen, '-resizeable', '-ac'], {
    detached: true,
    stdio: 'ignore'
  })
  child.unref()
  await delay(Number(options.xephyrDelayMs ?? 1500))
}

async function captureXephyrScreenshot (plan, options) {
  if (options.skipScreenshot === true || process.env.VIBECRAFT_VANILLA_CLIENT_SKIP_SCREENSHOT === '1') return
  await runCommand('import', ['-window', 'root', plan.screenshotPath], {
    env: {
      ...process.env,
      DISPLAY: plan.display
    }
  })
}

function runCommand (command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { env: options.env ?? process.env })
    let stderr = ''
    child.stderr?.on('data', chunk => {
      stderr += chunk.toString('utf8')
    })
    child.on('error', error => {
      if (options.allowFailure) resolve({ code: 127, stderr: String(error) })
      else reject(error)
    })
    child.on('close', code => {
      if (code === 0 || options.allowFailure) resolve({ code, stderr })
      else reject(new Error(`${command} ${args.join(' ')} failed with exit ${code}: ${stderr}`))
    })
  })
}

async function collectPathIfPresent (filePath) {
  try {
    await stat(filePath)
    return filePath
  } catch {
    return null
  }
}

export async function vanillaClientArtifactInventory (options = {}) {
  const plan = createVanillaClientXephyrPlan(options)
  const crashReports = []
  try {
    for (const entry of await readdir(plan.crashReportsDir)) {
      if (entry.endsWith('.txt')) crashReports.push(path.join(plan.crashReportsDir, entry))
    }
  } catch {
    // Missing crash report directory is normal for successful launches.
  }
  return {
    screenshot: await collectPathIfPresent(plan.screenshotPath),
    launchLog: await collectPathIfPresent(plan.launchLogPath),
    latestLog: await collectPathIfPresent(plan.clientLogPath),
    crashReports: crashReports.sort()
  }
}

function delay (ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runVanillaClientXephyr().then(async result => {
    console.log(JSON.stringify({
      ...result,
      artifacts: await vanillaClientArtifactInventory(result.plan)
    }, null, 2))
  }).catch(error => {
    console.error(error)
    process.exitCode = 1
  })
}
