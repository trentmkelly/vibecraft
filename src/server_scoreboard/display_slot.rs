//! Java `net.minecraft.world.scores.DisplaySlot`.

/// `DisplaySlot` serialized names, indexed by `DisplaySlot.id()` (`DisplaySlot.java`).
const NAMES: [&str; 19] = [
    "list",
    "sidebar",
    "below_name",
    "sidebar.team.black",
    "sidebar.team.dark_blue",
    "sidebar.team.dark_green",
    "sidebar.team.dark_aqua",
    "sidebar.team.dark_red",
    "sidebar.team.dark_purple",
    "sidebar.team.gold",
    "sidebar.team.gray",
    "sidebar.team.dark_gray",
    "sidebar.team.blue",
    "sidebar.team.green",
    "sidebar.team.aqua",
    "sidebar.team.red",
    "sidebar.team.light_purple",
    "sidebar.team.yellow",
    "sidebar.team.white",
];

/// A scoreboard display slot, identified by its wire id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplaySlot(u8);

impl DisplaySlot {
    /// `DisplaySlot.values()` in declaration (= id) order.
    pub const VALUES: [DisplaySlot; 19] = {
        let mut values = [DisplaySlot(0); 19];
        let mut index = 0;
        while index < values.len() {
            values[index] = DisplaySlot(index as u8);
            index += 1;
        }
        values
    };

    /// `DisplaySlot.id()`, the VarInt sent in `ClientboundSetDisplayObjectivePacket`.
    pub fn id(self) -> i32 {
        i32::from(self.0)
    }

    /// `DisplaySlot.getSerializedName()`.
    pub fn serialized_name(self) -> &'static str {
        NAMES[usize::from(self.0)]
    }

    /// `DisplaySlot.CODEC` lookup by serialized name.
    pub fn by_name(name: &str) -> Option<Self> {
        Self::VALUES
            .into_iter()
            .find(|slot| slot.serialized_name() == name)
    }
}
