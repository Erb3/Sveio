# Sveio

A geography game inspired by [Posio](https://github.com/abrenaut/posio), written in 🔥🚀Rust.
View the public instance at [sveio.shuttleapp.rs](https://sveio.shuttleapp.rs)!

## Deployment

> [!IMPORTANT]
> Sveio does not impose any rate limit itself.
> You have to do this yourself, with something like nginx.

### Docker

Sveio is available as a docker image. If you use `docker run` you can run the following to start it:

```bash
docker run -d -p 8085:8085 ghcr.io/erb3/sveio:latest
```

Alternatively, you can use the following docker compose:

```yml
services:
  sveio:
    container_name: sveio
    image: ghcr.io/erb3/sveio:latest
    ports:
      - 8085:8085
    restart: unless-stopped
```

### Shuttle

[Shuttle.rs](https://shuttle.rs) is supported for the demo server.
To enable shuttle support use the `shuttle` feature.

Deploy with Shuttle:

```shell
cargo shuttle deploy
```

## Configuration

| Name          | Description                        | Env                   | Flag              | Default |
| ------------- | ---------------------------------- | --------------------- | ----------------- | ------- |
| Port          | The HTTP port to listen to         | `SVEIO_PORT`          | `-p`              | 8085    |
| Logging       | Logging level                      | `SVEIO_LOGGING_LEVEL` | `-l`              | info    |
| Guess time    | Amount of seconds allowed to guess | `SVEIO_GUESS_TIME`    | `--guess-time`    | 7       |
| Showcase time | Amount of seconds to show markers  | `SVEIO_SHOWCASE_TIME` | `--showcase-time` | 3       |

For more information please run `sveio -h`.

## Socket.io

Sveio uses [Socketioxide](https://github.com/Totodore/socketioxide)

Here are some resources to get you started with Socketioxide:

- [I never thought I'd use Socket.Io again](https://www.youtube.com/watch?v=HEhhWL1oUTM) by Dreams of Code
- The examples

## Credits

- [Posio](https://github.com/abrenaut/posio) by [Abrenaut](https://github.com/abrenaut)
- [JSON of cities](https://github.com/abrenaut/posio/blob/master/game/data/cities.json) by [Abrenaut](https://github.com/abrenaut)
- [Leaflet.js](https://leafletjs.com/)
- [Carto maps](https://carto.com/)
