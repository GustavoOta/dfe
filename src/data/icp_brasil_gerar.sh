#!/bin/bash
# Regenera src/data/icp_brasil.pem — cadeia ICP-Brasil (raízes + ACs intermediárias) publicada
# pelo ITI, que a crate soma à lista de ACs do sistema nos clientes mTLS **fora do Windows**.
#
# Por quê: a maioria dos servidores da SEFAZ (SP, PR, RS/SVRS, BA, GO, MS, MT, AM, PE…) usa
# certificado ICP-Brasil. O Windows conhece a raiz e baixa a intermediária sozinho; a lista do
# Linux (Mozilla) não tem nenhuma das duas, e o OpenSSL não baixa intermediária.
#
# Saída determinística: só certificados válidos hoje, sem repetidos, ordenados pela impressão
# digital SHA-256. Só troca o arquivo se tudo der certo.
#
# Uso (Linux/WSL, com curl, unzip e openssl): bash src/data/icp_brasil_gerar.sh
set -euo pipefail

URL="https://acraiz.icpbrasil.gov.br/credenciadas/CertificadosAC-ICP-Brasil/ACcompactado.zip"
DESTINO="$(cd "$(dirname "$0")" && pwd)/icp_brasil.pem"
TMP="$(mktemp -d)"
trap 'rm -f "$TMP"/zip "$TMP"/novo.pem "$TMP"/lista "$TMP"/certs/* && rmdir "$TMP"/certs "$TMP"' EXIT

curl -fsSL -o "$TMP/zip" "$URL"
mkdir "$TMP/certs"
if command -v unzip >/dev/null; then
  unzip -jq "$TMP/zip" -d "$TMP/certs"
else
  # Sem unzip: Python (sem pastas, como o -j). Nomes vêm do zip — só o basename é usado.
  python3 -I -c 'import os,sys,zipfile
z = zipfile.ZipFile(sys.argv[1])
for i in z.infolist():
    n = os.path.basename(i.filename)
    if n and not i.is_dir():
        open(os.path.join(sys.argv[2], n), "wb").write(z.read(i))' "$TMP/zip" "$TMP/certs"
fi

: > "$TMP/lista"
vencidos=0
for f in "$TMP"/certs/*; do
  if openssl x509 -in "$f" -noout 2>/dev/null; then fmt=PEM
  elif openssl x509 -inform DER -in "$f" -noout 2>/dev/null; then fmt=DER
  else echo "ignorado (não é certificado): $(basename "$f")" >&2; continue
  fi
  if ! openssl x509 -inform "$fmt" -in "$f" -noout -checkend 0 >/dev/null; then
    vencidos=$((vencidos + 1)); continue
  fi
  fp="$(openssl x509 -inform "$fmt" -in "$f" -noout -fingerprint -sha256 | cut -d= -f2)"
  printf '%s\t%s\t%s\n' "$fp" "$fmt" "$f" >> "$TMP/lista"
done

{
  echo "# Cadeia ICP-Brasil (ITI: $URL)"
  echo "# Gerado por src/data/icp_brasil_gerar.sh em $(date -u +%Y-%m-%d). Não editar à mão."
  sort -u -t$'\t' -k1,1 "$TMP/lista" | while IFS=$'\t' read -r fp fmt f; do
    echo "# $(openssl x509 -inform "$fmt" -in "$f" -noout -subject -nameopt utf8,sep_comma_plus | sed 's/^subject=//')"
    openssl x509 -inform "$fmt" -in "$f"
  done
} > "$TMP/novo.pem"

total="$(grep -c 'BEGIN CERTIFICATE' "$TMP/novo.pem")"
[ "$total" -gt 0 ] || { echo "nenhum certificado — arquivo mantido" >&2; exit 1; }
mv "$TMP/novo.pem" "$DESTINO"
echo "icp_brasil.pem: $total certificados (vencidos descartados: $vencidos)"
