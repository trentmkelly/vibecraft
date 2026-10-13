//! Equipment component patches for inventory synchronization. Java's
//! DataComponents uses VarInts for durability and ItemEnchantments.STREAM_CODEC
//! uses a map of registry holder IDs to VarInt levels (no tooltip flag).
use super::{write_var_i32, ItemComponent, ItemStack, RawDataComponentPatch};
use crate::item_properties::item_definition;
use crate::registry_pipeline::registry_element_id;
use std::io;

fn registry_id(registry: &str, key: &str) -> io::Result<i32> {
    registry_element_id(registry, key)
        .and_then(|id| i32::try_from(id).ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown {registry} entry {key}"),
            )
        })
}

pub(super) fn append_equipment_components(
    stack: &ItemStack,
    patch: &mut RawDataComponentPatch,
) -> io::Result<()> {
    let prototype = item_definition(stack.item_id());
    for key in [
        "minecraft:max_damage",
        "minecraft:damage",
        "minecraft:enchantments",
        "minecraft:stored_enchantments",
    ] {
        let original = prototype.as_ref().and_then(|definition| {
            definition
                .components
                .iter()
                .find(|component| component.key() == key)
        });
        let current = stack.component(key);
        if current == original {
            continue;
        }
        let id = registry_id("minecraft:data_component_type", key)?;
        let Some(component) = current else {
            patch.removed.push(id);
            continue;
        };
        let mut payload = Vec::new();
        match component {
            ItemComponent::MaxDamage(value) | ItemComponent::Damage(value) => {
                let value = i32::try_from(*value).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Durability exceeds Java integer range",
                    )
                })?;
                write_var_i32(&mut payload, value)?;
            }
            ItemComponent::Enchantments(entries) | ItemComponent::StoredEnchantments(entries) => {
                write_var_i32(&mut payload, entries.len() as i32)?;
                for (name, level) in entries {
                    write_var_i32(&mut payload, registry_id("minecraft:enchantment", name)?)?;
                    write_var_i32(&mut payload, *level)?;
                }
            }
            _ => unreachable!("component keys identify their variants"),
        }
        patch.added.push((id, payload));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::play::raw_item_stack_from_item_stack;
    use crate::network::varint::read_var_i32;
    use std::collections::BTreeMap;

    #[test]
    fn equipment_patch_carries_repaired_damage_and_registry_enchantments() {
        let mut stack = ItemStack::new("minecraft:diamond_sword", 1);
        stack.set_damage_value(37);
        stack.set_component(ItemComponent::Enchantments(BTreeMap::from([(
            "minecraft:mending".to_owned(),
            1,
        )])));
        let raw = raw_item_stack_from_item_stack(&stack).unwrap();
        let payload = |key| {
            raw.components
                .added
                .iter()
                .find(|(id, _)| *id == registry_id("minecraft:data_component_type", key).unwrap())
                .unwrap()
                .1
                .as_slice()
        };
        let mut damage = payload("minecraft:damage");
        assert_eq!(read_var_i32(&mut damage).unwrap(), 37);
        assert!(damage.is_empty());
        let mut enchantments = payload("minecraft:enchantments");
        assert_eq!(read_var_i32(&mut enchantments).unwrap(), 1);
        assert_eq!(
            read_var_i32(&mut enchantments).unwrap(),
            registry_id("minecraft:enchantment", "minecraft:mending").unwrap()
        );
        assert_eq!(read_var_i32(&mut enchantments).unwrap(), 1);
        assert!(enchantments.is_empty());
        stack.set_damage_value(0);
        assert!(!raw_item_stack_from_item_stack(&stack)
            .unwrap()
            .components
            .added
            .iter()
            .any(|(id, _)| *id
                == registry_id("minecraft:data_component_type", "minecraft:damage").unwrap()));
    }

    #[test]
    fn equipment_patch_removes_prototype_components_and_rejects_unknown_enchantments() {
        let mut stack = ItemStack::new("minecraft:diamond_sword", 1);
        stack.remove_component("minecraft:max_damage");
        let raw = raw_item_stack_from_item_stack(&stack).unwrap();
        assert!(raw.components.removed.contains(
            &registry_id("minecraft:data_component_type", "minecraft:max_damage").unwrap()
        ));
        stack.set_component(ItemComponent::Enchantments(BTreeMap::from([(
            "test:missing".to_owned(),
            1,
        )])));
        assert!(raw_item_stack_from_item_stack(&stack).is_err());
    }
}
