//! `dappswarm` — publish and install Dappnode packages over Swarm.
//!
//! See `PLAN.md` and `CONTEXT.md` at the repo root for the project-wide
//! design. The library exposes a small layered API: a thin HTTP client
//! to a local `antd` (`swarm`), a Single-Owner-Chunk builder (`soc`),
//! a Swarm-Feed convention on top of SOCs (`feed`), a DNP bundle model
//! (`bundle`), and orchestrators (`publish`, `resolve`, `install`).

pub mod bundle;
pub mod feed;
pub mod install;
pub mod publish;
pub mod resolve;
pub mod soc;
pub mod swarm;
