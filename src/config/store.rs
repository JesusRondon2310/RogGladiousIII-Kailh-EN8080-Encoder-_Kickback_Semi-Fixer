//!config/store.rs

use std::fs;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use serde::Deserialize;

const PATH: &str = "config.toml";
const DEFAULT_TOML: &str = "\
# Kickback_Fix — configuración
# Edita este archivo y guarda; los cambios se aplican solos, sin reiniciar el programa.

Filtro = true                  # true = filtro activado, false = lo apaga (el programa se cierra solo al detectar el cambio)
SilencioInicial = 3            # ticks bloqueados en silencio antes de empezar a compensar (0 = sin silencio)
TechoKickback = 7              # racha total a la que la dirección se da por confirmada (0 = sin compensación por inyección)
";

static FILTRO: AtomicBool = AtomicBool::new(true);
static SILENCIO_INICIAL: AtomicI32 = AtomicI32::new(3);
static TECHO_KICKBACK: AtomicI32 = AtomicI32::new(7);

#[derive(Deserialize, Default)]
struct ConfigFile {
    #[serde(rename = "Filtro")]
    filtro: bool,
    #[serde(rename = "SilencioInicial")]
    silencio_inicial: i32,
    #[serde(rename = "TechoKickback")]
    techo_kickback: i32,
}

// file_load: lee config.toml; si no existe, lo crea con los valores por defecto comentados.
fn file_load() -> ConfigFile {
    let text = fs::read_to_string(PATH).unwrap_or_else(|_| {
        let _ = fs::write(PATH, DEFAULT_TOML);
        DEFAULT_TOML.to_string()
    });

    let mut cfg: ConfigFile = toml::from_str(&text).unwrap_or_default();
    cfg.silencio_inicial = cfg.silencio_inicial.max(0);
    cfg.techo_kickback = cfg.techo_kickback.max(0);
    cfg
}

// file_modified: fecha de modificación de config.toml, para que el vigilante detecte cambios sin releer el archivo entero cada vez.
fn file_modified() -> Option<SystemTime> {
    fs::metadata(PATH).ok()?.modified().ok()
}

// state_set: vuelca los valores recién leídos del archivo al estado compartido en memoria.
fn state_set(cfg: &ConfigFile) {
    FILTRO.store(cfg.filtro, Ordering::SeqCst);
    SILENCIO_INICIAL.store(cfg.silencio_inicial, Ordering::SeqCst);
    TECHO_KICKBACK.store(cfg.techo_kickback, Ordering::SeqCst);
}

// watcher_start: vigila config.toml en un hilo aparte: cada segundo revisa su fecha de modificación, y solo si cambió,
// lo relee y actualiza el estado compartido.
fn watcher_start() {
    thread::spawn(|| {
        let mut last_modified = file_modified();
        loop {
            thread::sleep(Duration::from_secs(1));
            let modified = file_modified();
            if modified != last_modified {
                last_modified = modified;
                state_set(&file_load());
            }
        }
    });
}

// 1. Carga config.toml (creándolo con valores por defecto si no existe) y arranca el vigilante de cambios.
pub(super) fn init() {
    state_set(&file_load());
    watcher_start();
}

pub(super) fn filtro() -> bool {
    FILTRO.load(Ordering::SeqCst)
}

pub(super) fn silencio_inicial() -> i32 {
    SILENCIO_INICIAL.load(Ordering::SeqCst)
}

pub(super) fn techo_kickback() -> i32 {
    TECHO_KICKBACK.load(Ordering::SeqCst)
}
