#!/usr/bin/env bash
# ==============================================================================
# Script: status.sh
# Finalidade: Consulta rápida de status do rádio Bluetooth, Wi-Fi e PipeWire
# ==============================================================================

set -euo pipefail

print_header() {
    printf "\n\033[1;36m=== %s ===\033[0m\n" "$*"
}

print_header "1. Estado de Energia do Bluetooth USB (sysfs)"
found=0
for dev in /sys/bus/usb/devices/*; do
    if [ -f "${dev}/idVendor" ] && [ -f "${dev}/idProduct" ]; then
        is_bt=0
        for iface in "${dev}"/*:*; do
            if [ -f "${iface}/bInterfaceClass" ]; then
                cls="$(cat "${iface}/bInterfaceClass" 2>/dev/null || true)"
                sub="$(cat "${iface}/bInterfaceSubClass" 2>/dev/null || true)"
                proto="$(cat "${iface}/bInterfaceProtocol" 2>/dev/null || true)"
                if [ "${cls}" = "e0" ] && [ "${sub}" = "01" ] && [ "${proto}" = "01" ]; then
                    is_bt=1
                    break
                fi
            fi
        done

        if [ "${is_bt}" -eq 1 ] || [ "$(cat "${dev}/idVendor" 2>/dev/null || true)" = "0bda" ]; then
            v="$(cat "${dev}/idVendor" 2>/dev/null || true)"
            p="$(cat "${dev}/idProduct" 2>/dev/null || true)"
            prod="$(cat "${dev}/product" 2>/dev/null || echo 'Bluetooth Radio')"
            printf "Dispositivo: %s [%s:%s] (%s)\n" "${prod}" "${v}" "${p}" "${dev}"
            printf "  power/control:        %s\n" "$(cat "${dev}/power/control" 2>/dev/null || echo 'N/A')"
            printf "  runtime_status:       %s\n" "$(cat "${dev}/power/runtime_status" 2>/dev/null || echo 'N/A')"
            printf "  autosuspend_delay_ms: %s\n" "$(cat "${dev}/power/autosuspend_delay_ms" 2>/dev/null || echo 'N/A')"
            found=1
        fi
    fi
done

if [ "${found}" -eq 0 ]; then
    echo "Nenhum controlador Bluetooth detectado no barramento USB."
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
