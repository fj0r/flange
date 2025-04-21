# quick start

load `__.nu`
```nu
overlay use -r __.nu as __ -p
```

start redpanda and create topic
```nu
__ rpk test
```

start server
```nu
__ dev serve
```

start clients
```nu
__ rpk consume event # redpanda
__ dev client # websocket
```
- Just enter some characters and press Enter to send the message. The message will appear in the `event` queue.


send message
```nu
__ rpk send --topic push (open data/message/event.yaml) --patch {receiver: ["user_1"]}
__ send event.yaml
```

- The sent messages will appear in the websocket client.
- `rpk send` listens to the `push` queue and then sends the message to the websocket (the user ID must be correct).

- `send` directly sends the message via HTTP POST; if no user is specified, it sends to all online users.
