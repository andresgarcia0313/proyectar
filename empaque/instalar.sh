#!/bin/bash
# Compila Proyectar y lo instala para el usuario actual: binario, icono y lanzador del menú.
# Uso: empaque/instalar.sh   (no pide sudo; desinstalar: empaque/instalar.sh --quitar)
set -euo pipefail
raiz="$(cd "$(dirname "$0")/.." && pwd)"
bin="$HOME/.local/bin/proyectar"
icono="$HOME/.local/share/icons/hicolor/scalable/apps/com.ingeniumcodex.proyectar.svg"
lanzador="$HOME/.local/share/applications/com.ingeniumcodex.proyectar.desktop"

if [ "${1:-}" = "--quitar" ]; then
  rm -f -- "$bin" "$icono" "$lanzador"
  update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
  echo "Proyectar desinstalado"; exit 0
fi

cargo build --profile fast -p app --manifest-path "$raiz/Cargo.toml"
install -Dm755 "$raiz/target/fast/proyectar" "$bin"
install -Dm644 "$raiz/crates/app/assets/proyectar.svg" "$icono"
install -Dm644 "$raiz/empaque/com.ingeniumcodex.proyectar.desktop" "$lanzador"
sed -i "s|^Exec=proyectar|Exec=$bin|" "$lanzador"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
desktop-file-validate "$lanzador" 2>/dev/null || true
echo "Proyectar instalado: menú de aplicaciones > Proyectar (o ejecuta: proyectar)"
