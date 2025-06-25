
use log::error;

use crate::action::ActionNode;


#[allow(dead_code)]
#[derive(Default)]
pub struct TodoUndo {
    todo: Option<ActionNode>,
    undo: Vec<ActionNode>,
    done: Vec<ActionNode>,
}

#[allow(dead_code)]
impl TodoUndo {
    /// this removes the returned action
    pub fn todo ( &mut self ) -> Option<ActionNode> {
        self.todo.take()  // leaves todo as None
    }

    // clears the done list
    pub fn add_todo ( &mut self, actions: ActionNode ) {
        if self.todo.is_some() {
            error!("adding todo action when a todo action is already present");
            // this destroys the previous todo item, probably without doing it
        }
        self.todo = Some(actions);
        self.done.clear();
    }

    pub fn add_done ( &mut self, actions: ActionNode ) {
        self.done.push(actions);
    }

    pub fn add_undo ( &mut self, actions: ActionNode ) {
        self.undo.push(actions);
    }

    pub fn clear_todo ( &mut self ) {
        self.todo = None;
    }

    pub fn clear_done ( &mut self ) {
        self.done.clear();
    }

    pub fn clear_undo ( &mut self ) {
        self.undo.clear();
    }
}
