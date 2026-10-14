#!/bin/bash
# Compila Proyectar y lo instala para el usuario actual: binario, icono y lanzador del menú.
# Uso: empaque/instalar.sh   (no pide sudo; desinstalar: empaque/instalar.sh --quitar)
# empaque/instalar.sh --sistema  (con sudo, una vez): nombre del equipo en Wi-Fi Direct y cortafuegos.
set -euo pipefail
raiz="$(cd "$(dirname "$0")/.." && pwd)"
bin="$HOME/.local/bin/proyectar"
icono="$HOME/.local/share/icons/hicolor/scalable/apps/com.ingeniumcodex.proyectar.svg"
lanzador="$HOME/.local/share/applications/com.ingeniumcodex.proyectar.desktop"

if [ "${1:-}" = "--quitar" ]; then
  rm -f -- "$bin" "$icono" "$lanzador"
  [ -e /etc/NetworkManager/dispatcher.d/90-proyectar-nombre ] && echo "Para quitar la parte del sistema: sudo rm /etc/NetworkManager/dispatcher.d/90-proyectar-nombre"
  update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
  echo "Proyectar desinstalado"; exit 0
fi

if [ "${1:-}" = "--sistema" ]; then
  sudo install -m 755 "$raiz/empaque/90-proyectar-nombre" /etc/NetworkManager/dispatcher.d/90-proyectar-nombre
  sudo /etc/NetworkManager/dispatcher.d/90-proyectar-nombre - up
  if command -v ufw >/dev/null && sudo ufw status | grep -q "Status: active\|Estado: activo"; then
    sudo ufw allow in on p2p-+ to any port 7236 proto tcp comment 'Proyectar Miracast RTSP'
    sudo ufw allow in on p2p-+ to any port 67 proto udp comment 'Proyectar DHCP Wi-Fi Direct'
    sudo ufw allow in on p2p-+ to any port 5001 proto udp comment 'Proyectar RTCP del televisor'
  fi
  echo "Sistema listo: el televisor verá el nombre $(hostname)"; exit 0
fi

cargo build --profile fast -p app --manifest-path "$raiz/Cargo.toml"
install -Dm755 "$raiz/target/fast/proyectar" "$bin"
install -Dm644 "$raiz/crates/app/assets/proyectar.svg" "$icono"
install -Dm644 "$raiz/empaque/com.ingeniumcodex.proyectar.desktop" "$lanzador"
sed -i "s|^Exec=proyectar|Exec=$bin|" "$lanzador"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
desktop-file-validate "$lanzador" 2>/dev/null || true
echo "Proyectar instalado: menú de aplicaciones > Proyectar (o ejecuta: proyectar)"
