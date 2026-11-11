#!/usr/bin/env bash
# ==============================================================================
# Script: apply-system-tuning.sh
# Finalidade: Implantação de regras permanentes de Kernel (modprobe) e UDEV
# Requisitos: Privilégios de Superusuário (Root)
# Diretrizes: Bash strict mode (set -euo pipefail), descoberta dinâmica e idempotência
# ==============================================================================

set -euo pipefail

readonly SCRIPT_NAME="$(basename "$0")"
readonly MODPROBE_BTUSB="/etc/modprobe.d/btusb.conf"
readonly MODPROBE_RTW88="/etc/modprobe.d/rtw88.conf"
readonly UDEV_RULE_FILE="/etc/udev/rules.d/50-bluetooth-realtek.rules"

# ------------------------------------------------------------------------------
# Funções Utilitárias de Logging
# ------------------------------------------------------------------------------
log_info() {
    printf "\033[1;36m[INFO]\033[0m %s\n" "$*"
}

log_success() {
    printf "\033[1;32m[SUCESSO]\033[0m %s\n" "$*"
}

log_error() {
    printf "\033[1;31m[ERRO]\033[0m %s\n" "$*" >&2
}

# ------------------------------------------------------------------------------
# Validação de Privilégios
# ------------------------------------------------------------------------------
assert_root() {
    if [ "${EUID}" -ne 0 ]; then
        log_error "Este script requer privilégios de superusuário (root)."
        log_error "Por favor, execute novamente utilizando: sudo $0"
        exit 1
    fi
}

# ------------------------------------------------------------------------------
# Descoberta Dinâmica de Hardware Bluetooth
# ------------------------------------------------------------------------------
discover_bluetooth_ids() {
    BT_VENDOR="0bda"
    BT_PRODUCT="c829"

    for dev in /sys/bus/usb/devices/*; do
        if [ -f "${dev}/idVendor" ] && [ -f "${dev}/idProduct" ]; then
            for iface in "${dev}"/*:*; do
                if [ -f "${iface}/bInterfaceClass" ]; then
                    local cls sub proto
                    cls="$(cat "${iface}/bInterfaceClass" 2>/dev/null || true)"
                    sub="$(cat "${iface}/bInterfaceSubClass" 2>/dev/null || true)"
                    proto="$(cat "${iface}/bInterfaceProtocol" 2>/dev/null || true)"
                    if [ "${cls}" = "e0" ] && [ "${sub}" = "01" ] && [ "${proto}" = "01" ]; then
                        BT_VENDOR="$(cat "${dev}/idVendor")"
                        BT_PRODUCT="$(cat "${dev}/idProduct")"
                        log_info "Detectado rádio Bluetooth USB dinamicamente: ${BT_VENDOR}:${BT_PRODUCT}"
                        return 0
                    fi
                fi
            done
        fi
    done

    log_info "Utilizando identificadores de contingência (Realtek): ${BT_VENDOR}:${BT_PRODUCT}"
}

# ------------------------------------------------------------------------------
# 1. Configuração de Módulos do Kernel (modprobe.d)
# ------------------------------------------------------------------------------
configure_kernel_modules() {
    log_info "Gravando parâmetros do módulo btusb em ${MODPROBE_BTUSB}..."
    cat << 'EOF' > "${MODPROBE_BTUSB}"
# Desativa autosuspend nativo do kernel para controladores Bluetooth USB
# IMPORTANTE: reset=0 desativa o reset de hardware agressivo que pode travar o barramento xHCI
options btusb enable_autosuspend=n reset=0
EOF

    log_info "Gravando parâmetros do módulo rtw88 (Wi-Fi/Coexistência) em ${MODPROBE_RTW88}..."
    cat << 'EOF' > "${MODPROBE_RTW88}"
# Previne suspensão agressiva da placa combo Realtek
options rtw88_pci disable_aspm=y
options rtw88_core disable_lps_deep=y
EOF

    log_success "Arquivos de modprobe configurados com sucesso."
}

# ------------------------------------------------------------------------------
# 2. Configuração de Regras de UDEV
# ------------------------------------------------------------------------------
configure_udev_rules() {
    log_info "Criando regra UDEV em ${UDEV_RULE_FILE} para ${BT_VENDOR}:${BT_PRODUCT}..."
    cat << EOF > "${UDEV_RULE_FILE}"
# Desativa economia de energia em tempo de execução para Bluetooth USB (${BT_VENDOR}:${BT_PRODUCT})
ACTION=="add", SUBSYSTEM=="usb", ATTR{idVendor}=="${BT_VENDOR}", ATTR{idProduct}=="${BT_PRODUCT}", ATTR{power/control}="on"
EOF

    log_info "Recarregando regras do daemon udev..."
    udevadm control --reload-rules
    udevadm trigger --subsystem-match=usb

    log_success "Regras UDEV implantadas e aplicadas."
}

# ------------------------------------------------------------------------------
# 3. Forçar Imediatamente no Sysfs sem Necessidade de Reboot
# ------------------------------------------------------------------------------
enforce_sysfs_runtime_power() {
    log_info "Varrendo dispositivos USB em /sys/bus/usb/devices/..."
    local dev_dir
    local matched=0

    for dev_dir in /sys/bus/usb/devices/*; do
        if [ -f "${dev_dir}/idVendor" ] && [ -f "${dev_dir}/idProduct" ]; then
            local v p
            v="$(cat "${dev_dir}/idVendor" 2>/dev/null || true)"
            p="$(cat "${dev_dir}/idProduct" 2>/dev/null || true)"

            if [ "${v}" = "${BT_VENDOR}" ] && [ "${p}" = "${BT_PRODUCT}" ]; then
                local pctrl="${dev_dir}/power/control"
                if [ -w "${pctrl}" ]; then
                    echo "on" > "${pctrl}"
                    log_success "Forçado power/control='on' em: ${dev_dir}"
                    matched=1
                fi
            fi
        fi
    done

    if [ "${matched}" -eq 0 ]; then
        log_info "Dispositivo ${BT_VENDOR}:${BT_PRODUCT} será ajustado automaticamente na próxima conexão."
    fi
}

# ------------------------------------------------------------------------------
# Entrypoint
# ------------------------------------------------------------------------------
main() {
    log_info "=== Aplicando Otimizações de Kernel e UDEV para Bluetooth ==="
    assert_root
    discover_bluetooth_ids
    configure_kernel_modules
    configure_udev_rules
    enforce_sysfs_runtime_power
    log_success "=== Sistema completamente ajustado e persistente! ==="
}

main "$@"
