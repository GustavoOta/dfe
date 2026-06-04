# Instalação

```toml
[dependencies]
dfe = "0.5.8"
```

> **Schemas XSD** (`PL_010b_NT2025_002_v1.21`) estão embutidos no binário via `include_bytes!`.

---

## Plataformas suportadas

### Windows ✅

As operações com certificado digital (abertura do `.pfx`, assinatura RSA-SHA1 e extração de dados) são realizadas via **Windows CryptoAPI (CAPI)** usando a crate `windows-sys`. Não há dependência de OpenSSL — o build é mais rápido e o binário não carrega nenhuma biblioteca criptográfica de terceiros.

O CAPI suporta nativamente os algoritmos legados dos certificados ICP-Brasil (RC2-40-CBC, 3DES), independentemente da versão do Windows. O certificado **não precisa estar instalado** no repositório de certificados do sistema — ele é carregado em memória a partir do arquivo `.pfx` e descartado ao final de cada operação.

### Linux / macOS 🔜

Ainda não suportado. As funções de certificado retornam erro em plataformas não-Windows. Ver [roadmap](notas-roadmap.md).

---

## libxml2 — Windows (linking estático)

Esta crate usa `libxml2` para validação XSD dos documentos fiscais. No Windows, o linker busca `libxml2.dll` por padrão, o que causa o erro abaixo ao executar o binário em máquinas sem a biblioteca instalada:

```
A execução de código não pode continuar porque libxml2.dll não foi encontrado.
Reinstalando o programa para corrigir o problema.
```

Para embutir `libxml2` estaticamente no executável (sem DLL), siga os passos abaixo.

### Passo 1 — Configurar o `Cargo.toml` do projeto consumidor

No `Cargo.toml` do seu projeto (o que depende de `dfe`), adicione a seção de metadata do vcpkg:

```toml
[package.metadata.vcpkg]
git = "https://github.com/microsoft/vcpkg"
branch = "master"
install = ["libxml2"]

[package.metadata.vcpkg.target]
x86_64-pc-windows-msvc = { triplet = "x64-windows-static", install = ["libxml2"] }
```

> O `Cargo.toml` da crate `dfe` já contém essa configuração. A seção precisa existir no **crate raiz** do seu projeto para que `cargo-vcpkg` a leia.

### Passo 2 — Instalar o `cargo-vcpkg`

```powershell
cargo install cargo-vcpkg
```

### Passo 3 — Baixar e compilar o vcpkg

Execute no diretório raiz do seu projeto:

```powershell
cargo vcpkg build
```

Este comando clona o repositório do vcpkg dentro da pasta da crate `dfe` (em `<caminho-dfe>/vcpkg/`) e compila o `libxml2` com o triplet `x64-windows-static`.

**A primeira execução pode levar vários minutos** — o vcpkg compila `libxml2` e suas dependências (`zlib`, `lzma`, `iconv`) a partir do código-fonte.

#### Solução de problemas: conflito de `.gitignore`

Se o `cargo vcpkg build` falhar com:

```
error: Your local changes to the following files would be overwritten by merge:
        .gitignore
Please commit your changes or stash them before you merge.
```

Descarte a alteração local no clone do vcpkg e execute novamente:

```powershell
git -C "<caminho-dfe>/vcpkg" checkout -- .gitignore
cargo vcpkg build
```

#### Solução de problemas: `vcpkg.exe` desatualizado

Se o vcpkg clone sofreu um pull grande e o `vcpkg.exe` ficou incompatível com os scripts atuais, re-bootstrap:

```powershell
& "<caminho-dfe>/vcpkg/bootstrap-vcpkg.bat" -disableMetrics
```

Depois continue com o passo 4.

### Passo 4 — Configurar as variáveis de ambiente

As variáveis devem apontar para o vcpkg instalado pela etapa anterior. No PowerShell:

```powershell
$env:VCPKG_ROOT    = "<caminho-dfe>\vcpkg"
$env:VCPKGRS_TRIPLET = "x64-windows-static"
```

Para persistir entre sessões, adicione ao seu perfil do PowerShell (`$PROFILE`) ou configure nas variáveis de ambiente do sistema (Painel de Controle → Variáveis de Ambiente).

> Substitua `<caminho-dfe>` pelo caminho absoluto do diretório onde a crate `dfe` está clonada. Exemplo: `D:\Projetos\dfe`.

### Passo 5 — Limpar o cache de compilação

O `cargo` armazena em cache os resultados de compilação anteriores, incluindo a forma como as bibliotecas nativas foram linkadas. Para que o linker passe a usar a versão estática do `libxml2`, é necessário limpar esse cache:

```powershell
cargo clean
```

> Este passo é **obrigatório** na primeira vez após configurar as variáveis de ambiente. Sem ele, o Rust pode reutilizar o build anterior que linkava `libxml2` de forma dinâmica.

### Passo 6 — Compilar

```powershell
cargo build --release
```

O `libxml2` será linkado estaticamente. O `.exe` gerado não terá dependência de `libxml2.dll`.

### Alternativa rápida (sem compilação estática)

Se precisar distribuir antes de configurar o vcpkg, copie `libxml2.dll` para a mesma pasta do executável. A DLL pode ser obtida via:

```powershell
# Com vcpkg (triplet dinâmico)
vcpkg install libxml2:x64-windows
```

Ou baixando o MinGW e copiando `libxml2-2.dll` da pasta `bin/`.
