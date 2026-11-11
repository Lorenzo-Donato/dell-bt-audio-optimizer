#!/usr/bin/env bash
# ==============================================================================
# Script: status.sh
# Finalidade: Consulta rápida de status do rádio Bluetooth, Wi-Fi e PipeWire
# ==============================================================================

set -euo pipefail

readonly REALTEK_VENDOR="0bda"
readonly REALTEK_BT_PRODUCT="c829"

print_header() {
    printf "\n\033[1;36m=== %s ===\033[0m\n" "$*"
}

print_header "1. Estado de Energia do Bluetooth USB (sysfs)"
found=0
for dev in /sys/bus/usb/devices/*; do
    if [ -f "${dev}/idVendor" ] && [ -f "${dev}/idProduct" ]; then
        v="$(cat "${dev}/idVendor" 2>/dev/null || true)"
        p="$(cat "${dev}/idProduct" 2>/dev/null || true)"
        if [ "${v}" = "${REALTEK_VENDOR}" ] && [ "${p}" = "${REALTEK_BT_PRODUCT}" ]; then
            printf "Dispositivo: Realtek Bluetooth Radio (%s)\n" "${dev}"
            printf "  power/control:        %s\n" "$(cat "${dev}/power/control" 2>/dev/null || echo 'N/A')"
            printf "  runtime_status:       %s\n" "$(cat "${dev}/power/runtime_status" 2>/dev/null || echo 'N/A')"
            printf "  autosuspend_delay_ms: %s\n" "$(cat "${dev}/power/autosuspend_delay_ms" 2>/dev/null || echo 'N/A')"
            found=1
        fi
    fi
done
if [ "${found}" -eq 0 ]; then
    echo "Controlador Realtek Bluetooth não encontrado no barramento USB."
fi

print_header "2. Fones de Ouvido e Perfil de Áudio (pactl)"
if command -v pactl >/dev/null 2>&1; then
    pactl list cards 2>/dev/null | grep -E 'Name: bluez_card|device.description|Active Profile|bluez5.enable-hw-volume' || echo "Nenhum fone bluez conectado."
else
    echo "pactl não instalado."
fi

print_header "3. Conexão Wi-Fi Ativa (nmcli)"
if command -v nmcli >/dev/null 2>&1; then
    nmcli -t -m tabular -f IN-USE,SSID,CHAN,FREQ,RATE,SIGNAL dev wifi 2>/dev/null | grep '^\*' || echo "Nenhuma rede Wi-Fi conectada."
else
    echo "nmcli não instalado."
fi
printf "\n"
