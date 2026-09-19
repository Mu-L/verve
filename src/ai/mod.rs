//! Change-plan executor shared by the MCP server (`verve mcp`): the typed
//! operation vocabulary (`ChangeOp` / `OpProtocol`) that AI clients use to
//! inspect and modify the Verve workspace. Only the ops layer ships in the
//! Community Edition — the LLM client and agent loop are upstream-only.

pub mod ops;
