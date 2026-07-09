# License

VNOX and LNEx are licensed under the **GNU General Public License v3.0 (GPL-3.0)**.

```
Copyright (C) 2026 VNOX Contributors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
```

Full license text: `LICENSE` in the repository root.

---

## What this means

You can:
- use VNOX for any purpose
- study and modify the source code
- distribute copies of VNOX
- distribute modified versions — but they must also be GPL-3.0

You cannot:
- distribute VNOX or derivatives under a proprietary license
- remove copyright notices

## Protocol specification

The LNEx protocol specification (`docs/02-protocol/`) is additionally
licensed under **CC0 1.0 Universal (Public Domain)**.

This means anyone can implement the LNEx protocol in any language,
under any license, without restriction. Third-party clients and servers
are explicitly encouraged.

## Plugins

Plugins you write for VNOX are your own code. The GPL does not automatically
apply to plugin code — plugins communicate with VNOX via WebSocket RPC,
which is considered a separate program. You may license your plugins however
you choose.
