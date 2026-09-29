//! An NBT-backed `ItemStack` for block-entity `Items` lists.
//!
//! Container block entities persist their contents as `ItemStackWithSlot`
//! compounds (`Slot`, `id`, `count`, `components`). Hopper transfers must keep
//! data components byte-for-byte (custom names, enchantments, nested container
//! contents), so this type carries every field it does not interpret in
//! [`Stack::extra`] instead of round-tripping through the component-less
//! `PotItemStack`.

use crate::storage::nbt::Tag;

/// One `ItemStackWithSlot` entry without its slot.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    /// Item registry id (`minecraft:coal`).
    pub id: String,
    /// `ItemStack.getCount()`.
    pub count: i32,
    /// Every other compound entry, chiefly `components`.
    pub extra: Vec<(String, Tag)>,
}

impl Stack {
    /// A component-less stack.
    pub fn new(id: impl Into<String>, count: i32) -> Self {
        Self {
            id: id.into(),
            count,
            extra: Vec::new(),
        }
    }

    /// Decodes one `Items` list entry into `(slot, stack)`; entries without a
    /// valid slot, id or positive count are dropped like `ItemStack.CODEC`
    /// rejecting empty stacks.
    pub fn from_tag(tag: &Tag) -> Option<(usize, Self)> {
        let Tag::Compound(entries) = tag else {
            return None;
        };
        let mut slot = None;
        let mut id = None;
        let mut count = 1;
        let mut extra = Vec::new();
        for (key, value) in entries {
            match (key.as_str(), value) {
                ("Slot", Tag::Byte(value)) => slot = usize::try_from(*value).ok(),
                ("id", Tag::String(value)) => id = Some(value.clone()),
                ("count", Tag::Int(value)) => count = *value,
                ("count", Tag::Byte(value)) => count = i32::from(*value),
                ("count", Tag::Short(value)) => count = i32::from(*value),
                ("Slot" | "id" | "count", _) => {}
                _ => extra.push((key.clone(), value.clone())),
            }
        }
        (count > 0).then_some(())?;
        Some((
            slot?,
            Self {
                id: id.filter(|id| id != "minecraft:air")?,
                count,
                extra,
            },
        ))
    }

    /// Encodes the stack as an `ItemStackWithSlot` compound.
    pub fn to_tag(&self, slot: usize) -> Tag {
        let mut entries = vec![
            ("Slot".to_string(), Tag::Byte(slot as i8)),
            ("id".to_string(), Tag::String(self.id.clone())),
            ("count".to_string(), Tag::Int(self.count)),
        ];
        entries.extend(self.extra.iter().cloned());
        Tag::Compound(entries)
    }

    /// `ItemStack.getMaxStackSize()`: the `minecraft:max_stack_size` component
    /// when the stack overrides it, otherwise the item's default.
    pub fn max_stack_size(&self) -> i32 {
        self.components()
            .and_then(|components| compound_get(components, "minecraft:max_stack_size"))
            .and_then(|value| match value {
                Tag::Int(value) => Some(*value),
                _ => None,
            })
            .unwrap_or_else(|| crate::command::item_max_stack_size(&self.id))
    }

    /// `ItemStack.isSameItemSameComponents`.
    pub fn same_item_same_components(&self, other: &Self) -> bool {
        self.id == other.id && canonical(self.components()) == canonical(other.components())
    }

    fn components(&self) -> Option<&[(String, Tag)]> {
        self.extra.iter().find_map(|(key, value)| match value {
            Tag::Compound(entries) if key == "components" => Some(entries.as_slice()),
            _ => None,
        })
    }

    /// `ItemStack.split(amount)`: removes and returns up to `amount` items.
    pub fn split(&mut self, amount: i32) -> Self {
        let taken = amount.min(self.count);
        self.count -= taken;
        Self {
            count: taken,
            ..self.clone()
        }
    }
}

fn compound_get<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a Tag> {
    entries
        .iter()
        .find_map(|(name, value)| (name == key).then_some(value))
}

/// Order-insensitive view of a components compound: `DataComponentPatch`
/// equality does not depend on map iteration order, NBT compounds do.
fn canonical(components: Option<&[(String, Tag)]>) -> Vec<(String, Tag)> {
    let mut sorted: Vec<(String, Tag)> = components
        .map(|entries| entries.iter().map(|(k, v)| (k.clone(), canonical_tag(v))).collect())
        .unwrap_or_default();
    sorted.sort_by(|left, right| left.0.cmp(&right.0));
    sorted
}

fn canonical_tag(tag: &Tag) -> Tag {
    match tag {
        Tag::Compound(entries) => Tag::Compound(canonical(Some(entries))),
        Tag::List(items) => Tag::List(items.iter().map(canonical_tag).collect()),
        other => other.clone(),
    }
}
