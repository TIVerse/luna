//! Task planning
//!
//! Breaks complex commands into sequential action steps with dependencies.

use crate::brain::command_parser::IntentType;
use crate::brain::intent_classifier::ClassificationResult;
use std::collections::HashMap;
use tracing::{debug, info};

/// Action type for execution
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    /// Launch an application
    LaunchApp,
    /// Close an application
    CloseApp,
    /// Find a file
    FindFile,
    /// Open a folder
    OpenFolder,
    /// Control system
    SystemControl,
    /// Control volume
    VolumeControl,
    /// Manage windows
    WindowManagement,
    /// Control media playback
    MediaControl,
    /// Search the web
    SearchWeb,
    /// Create reminder
    CreateReminder,
    /// Take note
    TakeNote,
    /// Answer question
    AnswerQuestion,
    /// Get time
    GetTime,
    /// Get date
    GetDate,
    /// Wait/delay
    Wait,
}

/// Single action step in a task plan
#[derive(Debug, Clone)]
pub struct ActionStep {
    /// The action type
    pub action: ActionType,
    /// Parameters for the action
    pub params: HashMap<String, String>,
    /// Step number
    pub step_number: usize,
    /// Preconditions that must be met before execution
    pub preconditions: Vec<Precondition>,
    /// Expected postconditions after execution
    pub postconditions: Vec<Postcondition>,
    /// Can this step run in parallel with others?
    pub parallel_group: Option<usize>,
}

/// Precondition that must be satisfied before an action
#[derive(Debug, Clone, PartialEq)]
pub enum Precondition {
    /// Another step must complete first
    StepCompleted(usize),
    /// A specific state must be true
    StateCondition(String, String),
    /// A resource must be available
    ResourceAvailable(String),
    /// Minimum confidence threshold
    ConfidenceThreshold(f32),
}

/// Postcondition that is true after an action
#[derive(Debug, Clone, PartialEq)]
pub enum Postcondition {
    /// A state has changed
    StateChanged(String, String),
    /// A resource was created/modified
    ResourceModified(String),
    /// Action was successful
    Success,
}

/// Complete task plan with steps and dependencies
#[derive(Debug, Clone)]
pub struct TaskPlan {
    /// Sequential steps to execute
    pub steps: Vec<ActionStep>,
    /// Dependencies between steps (step_a, step_b) means step_b depends on step_a
    pub dependencies: Vec<(usize, usize)>,
    /// Original classification result
    pub classification: ClassificationResult,
    /// Parallel execution groups (steps in same group can run concurrently)
    pub parallel_groups: Vec<Vec<usize>>,
    /// Is this plan valid and executable?
    pub is_valid: bool,
    /// Validation errors if any
    pub validation_errors: Vec<String>,
}

/// Task planner that breaks commands into executable steps
pub struct TaskPlanner;

impl TaskPlanner {
    /// Create a new task planner
    pub fn new() -> Self {
        Self
    }

    /// Plan tasks from a classification result
    pub fn plan(&self, classification: ClassificationResult) -> TaskPlan {
        info!("Planning tasks for intent: {:?}", classification.intent);

        let steps = self.create_steps(&classification);
        let dependencies = self.compute_dependencies(&steps);
        let parallel_groups = self.compute_parallel_groups(&steps);

        debug!(
            "Created {} steps with {} dependencies and {} parallel groups",
            steps.len(),
            dependencies.len(),
            parallel_groups.len()
        );

        let mut plan = TaskPlan {
            steps,
            dependencies,
            classification,
            parallel_groups,
            is_valid: true,
            validation_errors: Vec::new(),
        };

        // Validate the plan
        self.validate_plan(&mut plan);

        plan
    }

