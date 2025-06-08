use std::collections::VecDeque;

use crate::action::ActionNode;


#[allow(dead_code)]
#[derive(Default)]
pub struct TodoUndo {
    todo: Option<ActionNode>,
    undo: VecDeque<ActionNode>,
    done: VecDeque<ActionNode>,
}