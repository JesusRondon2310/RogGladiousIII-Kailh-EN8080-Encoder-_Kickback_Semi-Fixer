//! machine.rs


use crate::filter::constants::*;

/// 2.1.1. Estados de la máquina.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Silence,            // 3. Silencio inicial: bloquea sin inyectar.
    Vigilance,          // 4. Vigilancia: inyecta 1:1, observa si llega al techo.
    BlockNewAttempt,    // 5. Tras un corte: espera N ticks limpios antes de re-armar.
    Compensating,       // 6-7. Confirmado: sigue inyectando hasta objetivo.
    Normal,             // 8. Flujo normal: pasa todo directo.
}

/// 2.1.2. Acciones que puede tomar el hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Pass,               // Dejar pasar el tick (forward_to_next_hook).
    Block,              // Bloquear el tick sin inyectar.
    BlockAndCompensate, // Bloquear el tick y encolar un sintético.
}

/// 2.2. Decisión principal. variables 
pub fn decide_action(
    phase: Phase,
    streak: i32,
    compensation_count: i32,
    block_attempt_count: i32,
) -> (Phase, i32, Action) {
    use Phase::*;
    use Action::*;

    // 2.3. Match sobre (phase, streak)
    match (phase, streak) {
        // 2.3.1. Cambio de dirección: corta cualquier cosa en curso
        // (Vigilancia o Compensando) y pasa a BlockNewAttempt.
        (Vigilance, 1) | (Compensating, 1) => {
            // 5. Corte antes del techo (o durante compensación)
            // Se descarta todo, se detiene la inyección.El tick del corte se bloquea sin compensar.
            (BlockNewAttempt, 0, Block)
        }
        // Si estamos en Silence o Normal y la racha se reinicia, volvemos a Silence.
        (Silence, 1) | (Normal, 1) => {
            (Silence, 0, Block)
        }

        // 2.3.2. Silencio inicial (Phase::Silence)
        (Silence, s) if s < WATCH_THRESHOLD => {
            // 3. Aún no se alcanzó el umbral: bloquea sin inyectar.
            (Silence, 0, Block)
        }
        (Silence, s) if s >= WATCH_THRESHOLD => {
            // 4. Se alcanzó el umbral: arranca vigilancia.
            // Este tick se bloquea y se inyecta un sintético.
            (Vigilance, 0, BlockAndCompensate)
        }

        // 2.3.3. Vigilancia 
        (Vigilance, s) if s < KICKBACK_CEILING => {
            // Aún no se superó el techo: sigue inyectando.
            (Vigilance, 0, BlockAndCompensate)
        }
        (Vigilance, s) if s >= KICKBACK_CEILING => {
            // la racha superó el techo. Se cambia a Compensando y se continúa inyectando.
            (Compensating, 0, BlockAndCompensate)
        }

        // 2.3.4. BlockNewAttempt (tras un corte)
        (BlockNewAttempt, _) => {
            // Esperamos N ticks limpios antes de re-armar.
            let new_count = block_attempt_count + 1;
            if new_count < BLOCK_NEW_ATTEMPT {
                // Aún no se completó la espera: seguir bloqueando sin inyectar.
                (BlockNewAttempt, new_count, Block)
            } else {
                // Se completó la espera: arranca vigilancia directamente.
                // Este tick se bloquea y se inyecta un sintético.
                (Vigilance, 0, BlockAndCompensate)
            }
        }

        // 2.3.5. Compensando (Phase::Compensating)
        (Compensating, _) => {
            // Seguir compensando hasta alcanzar el objetivo.
            let new_comp = compensation_count + 1; // cada tick bloqueado + inyectado suma 1
            if new_comp < COMPENSATION_TARGET {
                (Compensating, new_comp, BlockAndCompensate)
            } else {
                // Objetivo alcanzado: pasar a flujo normal.
                (Normal, 0, Pass)
            }
        }

        // 2.3.6. Normal (Phase::Normal)
        (Normal, _) => {
            // todo pasa directo.
            // Si cambia la dirección, ya se capturó en (Normal, 1) arriba.
            (Normal, 0, Pass)
        }


        _ => {
            // Por seguridad, si algún caso no está cubierto, reiniciamos a Silence.
            eprintln!("[machine] Estado no cubierto: {:?}, streak={}", phase, streak);
            (Silence, 0, Block)
        }
    }
}

/// 2.4. Función de reinicio global (resetea todos los contadores).
/// Debe llamarse desde el orquestador cuando se quiera reiniciar el filtro.
pub fn reset_machine() -> (Phase, i32, i32) {
    (Phase::Silence, 0, 0)
}