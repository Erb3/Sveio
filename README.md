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
docker run -d -p 8085:8085 ghcr.io/erb3/sveio:main
```

Alternatively, you can use the following docker compose:

```yml
services:
  sveio:
    container_name: sveio
    image: ghcr.io/erb3/sveio:main
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

The server uses the following environmental variables, and it also parses the .env file if present:

- `SVEIO_PORT`: The port to serve on. Defaults to `8085`.

There is an example env file, see [`.env.example`](https://github.com/Erb3/sveio/blob/main/.env.example)

## Socket.io

Sveio uses [Socketioxide](https://github.com/Totodore/socketioxide)

Here are some resources to get you started with Socketioxide:

- [I never thought I'd use Socket.Io again](https://www.youtube.com/watch?v=HEhhWL1oUTM) by Dreams of Code
- The examples

## Credits

- [Posio](https://github.com/abrenaut/posio) by [Abrenaut](https://github.com/abrenaut)
- [JSON of cities](https://github.com/abrenaut/posio/blob/master/game/data/cities.json) by [Abrenaut](https://github.com/abrenaut)
- [Leaflet.js](https://leafletjs.com/)
