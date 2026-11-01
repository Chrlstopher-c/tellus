//! Journal d'événements du monde : les événements détectés attendent ici d'être envoyés à l'interface.

#[derive(Debug, Clone, PartialEq)]
pub struct WorldEvent {
    pub tick: u64,
    pub years: f64,
    pub kind: &'static str,
    pub title: String,
    pub body: String,
    pub cell: Option<u32>,
}

#[derive(Debug, Default)]
pub struct Journal {
    pending: Vec<WorldEvent>,
}

impl Journal {
    pub fn record(&mut self, event: WorldEvent) {
        self.pending.push(event);
    }

    /// Événements enregistrés depuis le dernier appel.
    pub fn drain(&mut self) -> Vec<WorldEvent> {
        std::mem::take(&mut self.pending)
    }
}
