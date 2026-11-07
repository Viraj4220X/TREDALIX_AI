use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct FeatureId(String);

impl FeatureId {
    pub fn new(value: impl AsRef<str>) -> Result<Self, String> {
        let value = value.as_ref();
        if value.is_empty()
            || value.len() > 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(
                "Feature ID must be 1-64 lowercase ASCII letters, digits, or underscores".into(),
            );
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FeatureId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureStatus {
    Available,
    Experimental,
    Planned,
    Unavailable,
    PlatformUnsupported,
    RequiresConfiguration,
    RequiresNetwork,
    RequiresProviderEntitlement,
}

impl FeatureStatus {
    fn can_run(self) -> bool {
        matches!(self, Self::Available | Self::Experimental)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Web,
    Windows,
    Android,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub id: FeatureId,
    pub name: String,
    pub description: String,
    pub status: FeatureStatus,
    pub default_enabled: bool,
    pub platforms: Vec<Platform>,
    pub dependencies: Vec<FeatureId>,
    pub offline_capable: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureGateError {
    UnknownFeature(FeatureId),
    Disabled(FeatureId),
    Unavailable {
        feature: FeatureId,
        reason: Option<String>,
    },
    DependencyDisabled {
        feature: FeatureId,
        dependency: FeatureId,
    },
}

impl std::fmt::Display for FeatureGateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFeature(feature) => {
                write!(formatter, "Unknown feature: {}", feature.as_str())
            }
            Self::Disabled(feature) => write!(formatter, "{} is disabled", feature.as_str()),
            Self::Unavailable { feature, reason } => {
                write!(formatter, "{} is unavailable", feature.as_str())?;
                if let Some(reason) = reason {
                    write!(formatter, ": {reason}")?;
                }
                Ok(())
            }
            Self::DependencyDisabled {
                feature,
                dependency,
            } => write!(
                formatter,
                "{} requires {} to be enabled",
                feature.as_str(),
                dependency.as_str()
            ),
        }
    }
}

impl std::error::Error for FeatureGateError {}

#[derive(Debug, Clone)]
pub struct FeatureRegistry {
    definitions: HashMap<FeatureId, FeatureDefinition>,
}

impl FeatureRegistry {
    pub fn new(definitions: Vec<FeatureDefinition>) -> Result<Self, String> {
        let mut indexed = HashMap::with_capacity(definitions.len());
        for definition in definitions {
            if indexed.insert(definition.id.clone(), definition).is_some() {
                return Err("Feature registry contains a duplicate ID".into());
            }
        }
        for definition in indexed.values() {
            if definition.name.trim().is_empty() || definition.description.trim().is_empty() {
                return Err(format!(
                    "Feature {} must have a name and description",
                    definition.id.as_str()
                ));
            }
            for dependency in &definition.dependencies {
                if !indexed.contains_key(dependency) {
                    return Err(format!(
                        "Feature {} depends on an unknown feature {}",
                        definition.id.as_str(),
                        dependency.as_str()
                    ));
                }
            }
            if definition.default_enabled && !definition.status.can_run() {
                return Err(format!(
                    "Unavailable feature {} cannot be enabled by default",
                    definition.id.as_str()
                ));
            }
            if definition.default_enabled
                && definition.dependencies.iter().any(|dependency| {
                    indexed
                        .get(dependency)
                        .is_some_and(|entry| !entry.default_enabled || !entry.status.can_run())
                })
            {
                return Err(format!(
                    "Feature {} cannot default on while a dependency is disabled",
                    definition.id.as_str()
                ));
            }
        }
        Self::validate_acyclic(&indexed)?;
        Ok(Self {
            definitions: indexed,
        })
    }

    pub fn get(&self, id: &FeatureId) -> Result<&FeatureDefinition, FeatureGateError> {
        self.definitions
            .get(id)
            .ok_or_else(|| FeatureGateError::UnknownFeature(id.clone()))
    }

    pub fn check_enabled(
        &self,
        id: &FeatureId,
        enabled: &HashMap<FeatureId, bool>,
        platform: Platform,
    ) -> Result<(), FeatureGateError> {
        let definition = self.get(id)?;
        if !definition.status.can_run() || !definition.platforms.contains(&platform) {
            return Err(FeatureGateError::Unavailable {
                feature: id.clone(),
                reason: definition.blocked_reason.clone(),
            });
        }
        if !enabled.get(id).copied().unwrap_or(false) {
            return Err(FeatureGateError::Disabled(id.clone()));
        }
        for dependency in &definition.dependencies {
            self.check_enabled(dependency, enabled, platform)
                .map_err(|_| FeatureGateError::DependencyDisabled {
                    feature: id.clone(),
                    dependency: dependency.clone(),
                })?;
        }
        Ok(())
    }

