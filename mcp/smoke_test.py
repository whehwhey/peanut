#!/usr/bin/env python3
"""Smoke test for the Peanut MCP server.

Launches mcp/server.py as a real stdio MCP subprocess, connects as an MCP client,
lists the tools, and calls a few of them -- headlined by peanut_decide on a
Thue-Morse square query, which must come back TRUE.

Run:  python mcp/smoke_test.py   (uses the same interpreter, so run it from the
venv that has both `mcp` and `psutil` installed).
"""
import asyncio
import json
import os
import sys

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

HERE = os.path.dirname(os.path.abspath(__file__))
SERVER = os.path.join(HERE, "server.py")


def _payload(result):
    """Pull the structured dict out of a tool result across SDK shapes."""
    sc = getattr(result, "structuredContent", None) or getattr(result, "structured_content", None)
    if sc:
        return sc.get("result", sc) if isinstance(sc, dict) else sc
    for block in getattr(result, "content", []) or []:
        text = getattr(block, "text", None)
        if text:
            try:
                return json.loads(text)
            except Exception:
                return {"_raw": text}
    return {}


async def main():
    params = StdioServerParameters(command=sys.executable, args=[SERVER], env=dict(os.environ))
    async with stdio_client(params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()

            tools = (await session.list_tools()).tools
            names = sorted(t.name for t in tools)
            print("tools:", names)
            assert names == sorted([
                "peanut_decide", "peanut_prove", "peanut_witness",
                "peanut_fe", "peanut_sequence", "peanut_export",
            ]), names

            # Headline: does Thue-Morse contain a square? -> TRUE.
            square = ("def T 2 2 0 01 10 01\n"
                      "? E i. E n. n>=1 & (A t. t<n => T[i+t]=T[i+n+t])")
            r = _payload(await session.call_tool("peanut_decide", {"script": square}))
            print("peanut_decide square:", r.get("verdict"), "states=", r.get("states"),
                  "ms=", r.get("ms"))
            assert r.get("verdict") == "TRUE", r

            # peanut_prove: Thue-Morse is overlap-free -> no overlap -> FALSE.
            overlap = "E i. E n. n>=1 & (A t. t<=n => T[i+t]=T[i+n+t])"
            r = _payload(await session.call_tool(
                "peanut_prove", {"sequence_def": "def T 2 2 0 01 10 01", "sentence": overlap}))
            print("peanut_prove overlap:", r.get("verdict"))
            assert r.get("verdict") == "FALSE", r

            # peanut_witness: first position with T[i]=1.
            r = _payload(await session.call_tool(
                "peanut_witness", {"sequence_def": "def T 2 2 0 01 10 01", "formula": "T[i]=1"}))
            print("peanut_witness T[i]=1:", r.get("assignment"))
            assert r.get("assignment") == {"i": 1}, r

            # peanut_sequence: first 20 terms of Thue-Morse.
            r = _payload(await session.call_tool(
                "peanut_sequence", {"sequence_def": "def T 2 2 0 01 10 01", "n": 20}))
            print("peanut_sequence:", r.get("terms"))
            assert r.get("terms") == "01101001100101101001", r

            # peanut_fe: equality-of-factors automaton size for Thue-Morse.
            r = _payload(await session.call_tool(
                "peanut_fe", {"sequence_def": "def T 2 2 0 01 10 01"}))
            print("peanut_fe states:", r.get("states"))
            assert r.get("states") == 15, r

            # peanut_export: the DFAO of Thue-Morse.
            r = _payload(await session.call_tool(
                "peanut_export", {"sequence_def": "def T 2 2 0 01 10 01", "name": "T"}))
            aut = r.get("automaton") or {}
            print("peanut_export kind:", aut.get("kind"), "nstates=", aut.get("nstates"))
            assert aut.get("kind") == "dfao" and aut.get("nstates") == 2, r

            # Structured error path: a bogus command must not raise.
            r = _payload(await session.call_tool("peanut_decide", {"script": "def BOGUS"}))
            print("peanut_decide error path: ok=", r.get("ok"), "error=", r.get("error"))
            assert r.get("ok") is False and r.get("error"), r

    print("\nSMOKE TEST PASSED")


if __name__ == "__main__":
    asyncio.run(main())
