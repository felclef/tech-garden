# Tech Garden - Cellular Lab Environment Fabric (C.L.E.F.)

C.L.E.F. é uma sandbox, um tech garden (nome mais bonitinho) para passar o tempo.

Lá na firma, eu venho falando de separar control e data planes faz um tempo e vou ver se dou uma aquecida com esse websítio.

E eu quero voltar a ter meu e-mail com domínio meu, currículo fora das redes sociais etc. E faz 10 anos que não faço nada em rust - estou enferrujado (kkkkkkkkjl)

pq não?

## Fase Atual: 1 - Célula Zero do Control Plane

### Objetivo
Validar a separação entre Control Plane e Data Plane, no esquema hello world e adicionar mais ~coisas.

### Quick Start

**Pré-requisitos:**
- tudo foi feito num linux Bazzite 43+ rodando distrobox Arch
- Docker 24+ e Docker Compose 5+
- o Bazzite nao tem Docker, então eu fiz um link pro sock e `docker ps` passou a funcionar na box Arch
- Rust 1.75+ (para desenvolvimento local, veja as imagens do Dockerfile tb)

**Subir o env:**

Na raiz:
```bash
docker-compose -f infra/docker/docker-compose.yaml up --build
```

**Validar as células:**

curl ou abre no browser e dá f5:
```bash
for i in {1..1000}; do curl -s http://localhost:8080 | jq -r '.cell_id'; done | sort | uniq -c
    333 cell-a
    334 cell-b
    333 cell-c


```

dashboard do traefik (lembrar q tem que fechar isso antes de por em hosting)

http://localhost:8080

debug que salvou várias vezes a vida, e.g. ver o load balancer:
```
docker exec techgarden-gateway wget -qO- http://localhost:8080/api/rawdata | jq '.routers."cells-lb@file"'
{
  "entryPoints": [
    "web"
  ],
  "service": "cells-weighted",
  "rule": "PathPrefix(`/`)",
  "priority": 100,
  "status": "enabled",
  "using": [
    "web"
  ]
}
```

**logs:**
```bash
docker-compose -f infra/docker/docker-compose.yaml logs -f cell-a cell-b cell-c

```

### Próximos Passos (Fase 2)
- kubernetes, video cassetes, os carro loco
- service discovery dinâmico (hash no redis e boas)
- persistir state com redis ou postgres ou algum schemaless (talvez cassandra por show and tell)
