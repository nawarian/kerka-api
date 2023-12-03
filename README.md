Kerka API
---

Uma API rest escrita em Rust e preparada para rodar em Cloudflare Workers.

## Desenvolvimento

Para testar suas alterações em ambiente local:

```
$ npm run dev
```

Para publicar suas alterações sem passar pela build:

```
$ npm run deploy
```

## Publicação em ambiente de produção

Existe uma Github Action que compila, empacota e publica os workers
diretamente no Cloudflare de produção.

Sempre que um commit entra no branch `main` uma publicação em produção
é feita em poucos minutos.
