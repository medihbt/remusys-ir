//! IR passes that transform the MIR (Mid-level Intermediate Representation) of a program.
//!
//! Including:
//!
//! * Critical Edge Elimination
//! * Live Variable Analysis
//! * PHI Node Elimination

pub mod critical_edge;
pub mod phi_node_ellimination;
