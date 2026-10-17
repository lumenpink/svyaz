# Svyaz (sv)

> **Micro HTTP/HTTPS client in pure Rust**  
> *Minimalist · Pure Rustls · Drop-in curl subset · Gosplan Ministry of Communications (Связь)*

[![CI](https://codeberg.org/lumenpink/svyaz/actions/workflows/ci.yml/badge.svg)](https://codeberg.org/lumenpink/svyaz/actions)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
[![Website](https://img.shields.io/badge/docs-svyaz.2lp.in-purple)](https://svyaz.2lp.in)

---

## 1. Visão Geral

O **Svyaz** (e seu atalho imediato **`sv`**) é um micro-cliente HTTP e HTTPS desenvolvido em **Rust puro**, concebido para fornecer requisições de rede ultrarrápidas, seguras e com footprint mínimo para scripts de automação, ferramentas de CI e ambientes minimalistas (como BusyBox / Alpine).

O nome homenageia o histórico *Министерство связи СССР* (Ministério de Comunicações da URSS).

Diferente do `curl` tradicional (que suporta dezenas de protocolos legados e links dinâmicos para bibliotecas C do sistema):
* **Rustls estático e puro:** Suporte a HTTPS/TLS 1.2 e 1.3 nativo em Rust, sem depender do OpenSSL do sistema operacional ou da libc dinâmica.
* **Subset focado e compatível com curl:** Aceita as flags essenciais (`-s`, `-f`, `-o`, `-m`, `-X`, `-H`, `-d`, `-w`, `-L`, `-I`, `-fsSL`) servindo como substituto direto e imediato em scripts shell.
* **Binário ultraleve e inicialização instantânea:** Compilado em binário estático único sem overhead de runtime.

---

## 2. Instalação

### 2.1 A partir do Código-Fonte (Cargo)
```bash
git clone ssh://git@codeberg.org/lumenpink/svyaz.git
cd svyaz
cargo install --path .
```
Isso instalará os binários **`svyaz`** e **`sv`** no seu `~/.cargo/bin` (ou `~/.local/bin`).

### 2.2 Compilação Manual Rápida
```bash
cargo build --release
cp target/release/sv ~/.local/bin/sv
cp target/release/svyaz ~/.local/bin/svyaz
```

---

## 3. Uso & Exemplos

```bash
sv [opções] <URL>
# ou
svyaz [opções] <URL>
```

### 3.1 Exemplos Rápidos

```bash
# Requisição GET simples
sv https://api.github.com/zen

# Modo silencioso com timeout de 3 segundos
sv -s -m 3 https://codeberg.org/api/v1/version

# Download de arquivo seguindo redirecionamentos (idêntico a curl -fsSL)
sv -fsSL https://example.com/archive.tar.gz -o /tmp/archive.tar.gz

# Requisição POST com JSON e Headers de autorização
sv -X POST \
  -H "Authorization: token $CODEBERG_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Bug report"}' \
  "https://codeberg.org/api/v1/repos/lumenpink/svyaz/issues"

# Obter apenas o código de status HTTP (200, 404, etc.)
sv -s -o /dev/null -w "%{http_code}\n" https://codeberg.org/api/v1/version
```

### 3.2 Tabela de Opções

| Opção | Atalho Combinado | Descrição |
| :--- | :--- | :--- |
| `-X, --request <MÉTODO>` | `-X POST` | Método HTTP (`GET`, `POST`, `PUT`, `PATCH`, `HEAD`) |
| `-H, --header <HEADER>` | `-H "Name: Value"` | Cabeçalho HTTP customizado (múltiplos permitidos) |
| `-d, --data <DADOS>` | `-d '{"foo":"bar"}'` | Corpo da requisição (ou `@arquivo`, ou `-` para stdin) |
| `-s, --silent` | `-s` | Modo silencioso (sem saída de progresso na stderr) |
| `-f, --fail` | `-f` | Retorna código de erro `22` em respostas HTTP 4xx ou 5xx |
| `-o, --output <ARQUIVO>` | `-o /caminho/arquivo` | Escreve resposta no arquivo em vez do stdout |
| `-m, --max-time <SEGUNDOS>` | `-m 5` | Timeout máximo da conexão e transferência |
| `-w, --write-out <FORMATO>` | `-w "%{http_code}\n"` | Formata código de saída (ex: `%{http_code}`) |
| `-I, --head` | `-I` | Executa requisição `HEAD` |
| `-L, --location` | `-L` | Segue redirecionamentos HTTP 3xx automaticamente |
| `-fsSL` | Combinado | Combinação padrão para scripts de download robustos |

---

## 4. Integração com o CCCP

O Svyaz foi projetado como cliente HTTP de referência para o ecossistema CCCP:
- Verificação de atualizações (`cccp check-update`)
- Sincronização de tickets (`cccp tickets sync`)
- Despacho de issues pelo Divã (`cccp diva`)
- Sincronização com forjas no catálogo (`cccp catalog`)

---

## 5. Licença & Matriz Modular

Em conformidade com a norma técnica [GOST v1.1.0](.gost/GOST.md):

* **Código-Fonte e Binário CLI (`src/`):** Licenciados sob a [Apache License 2.0 (Apache-2.0)](LICENSE).
* **Identidade Visual, Emblema e Insígnia (`site/assets/logo.svg`):** Licenciados sob a [Creative Commons Atribuição-SemDerivações 4.0 Internacional (CC-BY-ND 4.0)](LICENSE-LOGO).
* **Mídias e Ilustrações:** Licenciados sob a [Creative Commons Atribuição-CompartilhaIgual 4.0 Internacional (CC-BY-SA 4.0)](LICENSE-MEDIA).
