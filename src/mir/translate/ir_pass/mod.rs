//! IR passes that transform the MIR (Mid-level Intermediate Representation) of a program.
//!
//! Including:
//!
//! * Critical Edge Elimination
//! * Live Variable Analysis
//! * PHI Node Elimination

pub(super) mod critical_edge;
pub(super) mod phi_node_ellimination;
