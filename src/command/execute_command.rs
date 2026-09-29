//! `/execute`: `net.minecraft.server.commands.ExecuteCommand`.
//!
//! The command is parsed once into an [`ExecuteChain`] (Brigadier's `ContextChain`): a list of
//! [`ExecuteStage`] modifiers (redirects and forks) ending in an [`ExecuteTerminal`]. The chain
//! is then run by [`super::execute_runner`], which reproduces `BuildContexts`, `ExecuteCommand`
//! (task) and the result-callback semantics.

use super::execute_args::*;
use super::execute_condition_support::CommandHeightmap;
use super::execute_conditions::ExecuteCondition;
use super::execute_runner::run_execute_chain;
use super::execute_store::StoreSpec;
use super::*;

/// The axes selected by `execute align <axes>` (`SwizzleArgument`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct AlignAxes {
    pub(super) x: bool,
    pub(super) y: bool,
    pub(super) z: bool,
}

impl AlignAxes {
    /// `SwizzleArgument.parse`: each of `x`, `y`, `z` at most once.
    fn parse(input: &str) -> Result<Self, CommandError> {
        let mut axes = Self::default();
        for axis in input.chars() {
            let slot = match axis {
                'x' => &mut axes.x,
                'y' => &mut axes.y,
                'z' => &mut axes.z,
                _ => return Err(CommandError::InvalidSyntax),
            };
            if *slot {
                return Err(CommandError::InvalidSyntax);
            }
            *slot = true;
        }
        Ok(axes)
    }
}

/// The `execute on <relation>` operations (`ExecuteCommand.createRelationOperations`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OnRelation {
    Owner,
    Leasher,
    Target,
    Attacker,
    Vehicle,
    Controller,
    Origin,
    Passengers,
}

impl OnRelation {
    fn parse(input: &str) -> Result<Self, CommandError> {
        Ok(match input {
            "owner" => Self::Owner,
            "leasher" => Self::Leasher,
            "target" => Self::Target,
            "attacker" => Self::Attacker,
            "vehicle" => Self::Vehicle,
            "controller" => Self::Controller,
            "origin" => Self::Origin,
            "passengers" => Self::Passengers,
            _ => return Err(CommandError::InvalidSyntax),
        })
    }
}

/// One redirect/fork link of the chain.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ExecuteStage {
    As(String),
    At(String),
    PositionedPos(Coordinates),
    PositionedAs(String),
    PositionedOver(CommandHeightmap),
    RotatedRot([WorldCoordinate; 2]),
    RotatedAs(String),
    FacingEntity {
        targets: String,
        anchor: EntityAnchor,
    },
    FacingPos(Coordinates),
    Align(AlignAxes),
    Anchored(EntityAnchor),
    In(String),
    Summon(String),
    On(OnRelation),
    Store(StoreSpec),
    Condition {
        expected: bool,
        condition: ExecuteCondition,
    },
}

impl ExecuteStage {
    /// Whether the node is registered with `fork(...)` rather than `redirect(...)`; a forking
    /// node turns error output off for the rest of the chain (`ChainModifiers.setForked`).
    pub(super) fn is_forking(&self) -> bool {
        matches!(
            self,
            Self::As(_)
                | Self::At(_)
                | Self::PositionedAs(_)
                | Self::RotatedAs(_)
                | Self::FacingEntity { .. }
                | Self::On(_)
                | Self::Condition { .. }
        )
    }
}

