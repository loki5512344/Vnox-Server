// VNOX example plugin — moderation bot
// Runtime: Deno
// See docs/community/plugins.md for full API reference

const BANNED_WORDS = ["badword", "spam"];

const ws = new WebSocket("ws://localhost:7800/plugins");

ws.addEventListener("open", () => {
  console.log("[modbot] connected to VNOX gateway");
});

ws.addEventListener("message", async (event) => {
  const e = JSON.parse(event.data);

  if (e.event === "message.created") {
    const { message_id, channel_id, content } = e.data;
    const lower = content.toLowerCase();

    if (BANNED_WORDS.some((word) => lower.includes(word))) {
      await rpc("message.delete", { message_id, channel_id });
      console.log(`[modbot] deleted message ${message_id}`);
    }
  }
});

ws.addEventListener("close", () => {
  console.log("[modbot] disconnected");
});

function rpc(method: string, params: object): Promise<unknown> {
  return new Promise((resolve) => {
    const id = crypto.randomUUID();
    ws.send(JSON.stringify({ id, method, params }));
    ws.addEventListener("message", function handler(e) {
      const res = JSON.parse((e as MessageEvent).data);
      if (res.id === id) {
        ws.removeEventListener("message", handler);
        resolve(res.result);
      }
    });
  });
}
