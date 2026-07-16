//! Dedicated-server ownership and persistence around `PlayerList` access files.
//!
//! This is the Rust counterpart of `DedicatedPlayerList.java`. The live
//! server supplies the inherited `PlayerList` instance; this model isolates
//! the subclass's observable work: startup load/save sequencing, whitelist
//! reload, and the two access-policy overrides.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crate::player_access::NameAndId;
use crate::stored_user_list::{IpBanList, ServerOpList, UserBanList, UserWhiteList};

const USER_BANLIST_FILE: &str = "banned-players.json";
const IP_BANLIST_FILE: &str = "banned-ips.json";
const OP_LIST_FILE: &str = "ops.json";
const WHITE_LIST_FILE: &str = "whitelist.json";

#[derive(Debug)]
pub struct DedicatedPlayerListModel {
    root: PathBuf,
    whitelist_enabled: bool,
    view_distance: u32,
    simulation_distance: u32,
    bans: UserBanList,
    ip_bans: IpBanList,
    ops: ServerOpList,
    whitelist: UserWhiteList,
}

// TODO(dedicated-player-list-live-wiring): bind this access-file model to the
// live DedicatedServer/PlayerList object once that server-owned lifecycle is
// exposed as a stable Rust interface.

impl DedicatedPlayerListModel {
    pub fn new(root: impl Into<PathBuf>, whitelist_enabled: bool) -> Self {
        let root = root.into();
        let whitelist_path = root.join(WHITE_LIST_FILE);
        let whitelist_preexisted = whitelist_path.is_file();
        let mut model = Self {
            bans: UserBanList::new(root.join(USER_BANLIST_FILE), ()),
            ip_bans: IpBanList::new(root.join(IP_BANLIST_FILE), ()),
            ops: ServerOpList::new(root.join(OP_LIST_FILE), ()),
            whitelist: UserWhiteList::new(&whitelist_path, ()),
            root,
            whitelist_enabled,
            view_distance: 0,
            simulation_distance: 0,
        };

        // DedicatedPlayerList's constructor intentionally logs and continues
        // when an access file is malformed or unavailable. The list facades
        // expose the same best-effort behavior through ignored I/O results.
        let _ = model.bans.load();
        let _ = model.bans.save();
        let _ = model.ip_bans.load();
        let _ = model.ip_bans.save();
        let _ = model.ops.load();
        let _ = model.whitelist.load();
        let _ = model.ops.save();
        if !whitelist_preexisted {
            let _ = model.whitelist.save();
        }
        model
    }

    pub fn set_view_distance(&mut self, distance: u32) {
        self.view_distance = distance;
    }

    pub fn set_simulation_distance(&mut self, distance: u32) {
        self.simulation_distance = distance;
    }

    pub fn view_distance(&self) -> u32 {
        self.view_distance
    }

    pub fn simulation_distance(&self) -> u32 {
        self.simulation_distance
    }

    pub fn reload_white_list(&mut self) {
        let _ = self.whitelist.load();
    }

    pub fn is_white_listed(&mut self, profile: &NameAndId) -> bool {
        !self.whitelist_enabled || self.is_op(profile) || self.whitelist.contains(profile)
    }

    pub fn is_op(&mut self, profile: &NameAndId) -> bool {
        self.ops.contains(profile)
    }

    pub fn can_bypass_player_limit(&mut self, profile: &NameAndId) -> bool {
        self.ops
            .can_bypass_player_limit(profile, chrono::Local::now())
    }

    pub fn whitelist_enabled(&self) -> bool {
        self.whitelist_enabled
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn bans(&self) -> &UserBanList {
        &self.bans
    }

    pub fn ip_bans(&self) -> &IpBanList {
        &self.ip_bans
    }

    pub fn ops(&self) -> &ServerOpList {
        &self.ops
    }

    pub fn whitelist(&self) -> &UserWhiteList {
        &self.whitelist
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/dedicated/DedicatedPlayerList.java");

    fn root(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("vibecraft-dedicated-list-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("fixture root");
        path
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn source_matches_dedicated_player_list_overrides_and_startup_sequence() {
        for fragment in [
            "public DedicatedPlayerList(final DedicatedServer server",
            "this.setViewDistance(server.viewDistance());",
            "this.setSimulationDistance(server.simulationDistance());",
            "this.loadUserBanList();",
            "this.saveUserBanList();",
            "this.loadIpBanList();",
            "this.saveIpBanList();",
            "this.loadOps();",
            "this.loadWhiteList();",
            "this.saveOps();",
            "public void reloadWhiteList()",
            "return !this.isUsingWhitelist() || this.isOp(nameAndId) || this.getWhiteList().isWhiteListed(nameAndId);",
            "return this.getOps().canBypassPlayerLimit(nameAndId);",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    fn constructor_creates_missing_access_files_and_reload_tracks_edits() {
        let path = root("startup");
        let profile = NameAndId::create_offline("Steve");
        let mut list = DedicatedPlayerListModel::new(&path, true);
        for file in [USER_BANLIST_FILE, IP_BANLIST_FILE, OP_LIST_FILE, WHITE_LIST_FILE] {
            assert!(path.join(file).is_file(), "missing startup file {file}");
        }
        assert!(!list.is_white_listed(&profile));

        fs::write(
            path.join(WHITE_LIST_FILE),
            serde_json::to_string(&[profile.to_json_value()]).expect("whitelist JSON"),
        )
        .expect("rewrite whitelist");
        list.reload_white_list();
        assert!(list.is_white_listed(&profile));

        let _ignored = fs::remove_dir_all(path);
    }

    #[test]
    fn operator_override_and_player_limit_bypass_match_dedicated_policy() {
        let path = root("policy");
        let profile = NameAndId::create_offline("Operator");
        fs::write(
            path.join(OP_LIST_FILE),
            serde_json::json!([{
                "uuid": profile.uuid,
                "name": profile.name,
                "level": 4,
                "bypassesPlayerLimit": true
            }])
            .to_string(),
        )
        .expect("ops JSON");
        let mut list = DedicatedPlayerListModel::new(&path, true);
        assert!(list.is_op(&profile));
        assert!(list.is_white_listed(&profile));
        assert!(list.can_bypass_player_limit(&profile));

        let _ignored = fs::remove_dir_all(path);
    }
}
