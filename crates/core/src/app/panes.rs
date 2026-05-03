#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PaneRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaneNode {
    Leaf { session_id: usize },
    Split {
        axis: SplitAxis,
        ratio: f32,
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaneTree {
    root: PaneNode,
    active_session_id: usize,
}

impl PaneTree {
    pub fn new(root_session_id: usize) -> Self {
        Self {
            root: PaneNode::Leaf {
                session_id: root_session_id,
            },
            active_session_id: root_session_id,
        }
    }

    pub fn active_session_id(&self) -> usize {
        self.active_session_id
    }

    pub fn root(&self) -> &PaneNode {
        &self.root
    }

    pub fn session_ids(&self) -> Vec<usize> {
        let mut ids = Vec::new();
        self.root.collect_session_ids(&mut ids);
        ids
    }

    pub fn contains_session(&self, session_id: usize) -> bool {
        self.root.contains_session(session_id)
    }

    pub fn set_active_session(&mut self, session_id: usize) -> bool {
        if !self.contains_session(session_id) {
            return false;
        }
        self.active_session_id = session_id;
        true
    }

    pub fn split_active(&mut self, axis: SplitAxis, new_session_id: usize) -> bool {
        if self.contains_session(new_session_id) {
            return false;
        }
        if self
            .root
            .split_leaf(self.active_session_id, axis, new_session_id)
        {
            self.active_session_id = new_session_id;
            true
        } else {
            false
        }
    }

    pub fn remove_session(&mut self, session_id: usize) -> bool {
        if !self.contains_session(session_id) {
            return false;
        }
        if matches!(self.root, PaneNode::Leaf { session_id: leaf } if leaf == session_id) {
            return false;
        }

        let replacement = self.root.remove_session(session_id);
        if let Some(node) = replacement {
            self.root = node;
        }
        self.root.reindex_after_removal(session_id);

        let remaining = self.session_ids();
        if let Some(&active) = remaining.first() {
            if self.active_session_id == session_id || !remaining.contains(&self.active_session_id) {
                self.active_session_id = active;
            } else if self.active_session_id > session_id {
                self.active_session_id -= 1;
            }
        }
        true
    }

    pub fn focus_next(&mut self) -> bool {
        let ids = self.session_ids();
        let Some(position) = ids.iter().position(|id| *id == self.active_session_id) else {
            return false;
        };
        let next = ids[(position + 1) % ids.len()];
        self.active_session_id = next;
        true
    }

    pub fn focus_previous(&mut self) -> bool {
        let ids = self.session_ids();
        let Some(position) = ids.iter().position(|id| *id == self.active_session_id) else {
            return false;
        };
        let prev = if position == 0 {
            ids[ids.len() - 1]
        } else {
            ids[position - 1]
        };
        self.active_session_id = prev;
        true
    }

    pub fn layout(&self, rect: PaneRect, gap: f32) -> Vec<(usize, PaneRect)> {
        let mut panes = Vec::new();
        self.root.layout(rect, gap, &mut panes);
        panes
    }
}

impl PaneNode {
    fn collect_session_ids(&self, ids: &mut Vec<usize>) {
        match self {
            PaneNode::Leaf { session_id } => ids.push(*session_id),
            PaneNode::Split { first, second, .. } => {
                first.collect_session_ids(ids);
                second.collect_session_ids(ids);
            }
        }
    }

    fn contains_session(&self, session_id: usize) -> bool {
        match self {
            PaneNode::Leaf { session_id: leaf } => *leaf == session_id,
            PaneNode::Split { first, second, .. } => {
                first.contains_session(session_id) || second.contains_session(session_id)
            }
        }
    }

    fn split_leaf(&mut self, target: usize, axis: SplitAxis, new_session_id: usize) -> bool {
        match self {
            PaneNode::Leaf { session_id } if *session_id == target => {
                let current = *session_id;
                *self = PaneNode::Split {
                    axis,
                    ratio: 0.5,
                    first: Box::new(PaneNode::Leaf {
                        session_id: current,
                    }),
                    second: Box::new(PaneNode::Leaf {
                        session_id: new_session_id,
                    }),
                };
                true
            }
            PaneNode::Leaf { .. } => false,
            PaneNode::Split { first, second, .. } => {
                first.split_leaf(target, axis, new_session_id)
                    || second.split_leaf(target, axis, new_session_id)
            }
        }
    }

    fn remove_session(&mut self, target: usize) -> Option<PaneNode> {
        match self {
            PaneNode::Leaf { session_id } if *session_id == target => None,
            PaneNode::Leaf { session_id } => Some(PaneNode::Leaf {
                session_id: *session_id,
            }),
            PaneNode::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                let left = first.remove_session(target);
                let right = second.remove_session(target);
                match (left, right) {
                    (Some(left), Some(right)) => Some(PaneNode::Split {
                        axis: *axis,
                        ratio: *ratio,
                        first: Box::new(left),
                        second: Box::new(right),
                    }),
                    (Some(node), None) | (None, Some(node)) => Some(node),
                    (None, None) => None,
                }
            }
        }
    }

    fn reindex_after_removal(&mut self, removed_session_id: usize) {
        match self {
            PaneNode::Leaf { session_id } => {
                if *session_id > removed_session_id {
                    *session_id -= 1;
                }
            }
            PaneNode::Split { first, second, .. } => {
                first.reindex_after_removal(removed_session_id);
                second.reindex_after_removal(removed_session_id);
            }
        }
    }

    fn layout(&self, rect: PaneRect, gap: f32, panes: &mut Vec<(usize, PaneRect)>) {
        match self {
            PaneNode::Leaf { session_id } => panes.push((*session_id, rect)),
            PaneNode::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                let ratio = ratio.clamp(0.2, 0.8);
                match axis {
                    SplitAxis::Vertical => {
                        let gap = gap.min(rect.width.max(0.0));
                        let first_width = ((rect.width - gap).max(0.0) * ratio).max(0.0);
                        let second_width = (rect.width - gap - first_width).max(0.0);
                        first.layout(
                            PaneRect::new(rect.x, rect.y, first_width, rect.height),
                            gap,
                            panes,
                        );
                        second.layout(
                            PaneRect::new(rect.x + first_width + gap, rect.y, second_width, rect.height),
                            gap,
                            panes,
                        );
                    }
                    SplitAxis::Horizontal => {
                        let gap = gap.min(rect.height.max(0.0));
                        let first_height = ((rect.height - gap).max(0.0) * ratio).max(0.0);
                        let second_height = (rect.height - gap - first_height).max(0.0);
                        first.layout(
                            PaneRect::new(rect.x, rect.y, rect.width, first_height),
                            gap,
                            panes,
                        );
                        second.layout(
                            PaneRect::new(rect.x, rect.y + first_height + gap, rect.width, second_height),
                            gap,
                            panes,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PaneRect, PaneTree, SplitAxis};

    #[test]
    fn split_active_should_create_two_leaf_layouts() {
        let mut tree = PaneTree::new(0);
        assert!(tree.split_active(SplitAxis::Vertical, 1));

        let panes = tree.layout(PaneRect::new(0.0, 0.0, 100.0, 40.0), 4.0);
        assert_eq!(tree.active_session_id(), 1);
        assert_eq!(panes.len(), 2);
        assert_eq!(panes[0].0, 0);
        assert_eq!(panes[1].0, 1);
        assert_eq!(panes[0].1.width, 48.0);
        assert_eq!(panes[1].1.x, 52.0);
    }

    #[test]
    fn remove_session_should_collapse_tree_and_reindex_remaining_sessions() {
        let mut tree = PaneTree::new(0);
        assert!(tree.split_active(SplitAxis::Vertical, 1));
        assert!(tree.set_active_session(0));
        assert!(tree.split_active(SplitAxis::Horizontal, 2));

        assert!(tree.remove_session(1));
        assert_eq!(tree.session_ids(), vec![0, 1]);
        assert!(tree.contains_session(1));
        assert!(!tree.contains_session(2));
    }

    #[test]
    fn focus_next_should_follow_leaf_order() {
        let mut tree = PaneTree::new(0);
        assert!(tree.split_active(SplitAxis::Vertical, 1));
        assert!(tree.split_active(SplitAxis::Horizontal, 2));

        assert_eq!(tree.active_session_id(), 2);
        assert!(tree.focus_next());
        assert_eq!(tree.active_session_id(), 0);
        assert!(tree.focus_next());
        assert_eq!(tree.active_session_id(), 1);
    }

    #[test]
    fn focus_previous_should_walk_backwards_through_leaf_order() {
        let mut tree = PaneTree::new(0);
        assert!(tree.split_active(SplitAxis::Vertical, 1));
        assert!(tree.split_active(SplitAxis::Horizontal, 2));

        assert_eq!(tree.active_session_id(), 2);
        assert!(tree.focus_previous());
        assert_eq!(tree.active_session_id(), 1);
        assert!(tree.focus_previous());
        assert_eq!(tree.active_session_id(), 0);
    }
}
