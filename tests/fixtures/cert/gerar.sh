#!/bin/bash
# Gera os certificados SINTÉTICOS usados nos testes de `CertInfo`/assinatura (tests/cert.rs).
#
# Não são ICP-Brasil: AC e titular são autoassinados aqui, CNPJ fictício e senha pública.
# A SEFAZ recusa qualquer coisa assinada com eles — servem só para provar que a crate abre o
# PFX, acha o certificado da empresa e assina igual no Windows (CAPI) e no Linux (OpenSSL).
#
# Os .pfx gerados são versionados (exceção no .gitignore). Rodar de novo troca as datas e as
# chaves, e os valores esperados em tests/cert.rs precisam ser atualizados junto.
#
# Uso (Linux/WSL, OpenSSL 3): bash tests/fixtures/cert/gerar.sh
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -f "$TMP"/*.pem "$TMP"/*.srl "$TMP"/*.csr && rmdir "$TMP"' EXIT
SENHA="senha-teste-dfe"

# AC sintética
openssl req -x509 -newkey rsa:2048 -nodes -utf8 -days 36500 \
  -keyout "$TMP/ac.key.pem" -out "$TMP/ac.pem" \
  -subj "/C=BR/O=ICP-Brasil/OU=Teste dfe/CN=AC SINTETICA DFE"

# Titular (e-CNPJ sintético), assinado pela AC
openssl req -newkey rsa:2048 -nodes -utf8 \
  -keyout "$TMP/empresa.key.pem" -out "$TMP/empresa.csr" \
  -subj "/C=BR/ST=SP/L=SÃO PAULO/O=ICP-Brasil/OU=Certificado SINTETICO/CN=EMPRESA TESTE DFE LTDA:12345678000195"
openssl x509 -req -in "$TMP/empresa.csr" -CA "$TMP/ac.pem" -CAkey "$TMP/ac.key.pem" \
  -CAcreateserial -days 36500 -out "$TMP/empresa.pem"

# PFX moderno (padrão do OpenSSL 3: AES-256-CBC + PBKDF2)
openssl pkcs12 -export -inkey "$TMP/empresa.key.pem" -in "$TMP/empresa.pem" \
  -certfile "$TMP/ac.pem" -passout "pass:$SENHA" -out "$DIR/sintetico_aes.pfx"

# PFX legado (RC2-40 + 3DES, como os A1 ICP-Brasil antigos) — exige o provider legacy
openssl pkcs12 -export -legacy -inkey "$TMP/empresa.key.pem" -in "$TMP/empresa.pem" \
  -certfile "$TMP/ac.pem" -passout "pass:$SENHA" -out "$DIR/sintetico_legado.pfx"

openssl x509 -in "$TMP/empresa.pem" -noout -subject -issuer -dates -nameopt utf8
echo "Gerados: $DIR/sintetico_aes.pfx e $DIR/sintetico_legado.pfx (senha: $SENHA)"