/// What a parsed chain finally executes.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ExecuteTerminal {
    /// `run <command>`: the redirect back to the command root.
    Command(String),
    /// A chain that ends on a condition uses that node's own `.executes(...)` handler.
    Condition {
        expected: bool,
        condition: ExecuteCondition,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ExecuteChain {
    pub(super) stages: Vec<ExecuteStage>,
    pub(super) terminal: ExecuteTerminal,
}

/// Parses `execute ...` tokens (`tokens[0] == "execute"`) into a chain.
///
/// `run execute ...` flattens into the same chain exactly like the `run` redirect to the root
/// followed by the `execute` literal does in Brigadier's `ContextChain`.
pub(super) fn parse_execute_chain(
    state: &ServerCommandState,
    tokens: &[&str],
) -> Result<ExecuteChain, CommandError> {
    if tokens.first() != Some(&"execute") {
        return Err(CommandError::InvalidSyntax);
    }
    let mut stages = Vec::new();
    let mut index = 1;
    loop {
        let keyword = *tokens.get(index).ok_or(CommandError::InvalidSyntax)?;
        match keyword {
            "run" => match tokens.get(index + 1) {
                None => return Err(CommandError::InvalidSyntax),
                Some(&"execute") => index += 2,
                Some(_) => {
                    return Ok(ExecuteChain {
                        stages,
                        terminal: ExecuteTerminal::Command(tokens[index + 1..].join(" ")),
                    })
                }
            },
            "if" | "unless" => {
                let (condition, used) = ExecuteCondition::parse(state, &tokens[index + 1..])?;
                index += 1 + used;
                let expected = keyword == "if";
                if index == tokens.len() {
                    // `if function` has no `.executes(...)`, so it cannot end a command.
                    if matches!(condition, ExecuteCondition::Function(_)) {
                        return Err(CommandError::InvalidSyntax);
                    }
                    return Ok(ExecuteChain {
                        stages,
                        terminal: ExecuteTerminal::Condition {
                            expected,
                            condition,
                        },
                    });
                }
                stages.push(ExecuteStage::Condition {
                    expected,
                    condition,
                });
            }
            _ => {
                let (stage, used) = parse_modifier(&tokens[index..])?;
                stages.push(stage);
                index += used;
            }
        }
    }
}

/// Parses one non-conditional modifier; returns the stage and the tokens it used.
fn parse_modifier(tokens: &[&str]) -> Result<(ExecuteStage, usize), CommandError> {
    let arg = |index: usize| tokens.get(index).copied().ok_or(CommandError::InvalidSyntax);
    let selector = |index: usize| -> Result<String, CommandError> {
        let selector = arg(index)?;
        validate_entity_selector(selector)?;
        Ok(selector.to_string())
    };
    let vec3 = |index: usize| -> Result<Coordinates, CommandError> {
        Coordinates::parse_vec3([arg(index)?, arg(index + 1)?, arg(index + 2)?])
    };
    Ok(match arg(0)? {
        "as" => (ExecuteStage::As(selector(1)?), 2),
        "at" => (ExecuteStage::At(selector(1)?), 2),
        "store" => {
            let (spec, used) = StoreSpec::parse(&tokens[1..])?;
            (ExecuteStage::Store(spec), 1 + used)
        }
        "positioned" => match arg(1)? {
            "as" => (ExecuteStage::PositionedAs(selector(2)?), 3),
            "over" => (
                ExecuteStage::PositionedOver(CommandHeightmap::parse(arg(2)?)?),
                3,
            ),
            _ => (ExecuteStage::PositionedPos(vec3(1)?), 4),
        },
        "rotated" => match arg(1)? {
            "as" => (ExecuteStage::RotatedAs(selector(2)?), 3),
            _ => (
                ExecuteStage::RotatedRot(parse_rotation([arg(1)?, arg(2)?])?),
                3,
            ),
        },
        "facing" => match arg(1)? {
            "entity" => (
                ExecuteStage::FacingEntity {
                    targets: selector(2)?,
                    anchor: parse_entity_anchor(arg(3)?)?,
                },
                4,
            ),
            _ => (ExecuteStage::FacingPos(vec3(1)?), 4),
        },
        "align" => (ExecuteStage::Align(AlignAxes::parse(arg(1)?)?), 2),
        "anchored" => (ExecuteStage::Anchored(parse_entity_anchor(arg(1)?)?), 2),
        "in" => (ExecuteStage::In(parse_resource_identifier(arg(1)?)?), 2),
        "summon" => (ExecuteStage::Summon(parse_resource_identifier(arg(1)?)?), 2),
        "on" => (ExecuteStage::On(OnRelation::parse(arg(1)?)?), 2),
        _ => return Err(CommandError::InvalidSyntax),
    })
}

/// Entry point of the `execute` literal: `ExecuteCommand.register`.
pub(super) fn execute_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let chain = parse_execute_chain(state, parts)?;
    run_execute_chain(state, permissions, &chain)
}
