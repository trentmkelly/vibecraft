// Offline-mode block-drop tests: break with bare hand (no drop), correct tool (drop),
// Silk Touch (block itself), Fortune 1/2/3 (bonus count), explosion (0% survival),
// doTileDrops=false gamerule (no drops).

export function createBlockDropsPlan(options = {}) {
  return {
    name: 'mineflayer-block-drops',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'BlockDropsBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'survival',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    },
    steps: [
      'break-bare-hand-no-drop',
      'break-correct-tool-drop',
      'break-silk-touch-block-drops',
      'break-fortune-1-bonus',
      'break-fortune-2-bonus',
      'break-fortune-3-bonus',
      'explosion-no-drops',
      'dotiledrops-false-no-drops'
    ]
  }
}

export function summarizeBlockDrops(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'drops')
    .map(e => e.summary?.[0])
  const result = Object.fromEntries(
    plan.steps.map(step => [step, actions.includes(stepToAction(step))])
  )
  return {
    ok: Object.values(result).every(Boolean),
    steps: result
  }
}

function stepToAction(step) {
  const map = {
    'break-bare-hand-no-drop': 'drops.bare_hand.none',
    'break-correct-tool-drop': 'drops.correct_tool.present',
    'break-silk-touch-block-drops': 'drops.silk_touch.block_itself',
    'break-fortune-1-bonus': 'drops.fortune.1',
    'break-fortune-2-bonus': 'drops.fortune.2',
    'break-fortune-3-bonus': 'drops.fortune.3',
    'explosion-no-drops': 'drops.explosion.none',
    'dotiledrops-false-no-drops': 'drops.gamerule.none'
  }
  return map[step] ?? step
}

export function recordDropsEvent(session, action, details = {}) {
  session.timeline.push({ name: 'drops', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

export function expectNoDrops(blockId, reason) {
  return { blockId, expectedDrops: [], reason }
}

export function expectDrops(blockId, expectedDrops) {
  return { blockId, expectedDrops }
}

export function expectFortuneDrops(blockId, fortuneLevel, minCount, maxCount) {
  return { blockId, fortuneLevel, minCount, maxCount }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('block_drops scenarios defined — run via the test harness')
}
