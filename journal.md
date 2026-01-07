# Journal - Tech Garden - Cellular Lab Environment Fabric

## 2025-01-07 | Fase 1: Alicerce (célula... zero? unknown?? control plane!)

### Decisões

**Arquitetura Base:**
- separação clara entre Control Plane (traefik) e Data Plane (cells, shards)
- cada célula é uma instância isolada identificada por `CELL_ID`
- traefik atua como gateway único (porta 80) com load balancing round-robin, ez
- ideia é sair de um hellow orld, depois fazer meu (micro) blog e talvez um encurtador de url (pq sim)

**Instrumentação:**
- logs estruturados em JSON via `tracing-subscriber`, mas nada muito robusto no começo
- contexto propagado via `request_id` (UUID v4, bad perform, ok) injetado em cada requisição
- header customizado `X-TechGarden-Cell` identifica qual shard processou a request
- """spans""" (rs) de tracing capturam `cell_id`, `request_id`, `method` e `path`

**Build Strategy:**
- Dockerfile multi-stage reduz imagem final (cheat de buildar no rust slim e rodar no debian)
- cache de dependências no primeiro stage acelera rebuilds, pelo que dizem por aí... search "docker rust cargo cache", muitas referências estão no web archives, prefiro não citar diretamente <\_<'
- docker-compose para validação local, antes de K8s e hosting deploy (cloudflared)

**Pontos de Atenção:**
- health checks configurados, mas não tem `curl` na imagem final
- traefik expõe dashboard na porta 8080 sem autenticação (talvez cloudflare passe um pano)
- todas as células compartilham o mesmo route (PathPrefix `/`) para testar lb

**Misc/Tactics/cheats**
- AIs mais atrapalham do que ajudam, mas ajudam como search hubs
    - tentei companions, só ferraram com meu nvim
    - tentei cloude cli (custa caro, como bom capricorniano, eu me recuso a pagar - e não acredito em signos)
    - tentei ollama local, mas minha vcard (RTX 2060 SUPER) tem 8BG rs
    - llms náo sabem rust de qq forma, segui com apenas Gemini para criar prompts de context para o Claude, que virou meu rubber duck - e isso foi o gotcha para debugar rede, ggwp

---

### Implementação

**Load Balancing - a epopeia OSI:**
- tentativa #1: service único `cells` com múltiplos containers → traefik reclama que "service defined multiple times with different configurations" (faz sentido, cada container tem IP diferente)
- tentativa #2: routers separados com mesma rule/priority → traefik escolhe sempre o último (cell-c), não faz lb automático entre routers
- **solução final:** file provider (`dynamic.yaml`) + weighted service
  - docker provider detecta células individuais (`cell-a@docker`, `cell-b@docker`, `cell-c@docker`)
  - file provider cria router `cells-lb@file` com priority 100 (sobrescreve os docker routers)
  - weighted service distribui igualmente: `weight: 1` para cada célula
  - load balancing perfeito: 333/333/334 requests em teste de 1000 chamadas

**Health Checks:**
- docker healthcheck inicial usava `/dev/tcp` (bash advanced features) → falhou porque debian slim não tem bash completo
- solução: remover docker healthcheck, deixar traefik fazer via HTTP (que funciona)
- traefik health check configurado via labels: `loadbalancer.healthcheck.path=/health` + `interval=10s`

**Podman + Traefik:**
- socket em `/run/user/1000/podman/podman.sock` (não o default `/var/run/docker.sock`)
- precisou adicionar `--providers.docker.network=docker_techgarden` explicitamente
- volumes precisam de `:z,U` para SELinux funcionar (podman thing)

**Cloudflare Tunnel:**
- container `cloudflare/cloudflared:latest` na mesma network
- token via env var `CF_TUNNEL_TOKEN` (armazenado em `.env`, git ignored obviamente)
- service configurado no dashboard cloudflare: `garden.clef.net.br` → `http://techgarden-gateway:8000`
  - importante: usar o **nome do container** (`techgarden-gateway`) não localhost/127.0.0.1
  - porta 8000 (entrypoint web do traefik), não 8080 (dashboard)