    /// Create action steps from classification
    fn create_steps(&self, classification: &ClassificationResult) -> Vec<ActionStep> {
        let mut steps = Vec::new();

        match &classification.intent {
            IntentType::LaunchApp => {
                // Simple single-step task
                steps.push(ActionStep {
                    action: ActionType::LaunchApp,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![Precondition::ConfidenceThreshold(0.7)],
                    postconditions: vec![
                        Postcondition::Success,
                        Postcondition::StateChanged("app_running".to_string(), "true".to_string()),
                    ],
                    parallel_group: None,
                });
            }

            IntentType::CloseApp => {
                steps.push(ActionStep {
                    action: ActionType::CloseApp,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::FindFile => {
                // Multi-step: find file, then potentially open it
                steps.push(ActionStep {
                    action: ActionType::FindFile,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::OpenFolder => {
                steps.push(ActionStep {
                    action: ActionType::OpenFolder,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::SystemControl => {
                steps.push(ActionStep {
                    action: ActionType::SystemControl,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::VolumeControl => {
                steps.push(ActionStep {
                    action: ActionType::VolumeControl,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::MediaControl => {
                steps.push(ActionStep {
                    action: ActionType::MediaControl,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::SearchWeb => {
                steps.push(ActionStep {
                    action: ActionType::SearchWeb,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::Reminder => {
                steps.push(ActionStep {
                    action: ActionType::CreateReminder,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::Note => {
                steps.push(ActionStep {
                    action: ActionType::TakeNote,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::Question => {
                steps.push(ActionStep {
                    action: ActionType::AnswerQuestion,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::GetTime => {
                steps.push(ActionStep {
                    action: ActionType::GetTime,
                    params: HashMap::new(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::GetDate => {
                steps.push(ActionStep {
                    action: ActionType::GetDate,
                    params: HashMap::new(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::WindowManagement => {
                steps.push(ActionStep {
                    action: ActionType::WindowManagement,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }

            IntentType::Unknown => {
                // Create a generic answer question step
                steps.push(ActionStep {
                    action: ActionType::AnswerQuestion,
                    params: classification.entities.clone(),
                    step_number: 0,
                    preconditions: vec![],
                    postconditions: vec![Postcondition::Success],
                    parallel_group: None,
                });
            }
        }

        steps
    }

    /// Compute dependencies between steps
    fn compute_dependencies(&self, steps: &[ActionStep]) -> Vec<(usize, usize)> {
        let mut dependencies = Vec::new();

        // For multi-step tasks, add sequential dependencies
        for i in 1..steps.len() {
            dependencies.push((i - 1, i));
        }

        dependencies
    }

    /// Check if plan is executable
    pub fn is_executable(&self, plan: &TaskPlan) -> bool {
        // Plan is executable if it has at least one step
        !plan.steps.is_empty()
    }

    /// Get execution order considering dependencies
    pub fn get_execution_order(&self, plan: &TaskPlan) -> Vec<usize> {
        // Simple topological sort for now
        // Since we mostly have sequential dependencies, this is straightforward
        (0..plan.steps.len()).collect()
    }

    /// Compute parallel execution groups
    fn compute_parallel_groups(&self, steps: &[ActionStep]) -> Vec<Vec<usize>> {
        let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();

        for (idx, step) in steps.iter().enumerate() {
            if let Some(group_id) = step.parallel_group {
                groups.entry(group_id).or_insert_with(Vec::new).push(idx);
            }
        }

        groups.into_values().collect()
    }

    /// Validate a task plan
    fn validate_plan(&self, plan: &mut TaskPlan) {
        // Check for circular dependencies
        if self.has_circular_dependencies(plan) {
            plan.is_valid = false;
            plan.validation_errors
                .push("Circular dependencies detected".to_string());
        }

        // Check preconditions are satisfiable
        for (idx, step) in plan.steps.iter().enumerate() {
            for precond in &step.preconditions {
                match precond {
                    Precondition::StepCompleted(dep_idx) => {
                        if *dep_idx >= plan.steps.len() {
                            plan.is_valid = false;
                            plan.validation_errors.push(format!(
                                "Step {} depends on non-existent step {}",
                                idx, dep_idx
                            ));
                        }
                    }
                    Precondition::ConfidenceThreshold(threshold) => {
                        if plan.classification.confidence < *threshold {
                            plan.validation_errors.push(format!(
                                "Step {} requires confidence >= {:.2}, but current is {:.2}",
                                idx, threshold, plan.classification.confidence
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check parallel groups don't have dependencies on each other
        for group in &plan.parallel_groups {
            for &step_a in group {
                for &step_b in group {
                    if step_a != step_b {
                        if plan.dependencies.contains(&(step_a, step_b))
                            || plan.dependencies.contains(&(step_b, step_a))
                        {
                            plan.is_valid = false;
                            plan.validation_errors.push(format!(
                                "Parallel group contains dependent steps {} and {}",
                                step_a, step_b
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Check for circular dependencies using DFS
    fn has_circular_dependencies(&self, plan: &TaskPlan) -> bool {
        let n = plan.steps.len();
        let mut visited = vec![false; n];
        let mut rec_stack = vec![false; n];

        // Build adjacency list
        let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();
        for &(from, to) in &plan.dependencies {
            graph.entry(from).or_insert_with(Vec::new).push(to);
        }

        fn dfs(
            node: usize,
            graph: &HashMap<usize, Vec<usize>>,
            visited: &mut [bool],
            rec_stack: &mut [bool],
        ) -> bool {
            visited[node] = true;
            rec_stack[node] = true;

            if let Some(neighbors) = graph.get(&node) {
                for &neighbor in neighbors {
                    if !visited[neighbor] {
                        if dfs(neighbor, graph, visited, rec_stack) {
                            return true;
                        }
                    } else if rec_stack[neighbor] {
                        return true;
                    }
                }
            }

            rec_stack[node] = false;
            false
        }

        for i in 0..n {
            if !visited[i] && dfs(i, &graph, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }
}

impl Default for TaskPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::command_parser::IntentType;

    fn create_test_classification(intent: IntentType) -> ClassificationResult {
        let mut entities = HashMap::new();
        entities.insert("test".to_string(), "value".to_string());

        ClassificationResult {
            intent,
            confidence: 0.9,
            entities,
            alternatives: Vec::new(),
        }
    }

    #[test]
    fn test_plan_launch_app() {
        let planner = TaskPlanner::new();
        let classification = create_test_classification(IntentType::LaunchApp);

        let plan = planner.plan(classification);

        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].action, ActionType::LaunchApp);
    }

    #[test]
    fn test_plan_find_file() {
        let planner = TaskPlanner::new();
        let classification = create_test_classification(IntentType::FindFile);

        let plan = planner.plan(classification);

        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].action, ActionType::FindFile);
    }

    #[test]
    fn test_is_executable() {
        let planner = TaskPlanner::new();
        let classification = create_test_classification(IntentType::LaunchApp);
        let plan = planner.plan(classification);

        assert!(planner.is_executable(&plan));
    }

    #[test]
    fn test_execution_order() {
        let planner = TaskPlanner::new();
        let classification = create_test_classification(IntentType::LaunchApp);
        let plan = planner.plan(classification);

        let order = planner.get_execution_order(&plan);
        assert_eq!(order, vec![0]);
    }

    #[test]
    fn test_plan_unknown_intent() {
        let planner = TaskPlanner::new();
        let classification = create_test_classification(IntentType::Unknown);

        let plan = planner.plan(classification);

        // Should still create a plan (with answer question fallback)
        assert!(!plan.steps.is_empty());
    }
}
