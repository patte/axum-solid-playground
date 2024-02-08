import { createSignal, onCleanup } from "solid-js";
import { createStore } from "solid-js/store";
import { createReconnectingWS, WSMessage } from "@solid-primitives/websocket";
import { Input } from "./ui/input";
import { Card } from "./ui/card";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { send } from "vite";
import { TbSend } from "solid-icons/tb";

function ChatComponent() {
  let ref: HTMLDivElement | undefined;
  const hostAndPort = location.host;
  const ws = createReconnectingWS(
    location.origin.startsWith("https")
      ? `wss://${hostAndPort}/chat`
      : `ws://${hostAndPort}/chat`
  );
  const [chatMessages, setChatMessages] = createStore<string[]>([]);
  const [onlineCount, setOnlineCount] = createSignal(0);
  const [message, setMessage] = createSignal("");

  ws.addEventListener("message", (ev: any) => {
    const refIsScrolledToBottom =
      ref && ref.scrollHeight - ref.clientHeight - ref.scrollTop < 1;

    const incomingMessage = ev.data as string;

    if (incomingMessage.startsWith("🧮")) {
      const count = parseInt(incomingMessage.split("🧮")[1]);
      if (!isNaN(count)) {
        setOnlineCount(count);
      }
    }

    setChatMessages(chatMessages.length, ev.data as string);

    // if user scrolled up, don't scroll down
    if (ref && refIsScrolledToBottom) {
      ref.scrollTo(0, ref.scrollHeight - ref.clientHeight);
    }
  });

  const sendMessage = () => {
    if (ref) {
      ref.scrollTo(0, ref.scrollHeight - ref.clientHeight);
    }
    ws?.send(message());
    setMessage("");
  };

  return (
    <div class="p-4 mt-2">
      <div class="text-center">
        <Badge variant={"secondary"}>{onlineCount()} online</Badge>{" "}
      </div>
      <Card
        class="block p-2 mt-2 h-64 w-96 overflow-y-auto overscroll-contain"
        ref={(e) => (ref = e)}
      >
        {chatMessages.map((message) => {
          if (message.startsWith("👋")) {
            return (
              <div class="text-muted-foreground">{message.split("👋")[1]}</div>
            );
          }
          if (message.startsWith("💬")) {
            const [username, content] = message.split("💬")[1].split(": ");
            return (
              <div class="flex items-top space-x-2">
                <div class="font-bold">{username}</div>
                <div>{content}</div>
              </div>
            );
          }
        })}
      </Card>

      <div class="flex justify-end space-x-2">
        <Input
          class="mt-2"
          type="text"
          placeholder="Broadcast something..."
          value={message()}
          onInput={(e) => setMessage(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              sendMessage();
            }
          }}
        />
        <Button variant={"outline"} class="mt-2" onClick={sendMessage}>
          <TbSend />
        </Button>
      </div>
    </div>
  );
}

export default ChatComponent;