- tunnel detecta automaticamente serviços na docker network, zero config extra no compose
- restart policy `unless-stopped` porque tunnel precisa ficar up always
- claro que se você fizer seu deploy disso, vai ficar com outro nome, outro DNS, etc.etc.

---

### Validação
```bash
# Load balancing confirmado (distribuição perfeita)
for i in {1..1000}; do curl -s http://localhost:8080 | jq -r '.cell_id'; done | sort | uniq -c
# Output: 333 cell-a, 333 cell-b, 334 cell-c

# Header de célula presente
curl -v http://localhost:8080 2>&1 | grep -i x-techgarden
# Output: X-Techgarden-Cell: cell-{a,b,c}

# Logs estruturados em JSON
docker logs techgarden-cell-a | jq
# Output: campos cell_id, request_id, timestamp presentes

# Acesso público via cloudflare tunnel
curl -s https://garden.clef.net.br | jq -r '.cell_id'
# Output: cell-{a,b,c} alternando
```

---

### Misc & Troubleshooting Highlights

**"Filtering unhealthy or starting container"**
- sintoma: traefik vê containers mas não cria routers/services
- causa: docker healthcheck falhando (comandos não existem na imagem)
- fix: remover docker healthcheck, traefik faz o próprio via HTTP

**"the service 'cells@docker' does not exist"**
- sintoma: router criado mas disabled, erro nos logs
- causa: múltiplos containers tentando criar mesmo service com configs diferentes
- fix: usar file provider com weighted service referenciando services individuais

**File provider carrega antes do docker provider**
- sintoma: erro `"the service 'cell-b@docker' does not exist"` no startup
- causa: race condition entre providers
- comportamento: erro é temporário, resolve sozinho quando docker provider termina (watch=true ajuda)
- solução: ignorar erro inicial, validar após ~10s que tudo estabilizou

**Podman socket permissions**
- add `user: "0:0"` no traefik (rootful container)
- add `security_opt: label=disable` 
- volume mount precisa ser `:z,U` (selinux + user namespace)

**Cloudflare tunnel não resolve localhost**
- sintoma: tunnel up mas não consegue alcançar traefik (502 bad gateway)
- causa: `localhost`/`127.0.0.1` resolve para o namespace do container do tunnel, não do traefik
- fix: usar nome do container `techgarden-gateway` (docker dns interno resolve automaticamente)

---

### Estrutura Atual
```
tech-garden % tree -ph -a -I 'target|.git'
[drwxr-xr-x   84]  .
├── [drwxr-xr-x   24]  apps
│   └── [drwxr-xr-x   78]  cell-service
│       ├── [-rw-r--r--  28K]  Cargo.lock
│       ├── [-rw-r--r--  509]  Cargo.toml
│       ├── [-rw-r--r--  761]  Dockerfile
│       └── [drwxr-xr-x   14]  src
│           └── [-rw-r--r-- 3.4K]  main.rs
├── [-rw-r--r--   26]  .gitignore
├── [drwxr-xr-x   12]  infra
│   └── [drwxr-xr-x   60]  docker
│       ├── [-rw-r--r-- 3.1K]  docker-compose.yaml
│       ├── [-rw-r--r--  247]  .env
│       └── [drwxr-xr-x   64]  traefik
│           ├── [-rw-r--r--  525]  dynamic.yaml
│           └── [-rw-r--r--  176]  traefik.yaml.deleted
├── [-rw-r--r-- 6.5K]  journal.md
└── [-rw-r--r-- 1.2K]  README.md

7 directories, 11 files
tech-garden %

```

---

### Status: ✅ Fase 1 (Passos 1-3) Completa + Tunnel Público

Próximos passos candidatos:
- [ ] adicionar redis para state compartilhado (testar failure scenarios)
- [ ] métricas (prometheus? ou só logs por enquanto?)
- [ ] migrar para kubernetes local (kind/minikube)
- [ ] implementar service discovery dinâmico
- [ ] gRPC para comunicação interna (HTTP/3 para externa)
- [ ] ssl/tls (cloudflare já termina, mas e2e encryption?)
