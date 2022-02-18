# bannana pho

[ring ring](https://gitlab.com/litecord/litecord)

---

### Environment Variables:

(Also found in `example.env`)

|       Variable       |                         Description                          |     Example      |
|:--------------------:|:------------------------------------------------------------:|:----------------:|
|    `LISTEN_ADDR`     |               Listen address of the websocket                |  `0.0.0.0:3621`  |
|     `REDIS_ADDR`     |            Redis database address (not yet used)             | `127.0.0.1:6379` |
|       `SECRET`       | Shared Secret, can be anything, must be the same on Litecord | `deez nuts 420`  |
| `HEARTBEAT_INTERVAL` |  Rate of which Litecord will send a heartbeat (in seconds)   |       `1`        |