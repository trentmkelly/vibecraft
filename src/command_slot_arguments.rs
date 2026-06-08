use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotArgumentModel;

impl SlotArgumentModel {
    pub fn slot() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<i32, SlotArgumentError> {
        let name = read_until_space(reader);
        let range = SlotRangesModel::name_to_ids(&name)
            .ok_or_else(|| SlotArgumentError::UnknownSlot { name: name.clone() })?;
        if range.size() != 1 {
            return Err(SlotArgumentError::OnlySingleSlotAllowed { name });
        }
        Ok(range.slots()[0])
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(SlotRangesModel::single_slot_names(), builder);
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["container.5", "weapon"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotsArgumentModel;

impl SlotsArgumentModel {
    pub fn slots() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<SlotRangeModel, SlotArgumentError> {
        let name = read_until_space(reader);
        SlotRangesModel::name_to_ids(&name).ok_or(SlotArgumentError::UnknownSlot { name })
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(SlotRangesModel::all_names(), builder);
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["container.*", "container.5", "weapon"]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    slot_arguments: HashMap<String, i32>,
    slots_arguments: HashMap<String, SlotRangeModel>,
}

impl CommandContextModel {
    pub fn with_slot(mut self, name: impl Into<String>, value: i32) -> Self {
        self.slot_arguments.insert(name.into(), value);
        self
    }

    pub fn with_slots(mut self, name: impl Into<String>, value: SlotRangeModel) -> Self {
        self.slots_arguments.insert(name.into(), value);
        self
    }
}

pub fn get_slot(context: &CommandContextModel, name: &str) -> Option<i32> {
    context.slot_arguments.get(name).copied()
}

pub fn get_slots(context: &CommandContextModel, name: &str) -> Option<SlotRangeModel> {
    context.slots_arguments.get(name).cloned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRangeModel {
    name: String,
    slots: Vec<i32>,
}

impl SlotRangeModel {
    pub fn of(name: impl Into<String>, slots: impl Into<Vec<i32>>) -> Self {
        Self {
            name: name.into(),
            slots: slots.into(),
        }
    }

    pub fn slots(&self) -> &[i32] {
        &self.slots
    }

    pub fn size(&self) -> usize {
        self.slots.len()
    }

    pub fn serialized_name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Display for SlotRangeModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRangesModel {
    slots: Vec<SlotRangeModel>,
}

impl SlotRangesModel {
    pub const MOB_INVENTORY_SLOT_OFFSET: i32 = 300;
    pub const MOB_INVENTORY_SIZE: i32 = 8;

    pub fn java_slots() -> Vec<SlotRangeModel> {
        let mut values = Vec::new();
        Self::add_single_slot(&mut values, "contents", 0);
        Self::add_slot_range(&mut values, "container.", 0, 54);
        Self::add_slot_range(&mut values, "hotbar.", 0, 9);
        Self::add_slot_range(&mut values, "inventory.", 9, 27);
        Self::add_slot_range(&mut values, "enderchest.", 200, 27);
        Self::add_slot_range(
            &mut values,
            "mob.inventory.",
            Self::MOB_INVENTORY_SLOT_OFFSET,
            Self::MOB_INVENTORY_SIZE,
        );
        Self::add_slot_range(&mut values, "horse.", 500, 15);

        let mainhand = EquipmentSlotModel::Mainhand.get_index(98);
        let offhand = EquipmentSlotModel::Offhand.get_index(98);
        Self::add_single_slot(&mut values, "weapon", mainhand);
        Self::add_single_slot(&mut values, "weapon.mainhand", mainhand);
        Self::add_single_slot(&mut values, "weapon.offhand", offhand);
        Self::add_slots(&mut values, "weapon.*", [mainhand, offhand]);

        let head = EquipmentSlotModel::Head.get_index(100);
        let chest = EquipmentSlotModel::Chest.get_index(100);
        let legs = EquipmentSlotModel::Legs.get_index(100);
        let feet = EquipmentSlotModel::Feet.get_index(100);
        let body = EquipmentSlotModel::Body.get_index(105);
        Self::add_single_slot(&mut values, "armor.head", head);
        Self::add_single_slot(&mut values, "armor.chest", chest);
        Self::add_single_slot(&mut values, "armor.legs", legs);
        Self::add_single_slot(&mut values, "armor.feet", feet);
        Self::add_single_slot(&mut values, "armor.body", body);
        Self::add_slots(&mut values, "armor.*", [head, chest, legs, feet, body]);

        Self::add_single_slot(
            &mut values,
            "saddle",
            EquipmentSlotModel::Saddle.get_index(106),
        );
        Self::add_single_slot(&mut values, "horse.chest", 499);
        Self::add_single_slot(&mut values, "player.cursor", 499);
        Self::add_slot_range(&mut values, "player.crafting.", 500, 4);
        values
    }

    pub fn name_to_ids(name: &str) -> Option<SlotRangeModel> {
        Self::java_slots()
            .into_iter()
            .find(|range| range.serialized_name() == name)
    }

    pub fn all_names() -> Vec<String> {
        Self::java_slots()
            .into_iter()
            .map(|range| range.serialized_name().to_string())
            .collect()
    }

    pub fn single_slot_names() -> Vec<String> {
        Self::java_slots()
            .into_iter()
            .filter(|range| range.size() == 1)
            .map(|range| range.serialized_name().to_string())
            .collect()
    }

    fn add_single_slot(output: &mut Vec<SlotRangeModel>, name: &str, slot: i32) {
        output.push(SlotRangeModel::of(name, [slot]));
    }

    fn add_slot_range(output: &mut Vec<SlotRangeModel>, prefix: &str, offset: i32, size: i32) {
        let mut all_slots = Vec::new();
        for index in 0..size {
            let slot_id = offset + index;
            output.push(SlotRangeModel::of(format!("{prefix}{index}"), [slot_id]));
            all_slots.push(slot_id);
        }
        output.push(SlotRangeModel::of(format!("{prefix}*"), all_slots));
    }

    fn add_slots(
        output: &mut Vec<SlotRangeModel>,
        name: &str,
        slots: impl IntoIterator<Item = i32>,
    ) {
        output.push(SlotRangeModel::of(
            name,
            slots.into_iter().collect::<Vec<_>>(),
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlotModel {
    Mainhand,
    Offhand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

impl EquipmentSlotModel {
    pub fn index(self) -> i32 {
        match self {
            Self::Mainhand | Self::Feet | Self::Body | Self::Saddle => 0,
            Self::Offhand | Self::Legs => 1,
            Self::Chest => 2,
            Self::Head => 3,
        }
    }

    pub fn get_index(self, base: i32) -> i32 {
        base + self.index()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringReaderModel {
    input: String,
    cursor: usize,
}

impl StringReaderModel {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn skip(&mut self) {
        self.cursor += 1;
    }
}

fn read_until_space(reader: &mut StringReaderModel) -> String {
    let start = reader.cursor;
    while reader.can_read() && reader.peek() != ' ' {
        reader.skip();
    }
    reader.input[start..reader.cursor].to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotArgumentError {
    UnknownSlot { name: String },
    OnlySingleSlotAllowed { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggestion_values(suggestions: Vec<SuggestionModel>) -> Vec<String> {
        suggestions
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect()
    }

    #[test]
    fn java_slot_ranges_table_matches_source_counts_and_core_names() {
        let all = SlotRangesModel::java_slots();
        assert_eq!(all.len(), 165);
        assert_eq!(SlotRangesModel::MOB_INVENTORY_SLOT_OFFSET, 300);
        assert_eq!(SlotRangesModel::MOB_INVENTORY_SIZE, 8);

        assert_eq!(
            SlotRangesModel::name_to_ids("contents").unwrap().slots(),
            &[0]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("container.53")
                .unwrap()
                .slots(),
            &[53]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("container.*").unwrap().slots(),
            &(0..54).collect::<Vec<_>>()
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("inventory.0").unwrap().slots(),
            &[9]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("enderchest.0")
                .unwrap()
                .slots(),
            &[200]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("mob.inventory.7")
                .unwrap()
                .slots(),
            &[307]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("horse.14").unwrap().slots(),
            &[514]
        );
    }

    #[test]
    fn java_equipment_slot_offsets_drive_named_slot_ranges() {
        assert_eq!(
            SlotRangesModel::name_to_ids("weapon").unwrap().slots(),
            &[98]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("weapon.mainhand")
                .unwrap()
                .slots(),
            &[98]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("weapon.offhand")
                .unwrap()
                .slots(),
            &[99]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("weapon.*").unwrap().slots(),
            &[98, 99]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("armor.*").unwrap().slots(),
            &[103, 102, 101, 100, 105]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("saddle").unwrap().slots(),
            &[106]
        );
        assert_eq!(
            SlotRangesModel::name_to_ids("player.crafting.3")
                .unwrap()
                .slots(),
            &[503]
        );
    }

    #[test]
    fn java_slot_range_accessors_return_slots_size_name_and_string() {
        let range = SlotRangesModel::name_to_ids("armor.body").unwrap();

        assert_eq!(range.slots(), &[105]);
        assert_eq!(range.size(), 1);
        assert_eq!(range.serialized_name(), "armor.body");
        assert_eq!(range.to_string(), "armor.body");
    }

    #[test]
    fn java_slot_argument_factory_examples_context_and_parse_single_slot() {
        assert_eq!(
            SlotArgumentModel::slot().examples(),
            ["container.5", "weapon"]
        );
        let context = CommandContextModel::default().with_slot("slot", 98);
        assert_eq!(get_slot(&context, "slot"), Some(98));
        assert_eq!(get_slot(&context, "missing"), None);

        let mut reader = StringReaderModel::new("weapon trailing");
        assert_eq!(SlotArgumentModel::slot().parse(&mut reader), Ok(98));
        assert_eq!(reader.cursor(), "weapon".len());
    }

    #[test]
    fn java_slot_argument_rejects_unknown_or_multi_slot_without_cursor_reset() {
        let mut unknown = StringReaderModel::new("missing trailing");
        assert_eq!(
            SlotArgumentModel::slot().parse(&mut unknown),
            Err(SlotArgumentError::UnknownSlot {
                name: "missing".to_string()
            })
        );
        assert_eq!(unknown.cursor(), "missing".len());

        let mut multi = StringReaderModel::new("container.* trailing");
        assert_eq!(
            SlotArgumentModel::slot().parse(&mut multi),
            Err(SlotArgumentError::OnlySingleSlotAllowed {
                name: "container.*".to_string()
            })
        );
        assert_eq!(multi.cursor(), "container.*".len());
    }

    #[test]
    fn java_slots_argument_factory_examples_context_and_parse_range() {
        assert_eq!(
            SlotsArgumentModel::slots().examples(),
            ["container.*", "container.5", "weapon"]
        );
        let stored = SlotRangesModel::name_to_ids("weapon.*").unwrap();
        let context = CommandContextModel::default().with_slots("slots", stored.clone());
        assert_eq!(get_slots(&context, "slots"), Some(stored));
        assert_eq!(get_slots(&context, "missing"), None);

        let mut reader = StringReaderModel::new("weapon.* trailing");
        let parsed = SlotsArgumentModel::slots().parse(&mut reader).unwrap();
        assert_eq!(parsed.slots(), &[98, 99]);
        assert_eq!(reader.cursor(), "weapon.*".len());
    }

    #[test]
    fn java_slots_argument_rejects_unknown_without_cursor_reset() {
        let mut reader = StringReaderModel::new("missing trailing");
        assert_eq!(
            SlotsArgumentModel::slots().parse(&mut reader),
            Err(SlotArgumentError::UnknownSlot {
                name: "missing".to_string()
            })
        );
        assert_eq!(reader.cursor(), "missing".len());
    }

    #[test]
    fn java_slot_suggestions_use_only_single_slot_names() {
        let single = SlotRangesModel::single_slot_names();
        assert_eq!(single.len(), 156);
        assert!(single.contains(&"container.5".to_string()));
        assert!(single.contains(&"weapon".to_string()));
        assert!(!single.contains(&"container.*".to_string()));
        assert!(!single.contains(&"weapon.*".to_string()));

        let mut builder = SuggestionsBuilderModel::new("weapon");
        assert_eq!(
            suggestion_values(SlotArgumentModel::slot().list_suggestions(&mut builder)),
            vec![
                "weapon".to_string(),
                "weapon.mainhand".to_string(),
                "weapon.offhand".to_string(),
            ]
        );
    }

    #[test]
    fn java_slots_suggestions_use_all_slot_range_names() {
        let all = SlotRangesModel::all_names();
        assert_eq!(all.len(), 165);
        assert!(all.contains(&"container.*".to_string()));
        assert!(all.contains(&"weapon.*".to_string()));

        let mut builder = SuggestionsBuilderModel::new("weapon");
        assert_eq!(
            suggestion_values(SlotsArgumentModel::slots().list_suggestions(&mut builder)),
            vec![
                "weapon".to_string(),
                "weapon.mainhand".to_string(),
                "weapon.offhand".to_string(),
                "weapon.*".to_string(),
            ]
        );
    }
}
