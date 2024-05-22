# Sveio

A geography game inspired by [Posio](https://github.com/abrenaut/posio), written in 🔥🚀Rust.

## Deployment

Sveio is available as a docker image. If you use `docker run` you can run the following to start it:

```bash
docker run -d -p 8085:8085 --env-file=.env ghcr.io/erb3/sveio:main
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
    env_file:
      - .env
```

## Configuration

The server uses the following environmental variables, and it also parses the .env file if present:

- `SVEIO_PORT`: The port to serve on. Defaults to `8085`.

There is an example env file, see [`.env.example`](https://github.com/Erb3/sveio/blob/main/.env.example)

## Socket.io

Sveio uses [Socketioxide](https://github.com/Totodore/socketioxide)

Here are some resources to get you started with socketioxide:

- [I never thought I'd use Socket.Io again](https://www.youtube.com/watch?v=HEhhWL1oUTM) by Dreams of Code
- The examples

## Credits

- [Posio](https://github.com/abrenaut/posio) by [abrenaut](https://github.com/abrenaut)
- [JSON of cities](https://github.com/abrenaut/posio/blob/master/game/data/cities.json) by [abrenaur](https://github.com/abrenaut)
- [Leaflet.js](https://leafletjs.com/)
