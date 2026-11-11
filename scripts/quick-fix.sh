#!/usr/bin/env bash
# ==============================================================================
# Script: quick-fix.sh
# Finalidade: Aplicação imediata das regras de estabilização de áudio no WirePlumber
# Diretrizes: POSIX-compliant / Bash strict mode (set -euo pipefail)
# ==============================================================================

set -euo pipefail

readonly SCRIPT_NAME="$(basename "$0")"
readonly CONF_DIR="${HOME}/.config/wireplumber/wireplumber.conf.d"
readonly TARGET_CONFIG="${CONF_DIR}/80-bluetooth-tweaks.conf"
readonly BACKUP_CONFIG="${TARGET_CONFIG}.bak"

# ------------------------------------------------------------------------------
# Funções Utilitárias de Logging
# ------------------------------------------------------------------------------
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

# ------------------------------------------------------------------------------
# 1. Validação de Pré-requisitos
# ------------------------------------------------------------------------------
check_prerequisites() {
    local missing=0
    for cmd in pactl systemctl; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            log_error "Comando essencial não encontrado no PATH: $cmd"
            missing=1
        fi
    done
    if [ "$missing" -ne 0 ]; then
        exit 1
    fi
}

# ------------------------------------------------------------------------------
# 2. Backup de Configuração Existente
# ------------------------------------------------------------------------------
backup_existing_config() {
    mkdir -p "${CONF_DIR}"
    if [ -f "${TARGET_CONFIG}" ] && [ ! -f "${BACKUP_CONFIG}" ]; then
        cp "${TARGET_CONFIG}" "${BACKUP_CONFIG}"
        log_info "Backup preventivo criado em: ${BACKUP_CONFIG}"
    fi
}

# ------------------------------------------------------------------------------
# 3. Gravação da Configuração WirePlumber 0.5
# ------------------------------------------------------------------------------
deploy_wireplumber_rules() {
    log_info "Gravando configurações de baixa latência e estabilidade..."

    cat << 'EOF' > "${TARGET_CONFIG}"
# =====================================================================
# Configuração de Alta Estabilidade Bluetooth - Combo Wi-Fi/BT + Fones TWS
# Gerada automaticamente por bt-dell-audio / quick-fix.sh
# =====================================================================

wireplumber.settings = {
  # Desativa troca automática para perfil de headset (mono 8kHz).
  # Evita que navegadores, jogos ou apps de voz degradem o som para qualidade de telefone.
  bluetooth.autoswitch-to-headset-profile = false
}

monitor.bluez.rules = [
  {
    matches = [
      {
        device.name = "~bluez_card.*"
      }
    ]
    actions = {
      update-props = {
        # CRUCIAL: Desativa Volume Absoluto de Hardware (AVRCP).
        # Fones TWS possuem falha crônica no DSP ao sincronizar ganho de hardware.
        # Desativar isso elimina quedas abruptas de volume e ganho!
        bluez5.enable-hw-volume = false

        # Desativa codec SBC-XQ com bitrate excessivo (~500kbps) que congestiona
        # antenas compartilhadas 1x1 com Wi-Fi.
        bluez5.enable-sbc-xq = false

        # Prioriza codecs estáveis (SBC e AAC).
        bluez5.codecs = [ sbc aac ]

        # Restringe o dispositivo ao papel de saída estéreo de alta fidelidade (A2DP).
        # Previne abertura acidental de canais duplex degradados de voz.
        bluez5.roles = [ a2dp_sink ]

        # Desativa suspensão por inatividade do nó Bluetooth para prevenir estalos
        session.suspend-timeout-seconds = 0
      }
    }
  }
]
EOF

    log_success "Regras implantadas com sucesso em: ${TARGET_CONFIG}"
}

# ------------------------------------------------------------------------------
# 4. Reinicialização de Serviços e Roteamento de Fluxos
# ------------------------------------------------------------------------------
reload_audio_stack() {
    log_info "Reiniciando sessão do WirePlumber..."
    systemctl --user restart wireplumber
    sleep 0.5

    # Chavear cartões conectados para perfil A2DP estável
    local card
    for card in $(pactl list cards short | awk '/bluez_card/ {print $2}'); do
        log_info "Ativando perfil estéreo estável (a2dp-sink-sbc) no dispositivo: ${card}"
        pactl set-card-profile "${card}" a2dp-sink-sbc || true

        # Re-ancorar sink padrão e migrar fluxos
        local sink_name
        sink_name="$(pactl list sinks short | awk -v c="${card}" '$2 ~ c {print $2}' | head -n 1)"
        if [ -n "${sink_name}" ]; then
            pactl set-default-sink "${sink_name}" || true
            for stream in $(pactl list sink-inputs short | awk '{print $1}'); do
                pactl move-sink-input "${stream}" "${sink_name}" 2>/dev/null || true
            done
        fi
    done
}

# ------------------------------------------------------------------------------
# Função Principal (Entrypoint)
# ------------------------------------------------------------------------------
main() {
    log_info "=== Iniciando Estabilização Imediata de Áudio (WirePlumber 0.5) ==="
    check_prerequisites
    backup_existing_config
    deploy_wireplumber_rules
    reload_audio_stack
    log_success "=== Correções aplicadas com sucesso! ==="
}

main "$@"
