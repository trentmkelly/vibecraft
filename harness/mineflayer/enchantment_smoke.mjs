// Enchantment smoke test: join offline mode, receive an enchanted item (sword, pick, boots)
// via /give, verify bot/client does not hit missing registry, missing tag, tooltip,
// or component decode failures.

export function createEnchantmentSmokePlan(options = {}) {
  return {
    name: 'mineflayer-enchantment-smoke',
    mode: 'offline',
    auth: 'offline',
    username: options.username ?? 'EnchantBot',
    serverProperties: {
      'online-mode': 'false',
      gamemode: 'creative',
      difficulty: 'peaceful',
      'spawn-protection': '0'
    },
    steps: [
      'give-enchanted-sword-no-registry-error',
      'give-enchanted-pickaxe-no-registry-error',
      'give-enchanted-boots-no-registry-error',
      'tooltip-decode-no-component-error',
      'enchant-tag-lookup-no-missing-tag'
    ]
  }
}

export function summarizeEnchantmentSmoke(session, plan) {
  const actions = (session.timeline ?? [])
    .filter(e => e.name === 'enchant')
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
    'give-enchanted-sword-no-registry-error': 'enchant.give.sword',
    'give-enchanted-pickaxe-no-registry-error': 'enchant.give.pickaxe',
    'give-enchanted-boots-no-registry-error': 'enchant.give.boots',
    'tooltip-decode-no-component-error': 'enchant.tooltip.decode_ok',
    'enchant-tag-lookup-no-missing-tag': 'enchant.tag.lookup_ok'
  }
  return map[step] ?? step
}

export function recordEnchantEvent(session, action, details = {}) {
  session.timeline.push({ name: 'enchant', at: Date.now(), summary: [action, details] })
  return { action, ...details }
}

// Enchanted items used in smoke tests
export const ENCHANTED_SMOKE_ITEMS = [
  {
    step: 'give-enchanted-sword-no-registry-error',
    action: 'enchant.give.sword',
    itemId: 'minecraft:diamond_sword',
    enchantments: [
      { id: 'minecraft:sharpness', level: 5 },
      { id: 'minecraft:looting', level: 3 },
      { id: 'minecraft:unbreaking', level: 3 }
    ]
  },
  {
    step: 'give-enchanted-pickaxe-no-registry-error',
    action: 'enchant.give.pickaxe',
    itemId: 'minecraft:diamond_pickaxe',
    enchantments: [
      { id: 'minecraft:efficiency', level: 5 },
      { id: 'minecraft:fortune', level: 3 },
      { id: 'minecraft:mending', level: 1 }
    ]
  },
  {
    step: 'give-enchanted-boots-no-registry-error',
    action: 'enchant.give.boots',
    itemId: 'minecraft:diamond_boots',
    enchantments: [
      { id: 'minecraft:feather_falling', level: 4 },
      { id: 'minecraft:protection', level: 4 },
      { id: 'minecraft:depth_strider', level: 3 }
    ]
  }
]

export function buildGiveCommand(item) {
  const enchStr = item.enchantments
    .map(e => `{id:"${e.id}",lvl:${e.level}}`)
    .join(',')
  return `/give @s ${item.itemId}{Enchantments:[${enchStr}]}`
}

if (import.meta.url === `file://${process.argv[1]}`) {
  console.log('enchantment_smoke defined — run via the test harness')
}
