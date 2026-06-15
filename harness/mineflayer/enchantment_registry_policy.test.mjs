import assert from 'node:assert/strict'
import { execFile } from 'node:child_process'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)
const here = path.dirname(fileURLToPath(import.meta.url))
const decompiledSourceRoot = process.env.VIBECRAFT_DECOMPILED_SOURCE_ROOT

const enchantmentPath = path.join(
  decompiledSourceRoot ?? '',
  'net',
  'minecraft',
  'world',
  'item',
  'enchantment',
  'Enchantment.java'
)
const effectComponentsPath = path.join(
  decompiledSourceRoot ?? '',
  'net',
  'minecraft',
  'world',
  'item',
  'enchantment',
  'EnchantmentEffectComponents.java'
)
const itemsPath = path.join(
  decompiledSourceRoot ?? '',
  'net',
  'minecraft',
  'world',
  'item',
  'Items.java'
)

const directCodecFields = [
  'description',
  'exclusive_set',
  'effects'
]

const definitionFields = [
  'supported_items',
  'primary_items',
  'weight',
  'max_level',
  'min_cost',
  'max_cost',
  'anvil_cost',
  'slots'
]

const costFields = [
  'base',
  'per_level_above_first'
]

const effectComponentIds = [
  'damage_protection',
  'damage_immunity',
  'damage',
  'smash_damage_per_fallen_block',
  'knockback',
  'armor_effectiveness',
  'post_attack',
  'post_piercing_attack',
  'hit_block',
  'item_damage',
  'equipment_drops',
  'location_changed',
  'tick',
  'ammo_use',
  'projectile_piercing',
  'projectile_spawned',
  'projectile_spread',
  'projectile_count',
  'trident_return_acceleration',
  'fishing_time_reduction',
  'fishing_luck_bonus',
  'block_experience',
  'mob_experience',
  'repair_with_xp',
  'attributes',
  'crossbow_charge_time',
  'crossbow_charging_sounds',
  'trident_sound',
  'prevent_equipment_drop',
  'prevent_armor_change',
  'trident_spin_attack_strength'
]

test('decompiled enchantment direct codec field audit covers the current omission policy', {
  skip: !decompiledSourceRoot ? 'optional Java source root unavailable' : false
}, async () => {
  const [enchantmentSource, effectSource] = await Promise.all([
    readFile(enchantmentPath, 'utf8'),
    readFile(effectComponentsPath, 'utf8')
  ])

  for (const field of directCodecFields) {
    assert.match(enchantmentSource, new RegExp(`(?:fieldOf|optionalFieldOf)\\("${field}"`))
  }
  for (const field of definitionFields) {
    assert.match(enchantmentSource, new RegExp(`(?:fieldOf|optionalFieldOf)\\("${field}"`))
  }
  for (const field of costFields) {
    assert.match(enchantmentSource, new RegExp(`fieldOf\\("${field}"\\)`))
  }
  for (const id of effectComponentIds) {
    assert.match(effectSource, new RegExp(`register\\(\\s*"${id}"`))
  }
})

test('default item initialization does not reference concrete enchantment holders before play entry', {
  skip: !decompiledSourceRoot ? 'optional Java source root unavailable' : false
}, async () => {
  const itemsSource = await readFile(itemsPath, 'utf8')

  assert.doesNotMatch(itemsSource, /(?<!Item)Enchantments\.[A-Z0-9_]+/)
  assert.match(itemsSource, /DataComponents\.STORED_ENCHANTMENTS,\s*ItemEnchantments\.EMPTY/)
})

test('raw 26.1.2 probe enforces omitted enchantment policy while proving play entry', {
  skip: process.env.VIBECRAFT_RUN_LIVE_ENCHANTMENT_POLICY_TEST !== '1'
}, async () => {
  const { stdout } = await execFileAsync(
    process.execPath,
    ['raw_26_1_2_join_probe.mjs'],
    {
      cwd: here,
      env: {
        ...process.env,
        VIBECRAFT_USERNAME: 'EnchantPolicy'
      },
      timeout: 30_000,
      maxBuffer: 1024 * 1024
    }
  )

  const probe = JSON.parse(stdout)
  const registryNames = new Set(probe.config.filter(packet => packet.id === 7).map(packet => packet.registry))
  assert.equal(probe.ok, true)
  assert.equal(registryNames.has('minecraft:enchantment'), false)
  assert.ok(probe.play.some(packet => packet.id === 49), 'omitted enchantments must still reach play entry')
})
