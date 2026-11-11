#!/usr/bin/env bash
# ==============================================================================
# Script: rollback.sh
# Finalidade: Reversão completa das configurações de Kernel (modprobe) e UDEV
# Requisitos: Privilégios de Superusuário (Root)
# Diretrizes: Bash strict mode (set -euo pipefail)
# ==============================================================================

set -euo pipefail

readonly SCRIPT_NAME="$(basename "$0")"
readonly MODPROBE_BTUSB="/etc/modprobe.d/btusb.conf"
readonly MODPROBE_RTW88="/etc/modprobe.d/rtw88.conf"
readonly UDEV_RULE_FILE="/etc/udev/rules.d/50-bluetooth-realtek.rules"

log_info() {
    printf "\033[1;36m[INFO]\033[0m %s\n" "$*"
}

log_success() {
    printf "\033[1;32m[SUCESSO]\033[0m %s\n" "$*"
}

log_warn() {
    printf "\033[1;33m[AVISO]\033[0m %s\n" "$*"
}

log_error() {
    printf "\033[1;31m[ERRO]\033[0m %s\n" "$*" >&2
}

assert_root() {
    if [ "${EUID}" -ne 0 ]; then
        log_error "Este script de rollback de sistema requer privilégios de root."
        log_error "Por favor, execute: sudo $0"
        exit 1
    fi
}

main() {
    log_info "=== Iniciando Rollback do Sistema (Kernel & UDEV) ==="
    assert_root

    # Confirmação
    if [ -t 0 ]; then
        printf "\033[1;33mTem certeza de que deseja remover todas as regras de kernel e udev criadas? [s/N]: \033[0m"
        read -r resp
        case "${resp}" in
            [sS]|[sS][iI][mM]|[yY]|[yY][eE][sS]) ;;
            *) log_info "Rollback cancelado pelo usuário."; exit 0 ;;
        esac
    fi

    # 1. Remover arquivos modprobe
    for f in "${MODPROBE_BTUSB}" "${MODPROBE_RTW88}"; do
        if [ -f "$f" ]; then
            rm -f "$f"
            log_success "Removido: $f"
        else
            log_info "Arquivo não existia: $f"
        fi
    done

    # 2. Remover regra udev
    if [ -f "${UDEV_RULE_FILE}" ]; then
        rm -f "${UDEV_RULE_FILE}"
        log_success "Removido: ${UDEV_RULE_FILE}"
    fi

    # 3. Recarregar udev
    udevadm control --reload-rules
    udevadm trigger --subsystem-match=usb
    log_success "Regras UDEV recarregadas."

    log_success "=== Rollback do Sistema concluído com sucesso! ==="
}

main "$@"
