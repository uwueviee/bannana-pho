# bannana pho

[ring ring](https://gitlab.com/litecord/litecord)

---

### Environment Variables:

(Also found in `example.env`)

|       Variable       |                         Description                          |     Example     | Required? |
|:--------------------:|:------------------------------------------------------------:|:---------------:|:---------:|
|    `LISTEN_ADDR`     |               Listen address of the websocket                | `0.0.0.0:3621`  |           |
|       `SECRET`       | Shared Secret, can be anything, must be the same on Litecord | `deez nuts 420` |    [x]    |
| `HEARTBEAT_INTERVAL` |  Rate of which Litecord will send a heartbeat (in seconds)   |       `1`       |           |
|     `REDIS_HOST`     |                   Redis database hostname                    |   `127.0.0.1`   |           |
|     `REDIS_PORT`     |                     Redis database port                      |     `6379`      |           |
