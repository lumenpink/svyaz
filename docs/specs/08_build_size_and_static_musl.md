# Especificação 08: Otimização de Tamanho Binário e Linking Estático musl

- **ID do Ticket:** #8
- **Status:** Aprovado
- **Autora:** Lumen
- **Alinhamento:** GOST 1.5.0 (Rust)

## 1. Contexto & Motivação
O Svyaz (`svyaz` e `sv`) é um micro cliente HTTP/HTTPS em Rust puro desenvolvido para ser ultraleve, portátil e executável em qualquer ambiente POSIX/Linux.
Para atingir o menor consumo de armazenamento possível e garantir compatibilidade estática universal sem falhas de interpretador dinâmico ELF (`/lib/ld-musl*`):
1. Declara-se o perfil `[profile.release]` focado em tamanho mínimo (`opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`).
2. Configura-se `.cargo/config.toml` forçando o linking estático da C runtime (`target-feature=+crt-static`) para targets musl.

## 2. Requisitos Técnicos
- `[profile.release]` configurado com `opt-level = "z"` no `Cargo.toml`.
- `.cargo/config.toml` contendo `rustflags = ["-C", "target-feature=+crt-static"]` para `x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`.
- Workflow de compilação multiarch preservando linking estático.

## 3. Critérios de Aceitação
- Teste automatizado validando as diretivas em `Cargo.toml` e `.cargo/config.toml`.
- Binários resultantes sem símbolos de depuração (`stripped`) e estáticos.