    fn validate_acyclic(definitions: &HashMap<FeatureId, FeatureDefinition>) -> Result<(), String> {
        fn visit(
            id: &FeatureId,
            definitions: &HashMap<FeatureId, FeatureDefinition>,
            visiting: &mut HashSet<FeatureId>,
            visited: &mut HashSet<FeatureId>,
        ) -> Result<(), String> {
            if visited.contains(id) {
                return Ok(());
            }
            if !visiting.insert(id.clone()) {
                return Err(format!("Feature dependency cycle includes {}", id.as_str()));
            }
            for dependency in &definitions[id].dependencies {
                visit(dependency, definitions, visiting, visited)?;
            }
            visiting.remove(id);
            visited.insert(id.clone());
            Ok(())
        }

        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        for id in definitions.keys() {
            visit(id, definitions, &mut visiting, &mut visited)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        FeatureDefinition, FeatureGateError, FeatureId, FeatureRegistry, FeatureStatus, Platform,
    };

    fn feature(
        id: &str,
        dependencies: &[&str],
        status: FeatureStatus,
        default_enabled: bool,
    ) -> FeatureDefinition {
        FeatureDefinition {
            id: FeatureId::new(id).unwrap(),
            name: id.into(),
            description: format!("Test feature {id}"),
            status,
            default_enabled,
            platforms: vec![Platform::Web],
            dependencies: dependencies
                .iter()
                .map(|dependency| FeatureId::new(dependency).unwrap())
                .collect(),
            offline_capable: true,
            blocked_reason: None,
        }
    }

    #[test]
    fn unknown_feature_id_fails_closed() {
        assert!(FeatureId::new("Backtesting").is_err());
        let registry = FeatureRegistry::new(vec![feature(
            "backtesting",
            &[],
            FeatureStatus::Available,
            true,
        )])
        .unwrap();
        assert!(matches!(
            registry.check_enabled(
                &FeatureId::new("not_registered").unwrap(),
                &HashMap::new(),
                Platform::Web
            ),
            Err(FeatureGateError::UnknownFeature(_))
        ));
    }

    #[test]
    fn dependencies_and_platform_capabilities_are_enforced() {
        let registry = FeatureRegistry::new(vec![
            feature("csv_import", &[], FeatureStatus::Available, true),
            feature(
                "backtesting",
                &["csv_import"],
                FeatureStatus::Available,
                true,
            ),
        ])
        .unwrap();
        let enabled = HashMap::from([
            (FeatureId::new("csv_import").unwrap(), false),
            (FeatureId::new("backtesting").unwrap(), true),
        ]);
        assert!(matches!(
            registry.check_enabled(
                &FeatureId::new("backtesting").unwrap(),
                &enabled,
                Platform::Web
            ),
            Err(FeatureGateError::DependencyDisabled { .. })
        ));
        let enabled = HashMap::from([
            (FeatureId::new("csv_import").unwrap(), true),
            (FeatureId::new("backtesting").unwrap(), true),
        ]);
        assert!(
            registry
                .check_enabled(
                    &FeatureId::new("backtesting").unwrap(),
                    &enabled,
                    Platform::Android
                )
                .is_err()
        );
    }

    #[test]
    fn rejects_unknown_dependencies_cycles_and_enabled_unavailable_features() {
        assert!(
            FeatureRegistry::new(vec![feature(
                "backtesting",
                &["missing"],
                FeatureStatus::Available,
                true,
            )])
            .is_err()
        );
        assert!(
            FeatureRegistry::new(vec![
                feature("one", &["two"], FeatureStatus::Available, true),
                feature("two", &["one"], FeatureStatus::Available, true),
            ])
            .is_err()
        );
        assert!(
            FeatureRegistry::new(vec![feature(
                "live_data",
                &[],
                FeatureStatus::Unavailable,
                true,
            )])
            .is_err()
        );
        assert!(
            FeatureRegistry::new(vec![
                feature("csv_import", &[], FeatureStatus::Available, false),
                feature(
                    "backtesting",
                    &["csv_import"],
                    FeatureStatus::Available,
                    true,
                ),
            ])
            .is_err()
        );
    }

    #[test]
    fn feature_definition_serializes_with_stable_snake_case_status() {
        let definition = feature("backtesting", &[], FeatureStatus::Available, true);
        let encoded = serde_json::to_string(&definition).unwrap();
        assert!(encoded.contains(r#""status":"available""#));
        let decoded: FeatureDefinition = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, definition);
    }
}
