#!/usr/bin/env bash
# ==============================================================================
# Script: apply-system-tuning.sh
# Finalidade: Implantação de regras permanentes de Kernel (modprobe) e UDEV
# Requisitos: Privilégios de Superusuário (Root)
# Diretrizes: Bash strict mode (set -euo pipefail), idempotência total
# ==============================================================================

set -euo pipefail

readonly SCRIPT_NAME="$(basename "$0")"
readonly MODPROBE_BTUSB="/etc/modprobe.d/btusb.conf"
readonly MODPROBE_RTW88="/etc/modprobe.d/rtw88.conf"
readonly UDEV_RULE_FILE="/etc/udev/rules.d/50-bluetooth-realtek.rules"

readonly REALTEK_VENDOR="0bda"
readonly REALTEK_BT_PRODUCT="c829"

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
# Previne suspensão agressiva da placa combo Realtek RTL8821CE
options rtw88_pci disable_aspm=y
options rtw88_core disable_lps_deep=y
EOF

    log_success "Arquivos de modprobe configurados com sucesso."
}

# ------------------------------------------------------------------------------
# 2. Configuração de Regras de UDEV
# ------------------------------------------------------------------------------
configure_udev_rules() {
    log_info "Criando regra UDEV em ${UDEV_RULE_FILE}..."
    cat << EOF > "${UDEV_RULE_FILE}"
# Desativa economia de energia em tempo de execução para Realtek Bluetooth (${REALTEK_VENDOR}:${REALTEK_BT_PRODUCT})
ACTION=="add", SUBSYSTEM=="usb", ATTR{idVendor}=="${REALTEK_VENDOR}", ATTR{idProduct}=="${REALTEK_BT_PRODUCT}", ATTR{power/control}="on"
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

            if [ "${v}" = "${REALTEK_VENDOR}" ] && [ "${p}" = "${REALTEK_BT_PRODUCT}" ]; then
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
        log_info "Dispositivo ${REALTEK_VENDOR}:${REALTEK_BT_PRODUCT} será ajustado automaticamente na próxima conexão."
    fi
}

# ------------------------------------------------------------------------------
# Entrypoint
# ------------------------------------------------------------------------------
main() {
    log_info "=== Aplicando Otimizações de Kernel e UDEV para Realtek Bluetooth ==="
    assert_root
    configure_kernel_modules
    configure_udev_rules
    enforce_sysfs_runtime_power
    log_success "=== Sistema completamente ajustado e persistente! ==="
}

main "$@"
