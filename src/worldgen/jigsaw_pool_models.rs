use super::*;

impl JigsawPoolElementTypeModel {
    pub const REGISTRY_ORDER: [Self; 5] = [
        Self::Single,
        Self::List,
        Self::Feature,
        Self::Empty,
        Self::LegacySingle,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Single => "minecraft:single_pool_element",
            Self::List => "minecraft:list_pool_element",
            Self::Feature => "minecraft:feature_pool_element",
            Self::Empty => "minecraft:empty_pool_element",
            Self::LegacySingle => "minecraft:legacy_single_pool_element",
        }
    }
}

impl JigsawPoolElementModel {
    pub fn empty() -> Self {
        Self {
            element_type: JigsawPoolElementTypeModel::Empty,
            projection: JigsawProjectionModel::TerrainMatching,
            location: None,
            processors: &[],
            override_liquid_settings: None,
            children: Vec::new(),
        }
    }

    pub fn single(
        location: &'static str,
        processors: &'static [&'static str],
        projection: JigsawProjectionModel,
        override_liquid_settings: Option<LiquidSettingsModel>,
    ) -> Self {
        Self {
            element_type: JigsawPoolElementTypeModel::Single,
            projection,
            location: Some(location),
            processors,
            override_liquid_settings,
            children: Vec::new(),
        }
    }

    pub fn legacy_single(
        location: &'static str,
        processors: &'static [&'static str],
        projection: JigsawProjectionModel,
        override_liquid_settings: Option<LiquidSettingsModel>,
    ) -> Self {
        Self {
            element_type: JigsawPoolElementTypeModel::LegacySingle,
            projection,
            location: Some(location),
            processors,
            override_liquid_settings,
            children: Vec::new(),
        }
    }

    pub fn feature(feature: &'static str, projection: JigsawProjectionModel) -> Self {
        Self {
            element_type: JigsawPoolElementTypeModel::Feature,
            projection,
            location: Some(feature),
            processors: &[],
            override_liquid_settings: None,
            children: Vec::new(),
        }
    }

    pub fn list(
        mut children: Vec<JigsawPoolElementModel>,
        projection: JigsawProjectionModel,
    ) -> Result<Self, String> {
        if children.is_empty() {
            return Err("Elements are empty".to_string());
        }
        for child in &mut children {
            child.set_projection(projection);
        }
        Ok(Self {
            element_type: JigsawPoolElementTypeModel::List,
            projection,
            location: None,
            processors: &[],
            override_liquid_settings: None,
            children,
        })
    }

    pub fn set_projection(&mut self, projection: JigsawProjectionModel) {
        self.projection = projection;
        if self.element_type == JigsawPoolElementTypeModel::List {
            for child in &mut self.children {
                child.set_projection(projection);
            }
        }
    }

    pub fn ground_level_delta(&self) -> i32 {
        1
    }

    pub fn empty_size(&self) -> Option<(i32, i32, i32)> {
        (self.element_type == JigsawPoolElementTypeModel::Empty).then_some((0, 0, 0))
    }

    pub fn empty_place_result(&self) -> Option<bool> {
        (self.element_type == JigsawPoolElementTypeModel::Empty).then_some(true)
    }

    pub fn default_feature_jigsaw(&self) -> Option<DefaultFeatureJigsawModel> {
        (self.element_type == JigsawPoolElementTypeModel::Feature).then_some(
            DefaultFeatureJigsawModel {
                name: "minecraft:bottom",
                final_state: "minecraft:air",
                pool: "minecraft:empty",
                target: "minecraft:empty",
                joint: "rollable",
                orientation: "down_south",
            },
        )
    }

    pub fn placement_processors(&self, keep_jigsaws: bool) -> Vec<&'static str> {
        match self.element_type {
            JigsawPoolElementTypeModel::Single | JigsawPoolElementTypeModel::LegacySingle => {
                let mut processors = Vec::new();
                if self.element_type == JigsawPoolElementTypeModel::LegacySingle {
                    processors.push("minecraft:structure_and_air");
                } else {
                    processors.push("minecraft:structure_block");
                }
                if !keep_jigsaws {
                    processors.push("minecraft:jigsaw_replacement");
                }
                processors.extend(self.processors.iter().copied());
                processors.extend(self.projection.processor_ids().iter().copied());
                processors
            }
            _ => Vec::new(),
        }
    }
}

impl JigsawTemplatePoolModel {
    pub fn new(
        fallback: &'static str,
        raw_templates: Vec<JigsawTemplatePoolElementEntry>,
    ) -> Result<Self, String> {
        let mut expanded_template_count = 0_usize;
        for entry in &raw_templates {
            if !(1..=150).contains(&entry.weight) {
                return Err("template pool element weight must be in 1..=150".to_string());
            }
            expanded_template_count += entry.weight as usize;
        }
        Ok(Self {
            fallback,
            raw_templates,
            expanded_template_count,
        })
    }

    pub fn size(&self) -> usize {
        self.expanded_template_count
    }

    pub fn get_weighted_template_index(&self, mut expanded_index: usize) -> Option<usize> {
        if expanded_index >= self.expanded_template_count {
            return None;
        }
        for (index, entry) in self.raw_templates.iter().enumerate() {
            let weight = entry.weight as usize;
            if expanded_index < weight {
                return Some(index);
            }
            expanded_index -= weight;
        }
        None
    }

    pub fn element_at_expanded_index(
        &self,
        expanded_index: usize,
    ) -> Option<(usize, JigsawPoolElementModel)> {
        let raw_index = self.get_weighted_template_index(expanded_index)?;
        Some((raw_index, self.raw_templates[raw_index].element.clone()))
    }
}

