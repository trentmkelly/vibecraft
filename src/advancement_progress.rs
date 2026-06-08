use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::advancement_criteria::AdvancementRequirementsModel;
use crate::advancement_system::CriterionProgressModel;
use crate::network::play::{AdvancementProgressData, CriterionProgressData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaAdvancementProgressModel {
    criteria: BTreeMap<String, CriterionProgressModel>,
    requirements: AdvancementRequirementsModel,
}

impl JavaAdvancementProgressModel {
    pub fn new() -> Self {
        Self {
            criteria: BTreeMap::new(),
            requirements: AdvancementRequirementsModel::empty(),
        }
    }

    pub fn from_criteria(criteria: BTreeMap<String, CriterionProgressModel>) -> Self {
        Self {
            criteria,
            requirements: AdvancementRequirementsModel::empty(),
        }
    }

    pub fn update(&mut self, requirements: AdvancementRequirementsModel) {
        let names = requirements.names();
        self.criteria.retain(|name, _| names.contains(name));
        for name in names {
            self.criteria
                .entry(name)
                .or_insert_with(CriterionProgressModel::new);
        }
        self.requirements = requirements;
    }

    pub fn is_done(&self) -> bool {
        self.requirements
            .test(|criterion| self.is_criterion_done(criterion))
    }

    pub fn has_progress(&self) -> bool {
        self.criteria.values().any(CriterionProgressModel::is_done)
    }

    pub fn grant_progress_at_epoch_millis(
        &mut self,
        name: &str,
        obtained_epoch_millis: i64,
    ) -> bool {
        match self.criteria.get_mut(name) {
            Some(progress) if !progress.is_done() => {
                progress.grant_at_epoch_millis(obtained_epoch_millis);
                true
            }
            _ => false,
        }
    }

    pub fn revoke_progress(&mut self, name: &str) -> bool {
        match self.criteria.get_mut(name) {
            Some(progress) if progress.is_done() => {
                progress.revoke();
                true
            }
            _ => false,
        }
    }

    pub fn get_criterion(&self, id: &str) -> Option<&CriterionProgressModel> {
        self.criteria.get(id)
    }

    pub fn get_percent(&self) -> f32 {
        if self.criteria.is_empty() {
            return 0.0;
        }

        let total = self.requirements.size() as f32;
        let complete = self.count_completed_requirements() as f32;
        complete / total
    }

    pub fn get_progress_text(&self) -> Option<String> {
        if self.criteria.is_empty() {
            return None;
        }

        let total = self.requirements.size();
        if total <= 1 {
            return None;
        }

        Some(format!(
            "advancements.progress {} {}",
            self.count_completed_requirements(),
            total
        ))
    }

    pub fn get_remaining_criteria(&self) -> Vec<String> {
        self.criteria
            .iter()
            .filter_map(|(name, progress)| (!progress.is_done()).then_some(name.clone()))
            .collect()
    }

    pub fn get_completed_criteria(&self) -> Vec<String> {
        self.criteria
            .iter()
            .filter_map(|(name, progress)| progress.is_done().then_some(name.clone()))
            .collect()
    }

    pub fn get_first_progress_date_epoch_millis(&self) -> Option<i64> {
        self.criteria
            .values()
            .filter_map(CriterionProgressModel::get_obtained_epoch_millis)
            .min()
    }

    pub fn java_compare_to(&self, other: &Self) -> Ordering {
        match (
            self.get_first_progress_date_epoch_millis(),
            other.get_first_progress_date_epoch_millis(),
        ) {
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
            (Some(ours), Some(theirs)) => ours.cmp(&theirs),
        }
    }

    pub fn to_network_data(&self) -> AdvancementProgressData {
        AdvancementProgressData {
            criteria: self
                .criteria
                .iter()
                .map(|(name, progress)| (name.clone(), progress.to_network_data()))
                .collect(),
        }
    }

    pub fn from_network_data(data: AdvancementProgressData) -> Self {
        Self::from_criteria(
            data.criteria
                .into_iter()
                .map(|(name, progress)| (name, CriterionProgressModel::from_network_data(progress)))
                .collect(),
        )
    }

    pub fn criteria_names(&self) -> BTreeSet<String> {
        self.criteria.keys().cloned().collect()
    }

    pub fn java_to_string(&self) -> String {
        let criteria = self
            .criteria
            .iter()
            .map(|(name, progress)| format!("{name}={progress}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "AdvancementProgress{{criteria={{{criteria}}}, requirements={}}}",
            self.requirements.java_to_string()
        )
    }

    fn is_criterion_done(&self, criterion: &str) -> bool {
        self.get_criterion(criterion)
            .is_some_and(CriterionProgressModel::is_done)
    }

    fn count_completed_requirements(&self) -> usize {
        self.requirements
            .count(|criterion| self.is_criterion_done(criterion))
    }
}

impl Default for JavaAdvancementProgressModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirements(groups: &[&[&str]]) -> AdvancementRequirementsModel {
        AdvancementRequirementsModel::new(
            groups
                .iter()
                .map(|group| group.iter().map(|name| name.to_string()).collect())
                .collect(),
        )
    }

    #[test]
    fn advancement_progress_update_matches_java_trim_insert_and_done_contract() {
        let mut progress = JavaAdvancementProgressModel::from_criteria(BTreeMap::from([
            (
                "stale".to_string(),
                CriterionProgressModel::from_obtained_epoch_millis(10),
            ),
            ("root".to_string(), CriterionProgressModel::new()),
        ]));

        progress.update(requirements(&[&["root"], &["mine_stone", "smelt_iron"]]));

        assert_eq!(
            progress.criteria_names(),
            BTreeSet::from([
                "root".to_string(),
                "mine_stone".to_string(),
                "smelt_iron".to_string(),
            ])
        );
        assert!(!progress.is_done());
        assert!(!progress.has_progress());
        assert!(progress.grant_progress_at_epoch_millis("root", 1_700_000_000_000));
        assert!(!progress.grant_progress_at_epoch_millis("root", 1_700_000_000_001));
        assert!(progress.has_progress());
        assert!(!progress.is_done());
        assert!(progress.grant_progress_at_epoch_millis("smelt_iron", 1_700_000_000_500));
        assert!(progress.is_done());
        assert!(progress.revoke_progress("root"));
        assert!(!progress.revoke_progress("root"));
        assert!(!progress.is_done());
    }

    #[test]
    fn advancement_progress_percent_text_remaining_and_completed_match_java() {
        let mut progress = JavaAdvancementProgressModel::new();
        progress.update(requirements(&[&["a"], &["b", "c"], &["d"]]));
        progress.grant_progress_at_epoch_millis("a", 100);
        progress.grant_progress_at_epoch_millis("c", 300);

        assert_eq!(progress.get_percent(), 2.0 / 3.0);
        assert_eq!(
            progress.get_progress_text(),
            Some("advancements.progress 2 3".to_string())
        );
        assert_eq!(
            progress.get_completed_criteria(),
            vec!["a".to_string(), "c".to_string()]
        );
        assert_eq!(
            progress.get_remaining_criteria(),
            vec!["b".to_string(), "d".to_string()]
        );

        let empty = JavaAdvancementProgressModel::new();
        assert_eq!(empty.get_percent(), 0.0);
        assert_eq!(empty.get_progress_text(), None);

        let from_network_without_requirements =
            JavaAdvancementProgressModel::from_network_data(AdvancementProgressData {
                criteria: vec![(
                    "a".to_string(),
                    CriterionProgressData {
                        obtained_epoch_millis: Some(100),
                    },
                )],
            });
        assert!(from_network_without_requirements.get_percent().is_nan());
        assert_eq!(from_network_without_requirements.get_progress_text(), None);
    }

    #[test]
    fn advancement_progress_network_first_date_compare_and_string_match_java() {
        let mut older = JavaAdvancementProgressModel::new();
        older.update(requirements(&[&["a"], &["b"]]));
        older.grant_progress_at_epoch_millis("b", 200);
        older.grant_progress_at_epoch_millis("a", 100);

        let mut newer = JavaAdvancementProgressModel::new();
        newer.update(requirements(&[&["a"]]));
        newer.grant_progress_at_epoch_millis("a", 300);

        let none = JavaAdvancementProgressModel::new();
        assert_eq!(older.get_first_progress_date_epoch_millis(), Some(100));
        assert_eq!(older.java_compare_to(&newer), Ordering::Less);
        assert_eq!(none.java_compare_to(&older), Ordering::Greater);
        assert_eq!(
            none.java_compare_to(&JavaAdvancementProgressModel::new()),
            Ordering::Equal
        );

        let roundtrip = JavaAdvancementProgressModel::from_network_data(older.to_network_data());
        assert_eq!(
            roundtrip
                .get_criterion("a")
                .unwrap()
                .get_obtained_epoch_millis(),
            Some(100)
        );
        assert_eq!(
            older.java_to_string(),
            "AdvancementProgress{criteria={a=CriterionProgress{obtained=100}, b=CriterionProgress{obtained=200}}, requirements=[[a], [b]]}"
        );
    }
}
